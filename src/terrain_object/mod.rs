use cgmath::Point2;
use gfx;
use specs;
use specs::prelude::{Read, ReadStorage, WriteStorage};

use crate::character::controls::CharacterInputState;
use crate::game::constants::{ASPECT_RATIO, VIEW_DISTANCE};
use crate::gfx_app::{ColorFormat, DepthFormat};
use crate::graphics::{camera::CameraInputState, dimensions::{Dimensions, get_projection, get_view_matrix}, texture::load_texture};
use crate::graphics::mesh::{RectangularTexturedMesh, Geometry};
use crate::graphics::texture::Texture;
use crate::shaders::{Position, Projection, static_element_pipeline, Time};
use crate::terrain_object::terrain_objects::TerrainObjects;

pub mod terrain_objects;

const SHADER_VERT: &[u8] = include_bytes!("../shaders/static_element.v.glsl");
const SHADER_FRAG: &[u8] = include_bytes!("../shaders/static_element.f.glsl");

pub struct TerrainObjectDrawable {
  projection: Projection,
  pub position: Position,
  previous_position: Position,
  pub object_type: TerrainTexture,
}

impl TerrainObjectDrawable {
  pub fn new(position: Position, object_type: TerrainTexture) -> TerrainObjectDrawable {
    let view = get_view_matrix(VIEW_DISTANCE);
    let projection = get_projection(view, ASPECT_RATIO);
    TerrainObjectDrawable {
      projection,
      position,
      previous_position: Position::origin(),
      object_type,
    }
  }

  pub fn update(&mut self, world_to_clip: &Projection, ci: &CharacterInputState) {
    self.projection = *world_to_clip;
    self.position = self.position + ci.movement - self.previous_position;
    self.previous_position = ci.movement;
  }
}

impl specs::prelude::Component for TerrainObjectDrawable {
  type Storage = specs::storage::VecStorage<TerrainObjectDrawable>;
}

#[derive(Clone, Copy, PartialEq)]
pub enum TerrainTexture {
  House,
  Tree,
  Ammo,
}

pub struct TerrainObjectDrawSystem<R: gfx::Resources> {
  bundle: gfx::pso::bundle::Bundle<R, static_element_pipeline::Data<R>>,
}

impl<R: gfx::Resources> TerrainObjectDrawSystem<R> {
  pub fn new<F>(factory: &mut F,
                rtv: gfx::handle::RenderTargetView<R, ColorFormat>,
                dsv: gfx::handle::DepthStencilView<R, DepthFormat>,
                texture: TerrainTexture) -> TerrainObjectDrawSystem<R>
    where F: gfx::Factory<R> {
    use gfx::traits::FactoryExt;

    let (texture_size, texture_bytes) = match texture {
      TerrainTexture::Ammo => (Point2::new(5.0, 7.0), &include_bytes!("../../assets/maps/ammo.png")[..]),
      TerrainTexture::House => (Point2::new(125.0, 125.0), &include_bytes!("../../assets/maps/house.png")[..]),
      TerrainTexture::Tree => (Point2::new(120.0, 120.0), &include_bytes!("../../assets/maps/tree.png")[..]),
    };

    let terrain_object_texture = load_texture(factory, texture_bytes);

    let mesh = RectangularTexturedMesh::new(factory, Texture::new(terrain_object_texture, None), Geometry::Rectangle, texture_size, None, None, None);

    let pso = factory.create_pipeline_simple(SHADER_VERT, SHADER_FRAG, static_element_pipeline::new())
      .expect("Terrain object shader loading error");

    let pipeline_data = static_element_pipeline::Data {
      vbuf: mesh.mesh.vertex_buffer,
      position_cb: factory.create_constant_buffer(1),
      time_passed_cb: factory.create_constant_buffer(1),
      projection_cb: factory.create_constant_buffer(1),
      static_element_sheet: (mesh.mesh.texture.raw, factory.create_sampler_linear()),
      out_color: rtv,
      out_depth: dsv,
    };

    TerrainObjectDrawSystem {
      bundle: gfx::Bundle::new(mesh.mesh.slice, pso, pipeline_data),
    }
  }

  pub fn draw<C>(&self,
                 drawable: &TerrainObjectDrawable,
                 time_passed: u64,
                 encoder: &mut gfx::Encoder<R, C>)
    where C: gfx::CommandBuffer<R> {
    encoder.update_constant_buffer(&self.bundle.data.projection_cb, &drawable.projection);
    encoder.update_constant_buffer(&self.bundle.data.position_cb, &drawable.position);
    encoder.update_constant_buffer(&self.bundle.data.time_passed_cb, &Time::new(time_passed));
    self.bundle.encode(encoder);
  }
}

pub struct PreDrawSystem;

impl<'a> specs::prelude::System<'a> for PreDrawSystem {
  type SystemData = (ReadStorage<'a, CameraInputState>,
                     ReadStorage<'a, CharacterInputState>,
                     WriteStorage<'a, TerrainObjects>,
                     Read<'a, Dimensions>);

  fn run(&mut self, (camera_input, character_input, mut terrain_objects, dim): Self::SystemData) {
    use specs::join::Join;

    for (camera, ci, obj) in (&camera_input, &character_input, &mut terrain_objects).join() {
      let world_to_clip = dim.world_to_projection(camera);

      for o in &mut obj.objects {
        o.update(&world_to_clip, ci);
      }
    }
  }
}
