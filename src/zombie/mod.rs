use cgmath;
use cgmath::Matrix4;
use character::controls::CharacterInputState;
use gfx;
use gfx_app::{ColorFormat, DepthFormat};
use graphics::orientation::{Orientation, Stance};
use graphics::{Dimensions, load_texture};
use graphics::camera::CameraInputState;
use game::constants::{ASPECT_RATIO, ZOMBIESHEET_TOTAL_WIDTH, SPRITE_OFFSET, STILL_SPRITE_OFFSET};
use critter::{CritterData, ZombieSprite};
use shaders::{critter_pipeline, VertexData, CharacterSheet, Position, Projection};
use specs;
use specs::{Fetch, ReadStorage, WriteStorage};
use data;

const SHADER_VERT: &[u8] = include_bytes!("../shaders/character.v.glsl");
const SHADER_FRAG: &[u8] = include_bytes!("../shaders/character.f.glsl");

const ZOMBIE_START_POSITION: (f32, f32) = (256.0, -32.0);

#[derive(Debug)]
pub struct ZombieDrawable {
  projection: Projection,
  position: Position,
  orientation: Orientation,
  pub stance: Stance,
  direction: Orientation,
}

impl ZombieDrawable {
  pub fn new(view: Matrix4<f32>) -> ZombieDrawable {
    ZombieDrawable {
      projection: Projection {
        model: view.into(),
        view: view.into(),
        proj: cgmath::perspective(cgmath::Deg(60.0f32), ASPECT_RATIO, 0.1, 4000.0).into(),
      },
      position: Position {
        position: [256.0, 0.0],
      },
      orientation: Orientation::Left,
      stance: Stance::Still,
      direction: Orientation::Left,
    }
  }

  pub fn update(&mut self, world_to_clip: &Projection, ci: &CharacterInputState) {
    self.projection = *world_to_clip;
    self.stance = Stance::Still;
    self.position = Position {
      position: [ZOMBIE_START_POSITION.0 + ci.x_movement, ZOMBIE_START_POSITION.1 + ci.y_movement]
    };
  }
}

impl specs::Component for ZombieDrawable {
  type Storage = specs::VecStorage<ZombieDrawable>;
}

pub struct ZombieDrawSystem<R: gfx::Resources> {
  bundle: gfx::pso::bundle::Bundle<R, critter_pipeline::Data<R>>,
  data: Vec<CritterData>,
}

impl<R: gfx::Resources> ZombieDrawSystem<R> {
  pub fn new<F>(factory: &mut F,
                rtv: gfx::handle::RenderTargetView<R, ColorFormat>,
                dsv: gfx::handle::DepthStencilView<R, DepthFormat>) -> ZombieDrawSystem<R>
    where F: gfx::Factory<R> {
    use gfx::traits::FactoryExt;

    let zombie_bytes = include_bytes!("../../assets/zombie.png");

    let vertex_data: Vec<VertexData> =
      vec![
        VertexData::new([-25.0, -35.0, 0.0], [0.0, 1.0]),
        VertexData::new([25.0, -35.0, 0.0], [1.0, 1.0]),
        VertexData::new([25.0, 35.0, 0.0], [1.0, 0.0]),
        VertexData::new([-25.0, -35.0, 0.0], [0.0, 1.0]),
        VertexData::new([25.0, 35.0, 0.0], [1.0, 0.0]),
        VertexData::new([-25.0, 35.0, 0.0], [0.0, 0.0]),
      ];

    let (vertex_buf, slice) = factory.create_vertex_buffer_with_slice(&vertex_data, ());

    let char_texture = load_texture(factory, zombie_bytes).unwrap();
    let pso = factory
      .create_pipeline_simple(SHADER_VERT,
                              SHADER_FRAG,
                              critter_pipeline::new())
      .unwrap();

    let pipeline_data = critter_pipeline::Data {
      vbuf: vertex_buf,
      projection_cb: factory.create_constant_buffer(1),
      position_cb: factory.create_constant_buffer(1),
      character_sprite_cb: factory.create_constant_buffer(1),
      charactersheet: (char_texture, factory.create_sampler_linear()),
      out_color: rtv,
      out_depth: dsv,
    };

    let data = data::load_zombie();

    ZombieDrawSystem {
      bundle: gfx::Bundle::new(slice, pso, pipeline_data),
      data
    }
  }

  fn get_next_sprite(&self, zombie_idx: usize, drawable: &mut ZombieDrawable) -> CharacterSheet {
    let zombie_sprite =
      if drawable.stance == Stance::Still {
        let sprite_idx = (drawable.direction as usize * 4 + zombie_idx) as usize;
        (&self.data[sprite_idx], sprite_idx)
      } else if drawable.orientation != Orientation::Still && drawable.stance == Stance::Walking {
        let sprite_idx = (drawable.direction as usize * 8 + zombie_idx + STILL_SPRITE_OFFSET) as usize;
        (&self.data[sprite_idx], sprite_idx)
      } else {
        drawable.direction = drawable.orientation;
        let sprite_idx = (drawable.orientation as usize * 8 + zombie_idx + STILL_SPRITE_OFFSET) as usize;
        (&self.data[sprite_idx], sprite_idx)
      };

    let elements_x = ZOMBIESHEET_TOTAL_WIDTH / (zombie_sprite.0.data[2] + SPRITE_OFFSET);
    CharacterSheet {
      div: elements_x,
      index: zombie_sprite.1 as f32
    }
  }

  pub fn draw<C>(&mut self,
                 mut drawable: &mut ZombieDrawable,
                 zombie: &ZombieSprite,
                 encoder: &mut gfx::Encoder<R, C>)
    where C: gfx::CommandBuffer<R> {
    encoder.update_constant_buffer(&self.bundle.data.projection_cb, &drawable.projection);
    encoder.update_constant_buffer(&self.bundle.data.position_cb, &drawable.position);
    encoder.update_constant_buffer(&self.bundle.data.character_sprite_cb,
                                   &self.get_next_sprite(zombie.zombie_idx, &mut drawable));
    self.bundle.encode(encoder);
  }
}

#[derive(Debug)]
pub struct PreDrawSystem;

impl PreDrawSystem {
  pub fn new() -> PreDrawSystem {
    PreDrawSystem {}
  }
}

impl<'a> specs::System<'a> for PreDrawSystem {
  type SystemData = (WriteStorage<'a, ZombieDrawable>,
                     ReadStorage<'a, CameraInputState>,
                     ReadStorage<'a, CharacterInputState>,
                     Fetch<'a, Dimensions>);


  fn run(&mut self, (mut zombie, camera_input, character_input, dim): Self::SystemData) {
    use specs::Join;

    for (z, camera, ci) in (&mut zombie, &camera_input, &character_input).join() {
      let world_to_clip = dim.world_to_projection(camera);
      z.update(&world_to_clip, ci);
    }
  }
}
