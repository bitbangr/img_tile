
use serde::{Deserialize, Serialize};
use std::{error::Error, path::Path};
use std::fs::File;
use std::io::Write;

use crate::modtile::{self, RGB};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    tiles: Vec<Vec<RGB>>,
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
// struct RGB {
//     r: u8,
//     g: u8,
//     b: u8,
// }

fn main() -> Result<(), Box<dyn Error>> {
    // Given input data
    let data: Vec<RGB> = vec![
        RGB { 0: 255, 1: 0, 2: 0 },
        RGB { 0: 0, 1: 255, 2: 0 },
        RGB { 0: 0, 1: 0, 2: 255 },
        RGB { 0: 255, 1: 255, 2: 255 },
        RGB { 0: 128, 1: 128, 2: 128 },
        RGB { 0: 255, 1: 255, 2: 0 },
        RGB { 0: 0, 1: 255, 2: 255 },
        RGB { 0: 255, 1: 0, 2: 255 },
    ];

    let row = 2;
    let col = 4;

    let mut output_tiles = vec![vec![RGB { 0: 0, 1: 0, 2: 0 }; col]; row];

    for (i, color) in data.into_iter().enumerate() {
        let row_idx = i / col;
        let col_idx = i % col;
        output_tiles[row_idx][col_idx] = color;
    }

    let config = Config {
        tiles: output_tiles,
    };

    let serialized = serde_json::to_string(&config)?;
    let mut file = File::create("output.json")?;
    file.write_all(serialized.as_bytes())?;

    Ok(())
}

/// Save the RGB values into an output_width_tile_count x output_height_tile_count array
/// That can be read in by hack-svg or other program to construct input data
pub fn dump_rgb_json(output_window: &Vec<Vec<(euclid::Box2D<i32, i32>, modtile::RGB)>>, 
            output_width_tile_count: usize, 
            output_height_tile_count: usize, 
            tiles_per_pane_width: usize, 
            tiles_per_pane_height: usize,
            save_path: &Path) -> Result <(), Box<dyn Error>> 
{
    println!("\n dump_rbg_json *********");
    println!(" output_width_tile_count {} ", output_width_tile_count);
    println!("output_height_tile_count {} ", output_height_tile_count);
    println!(" tiles_per_pane_width {} ", tiles_per_pane_width);
    println!("tiles_per_pane_height {} ", tiles_per_pane_height);

    println!("input_window.len {} ", output_window.len());

    let mut output_tiles = vec![vec![RGB { 0: 0, 1: 0, 2: 0 }; output_width_tile_count ]; output_height_tile_count];

    // grab first pane
    let first_pane = &output_window[0];

    println!("first pane len = {:?}", &first_pane.len());

    for (i, color) in first_pane.iter().enumerate() {
        let row_idx = i / output_width_tile_count  ;
        let col_idx = i % output_width_tile_count ;
        // println!("i: {} , row_idx:{:?} , col_idx:{:?}",&i,  &row_idx,&col_idx);
        output_tiles[row_idx][col_idx] = color.1;
    }

    let config = Config {
        tiles: output_tiles,
    };

    let file_path = save_path.with_extension("json");
    let mut file = File::create(file_path)?;
    let serialized = serde_json::to_string(&config)?;
    file.write_all(serialized.as_bytes())?;

    Ok(())
}

fn _dump_info(input_window: &Vec<Vec<(euclid::Box2D<i32, i32>, modtile::RGB)>>)
 {
    println!("\n YO ********* \n\n {:?} \n ********* \n\n", input_window);
}


// save the tile color data to an output file
// todo need throw exception for any output errors
// pub fn save_output_json<P: AsRef<Path>>(path: P, config: &Config) -> Result<()> {
//     let mut f = File::create(path)?;
//     let buf = serde_json::to_vec(&config)?;
//     f.write_all(&buf[..])?;
//     Ok(())
// }
//