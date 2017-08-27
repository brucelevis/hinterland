use terrain::gfx_macros::{TileMapData};
use data::{load_map_file, get_map_tile};
use game::constants::{MAP_FILE_PATH, TILEMAP_BUF_LENGTH, TILES_PCS_W, TILES_PCS_H};

#[derive(Debug)]
pub struct Terrain {
  pub tiles: Vec<TileMapData>
}

fn calc_index(xpos: usize, ypos: usize) -> usize {
  (ypos * TILES_PCS_W) + xpos
}

fn populate_tilemap(mut tiles: Vec<TileMapData>) -> Vec<TileMapData> {
  let map = load_map_file(MAP_FILE_PATH);
  for ypos in 0..TILES_PCS_H {
    for xpos in 0..TILES_PCS_W {
      let map_val = get_map_tile(&map, 0, xpos as usize, ypos as usize);
      let tex_x = map_val % (TILES_PCS_W as u32);
      let tex_y = (map_val - tex_x) / (TILES_PCS_W as u32);
      let idx = calc_index(xpos, ypos);
      tiles[idx] = TileMapData::new([(tex_x-1) as f32, tex_y as f32, 0.0, 0.0]);
    }
  }
  tiles
}

pub fn generate() -> Terrain {
  let mut charmap_data = Vec::with_capacity(TILEMAP_BUF_LENGTH);

  for _ in 0..TILEMAP_BUF_LENGTH {
    charmap_data.push(TileMapData::new_empty());
  }

  Terrain {
    tiles: populate_tilemap(charmap_data)
  }
}