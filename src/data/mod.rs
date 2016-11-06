use std::io::prelude::*;
use std::fs::File;
use std::error::Error;
use std::string::String;
use std::path::Path;
use std::vec::Vec;
use game::data::Rectangle;
use game::gfx::{CopySprite, Sprite};
use json::{JsonValue, parse};

fn read_sprite_file(filename: &str) -> String {
  let path = Path::new(&filename);
  let mut file = match File::open(&path) {
    Ok(f) => f,
    Err(e) => panic!("File {} not found: {}", filename, e),
  };
  let mut buf = String::new();
  match file.read_to_string(&mut buf) {
    Ok(_) => buf,
    Err(_) => panic!("Couldn't read file {}", filename),
  }
}

pub fn load_character() -> Vec<Rectangle> {
  let mut sprites = Vec::with_capacity(224);
  let mut sprite_names = Vec::with_capacity(14);
  let character_json = read_sprite_file("./assets/character.json");
  let character = match parse(&character_json) {
    Ok(res) => res,
    Err(e) => panic!("Character JSON parse error {:?}", e),
  };
  for x in 1..15 {
    let i = if x < 10 { format!("0{}", x) } else { format!("{}", x) };
    sprite_names.push(format!("Jog_45_{}", i));
    sprite_names.push(format!("Jog_135_{}", i));
    sprite_names.push(format!("Jog_225_{}", i));
    sprite_names.push(format!("Jog_315_{}", i));
  }
  for &ref sprite in &sprite_names {
    let x = character[sprite]["frame"]["x"].as_f64();
    let y = character[sprite]["frame"]["y"].as_f64();
    let w = character[sprite]["frame"]["w"].as_f64();
    let h = character[sprite]["frame"]["h"].as_f64();
    sprites.push(Rectangle {
      w: w.unwrap() as f64,
      h: h.unwrap() as f64,
      x: x.unwrap() as f64,
      y: y.unwrap() as f64,
    });
  }
  sprites
}

