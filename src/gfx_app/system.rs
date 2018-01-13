use gfx_app::{ColorFormat, DepthFormat};
use gfx_app::renderer::EncoderQueue;
use gfx;
use bullet;
use terrain;
use character;
use zombie;
use specs;
use specs::{Fetch, WriteStorage};
use std::time::Instant;
use critter::{CharacterSprite, ZombieSprite};
use graphics::orientation::Stance;
use graphics::DeltaTime;

pub struct DrawSystem<D: gfx::Device> {
  render_target_view: gfx::handle::RenderTargetView<D::Resources, ColorFormat>,
  depth_stencil_view: gfx::handle::DepthStencilView<D::Resources, DepthFormat>,
  terrain_system: terrain::TerrainDrawSystem<D::Resources>,
  character_system: character::CharacterDrawSystem<D::Resources>,
  zombie_system: zombie::ZombieDrawSystem<D::Resources>,
  bullet_system: bullet::BulletDrawSystem<D::Resources>,
  encoder_queue: EncoderQueue<D>,
  game_time: Instant,
  frames: u32,
  cool_down: f64,
  fire_cool_down: f64,
}

impl<D: gfx::Device> DrawSystem<D> {
  pub fn new<F>(factory: &mut F,
                rtv: &gfx::handle::RenderTargetView<D::Resources, ColorFormat>,
                dsv: &gfx::handle::DepthStencilView<D::Resources, DepthFormat>,
                encoder_queue: EncoderQueue<D>)
                -> DrawSystem<D>
    where F: gfx::Factory<D::Resources>
  {
    DrawSystem {
      render_target_view: rtv.clone(),
      depth_stencil_view: dsv.clone(),
      terrain_system: terrain::TerrainDrawSystem::new(factory, rtv.clone(), dsv.clone()),
      character_system: character::CharacterDrawSystem::new(factory, rtv.clone(), dsv.clone()),
      zombie_system: zombie::ZombieDrawSystem::new(factory, rtv.clone(), dsv.clone()),
      bullet_system: bullet::BulletDrawSystem::new(factory, rtv.clone(), dsv.clone()),
      encoder_queue,
      game_time: Instant::now(),
      frames: 0,
      cool_down: 1.0,
      fire_cool_down: 1.0
    }
  }
}

impl<'a, D> specs::System<'a> for DrawSystem<D>
  where D: gfx::Device,
        D::CommandBuffer: Send,
{
  #[cfg_attr(feature = "cargo-clippy", allow(type_complexity))]
  type SystemData = (WriteStorage<'a, terrain::TerrainDrawable>,
                     WriteStorage<'a, character::CharacterDrawable>,
                     WriteStorage<'a, CharacterSprite>,
                     WriteStorage<'a, zombie::ZombieDrawable>,
                     WriteStorage<'a, ZombieSprite>,
                     WriteStorage<'a, bullet::BulletDrawable>,
                     Fetch<'a, DeltaTime>);

  fn run(&mut self, (mut terrain, mut character, mut character_sprite, mut zombie, mut zombie_sprite, mut bullets, d): Self::SystemData) {
    use specs::Join;
    let mut encoder = self.encoder_queue.receiver.recv().unwrap();

    let delta = d.0;

    if self.cool_down == 0.0 {
      self.cool_down += 0.07;
    }
    if self.fire_cool_down == 0.0 {
      self.fire_cool_down += 0.2;
    }
    self.cool_down = (self.cool_down - delta).max(0.0);
    self.fire_cool_down = (self.fire_cool_down - delta).max(0.0);

    let current_time = Instant::now();
    self.frames += 1;
    if cfg!(feature = "fps") && (current_time.duration_since(self.game_time).as_secs()) >= 1 {
      println!("{:?} ms/frames", 1000.0 / f64::from(self.frames));
      self.frames = 0;
      self.game_time = Instant::now();
    }

    encoder.clear(&self.render_target_view, [16.0 / 256.0, 16.0 / 256.0, 20.0 / 256.0, 1.0]);
    encoder.clear_depth(&self.depth_stencil_view, 1.0);

    for (t, c, cs, z, zs, b) in (&mut terrain, &mut character, &mut character_sprite, &mut zombie, &mut zombie_sprite, &mut bullets).join() {
      self.terrain_system.draw(t, &mut encoder);

      if self.cool_down == 0.0 {
        if c.stance == Stance::Walking {
          cs.update_run();
        }
        if z.stance == Stance::NormalDeath {
          zs.update_normal_death();
        }
        if z.stance == Stance::Walking {
          zs.update_walk();
        }
        if z.stance == Stance::Still {
          zs.update_still();
        }
      } else if self.fire_cool_down == 0.0 && c.stance == Stance::Firing {
        cs.update_fire();
      }
      self.character_system.draw(c, cs, &mut encoder);
      self.zombie_system.draw(z, zs, &mut encoder);
      self.bullet_system.draw(b, &mut encoder);
    }

    if let Err(e) = self.encoder_queue.sender.send(encoder) {
      panic!("Disconnected, cannot return encoder to mpsc: {}", e);
    };
  }
}
