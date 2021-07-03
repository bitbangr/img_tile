use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;
use std::io::{Read, Write, Result};
use std::fmt::{self, Formatter, Display};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Config{
    pub tile_colors: String,// "./json_files/crayola_colors.json",
    pub input: String,      // :"./images/4x4_16_color_test.png",
    pub output: String,     // ./images/output/4x4_cray_15x15.jpg",
    pub output_width: f64,//600,
    pub output_height: f64, //600,
    pub tile_size_x:  f64,   //10,
    pub tile_size_y: f64,   //10,
    pub tile_space_x: f64,
    pub tile_space_y: f64,
    pub tiles_per_pane_width: usize,  //4,
    pub tiles_per_pane_height: usize,  //4,
}

pub fn load_configs(path_str: &str) -> Config {

    let path = Path::new(path_str);

    match File::open(path) {
       Ok(mut file) => {
           println!("Config File was successfully opened");
           let mut buf = vec![];
           if file.read_to_end(&mut buf).is_ok() {
               match serde_json::from_slice(&buf[..]) {
                   Ok(config) => return config,
                   Err(e) => {
                       eprintln!("Could not read config file {:?} \n  {}", path, e );
                       panic!("Improperly formed JSON file");
                   }
               } // match serde_json
           }
           else {
               eprintln!("Could not read config file {:?}", path);
               panic!("Improperly formed JSON file");
           }
       },
       Err(e) => {
           eprintln!(" Could not open config file {:?} \n  {}", path, e );
           panic!("Missing Config File");
        },
     }
}

pub fn create_and_save_test_config(path_str: &str) -> Config {

    let cfg = Config{
        tile_colors:"./tile_json/crayola_colors.json".to_owned(),
        input:"./images/4x4_16_color_test.png".to_owned(),
        output:"./images/output/4x4_cray_15x15.jpg".to_owned(),
        output_width:10.0,
        output_height:10.0,
        tile_size_x:2.0,
        tile_size_y:2.0,
        tile_space_x:1.0,
        tile_space_y:1.0,
        tiles_per_pane_width:3,
        tiles_per_pane_height:3,

    };

    let path = Path::new(path_str);
    match save_config(path, &cfg) {
        Ok(()) => cfg,
        Err(e) => { eprintln!("Could not write test config file {:?} \n  {}", path, e );
                    panic!("Error Writing Test Config File");}
    }
}


#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub struct RGB(pub u8,pub u8,pub u8);

impl Display for RGB {
    // `f` is a buffer, and this method must write the formatted string into it
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {

        // `write!` is like `format!`, but it will write the formatted string
        // into a buffer (the first argument)
        write!(f, "rgb ({}, {}, {})", self.0, self.1, self.2)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct TileColor {
    pub rgb: RGB,
    pub name: String,
    pub number: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct AllColors{
    pub name: String,
    pub url: String,
    pub description: String,
    pub colors: Vec<TileColor>
}
// Quick function to create populate and return an AllColors struct
pub fn test_allcolors_struct()  -> AllColors {

    let tc1 = TileColor {
        rgb: RGB(0,0,0),
        name: "black".to_owned(),
        number: "1".to_owned(),
    };
    let tc2 = TileColor {
        rgb: RGB(255,0,0),
        name: "red".to_owned(),
        number: "2".to_owned(),
    };
    let tc3 = TileColor {
        rgb: RGB(0,255,0),
        name: "blue".to_owned(),
        number: "3".to_owned(),
    };
    let tc4 = TileColor {
        rgb: RGB(0,0,255),
        name: "green".to_owned(),
        number: "4".to_owned(),
    };

    let v = vec![tc1,tc2,tc3,tc4];

    let ac = AllColors{ name:"Test".to_owned(),url:"none".to_owned(),description:"Test".to_owned(), colors: v};
    println!("{:?}",ac);

    // function return value
    ac
}


// load tile color files from a json file
// todo need throw exception for missing or deformed file
pub fn load_all_colors(path_str: &str) -> AllColors {
    let path = Path::new(path_str);

    match File::open(path) {
       Ok(mut file) => {
           println!();
           println!("Tile Colour File was successfully opened");
           let mut buf = vec![];
           if file.read_to_end(&mut buf).is_ok() {
               match serde_json::from_slice(&buf[..]) {
                   Ok(all_colors) => return all_colors,
                   Err(e) => {
                       eprintln!("Could not read Colour file {:?} \n  {}", path, e );
                       panic!("Improperly formed JSON file");
                   }
               } // match serde_json
           }
           else {
               eprintln!("Could not read tile colors file {:?}", path);
               panic!("Improperly formed JSON file");
           }
       },
       Err(e) => {
           eprintln!(" Could not open Colors file {:?} \n  {}", path, e );
           panic!("Missing Tile Colors File");
        },
     }

    // if let Ok(mut file) = File::open(path) {
    //     let mut buf = vec![];
    //     if file.read_to_end(&mut buf).is_ok() {
    //         if let Ok(all_colors) = serde_json::from_slice(&buf[..]) {
    //             return all_colors;
    //         }
    //     }
    // }
    // // todo  update error handling
    // // See load_configs
    // // There was no file, or the file failed to load, create a new All_Colors.
    // println!("no file, or the file failed to load, create a new All_Colors\n*****\n*****\nThere was a problem \n*****\n*****" );
    //
    // let tc1 = TileColor { rgb: RGB(0,0,0), name: "black".to_owned() , number: "0".to_owned() };
    // AllColors{name:"Hack".to_owned(),url:"none".to_owned(),description:"MadeUp".to_owned(), colors: vec![tc1] }
}


// load tile color files from a json file
// todo need throw exception for missing or deformed file
pub fn old_load_all_colors<P: AsRef<Path>>(path: P) -> AllColors {
    if let Ok(mut file) = File::open(path) {
        let mut buf = vec![];
        if file.read_to_end(&mut buf).is_ok() {
            if let Ok(all_colors) = serde_json::from_slice(&buf[..]) {
                return all_colors;
            }
        }
    }
    // todo  update error handling
    // See load_configs
    // There was no file, or the file failed to load, create a new All_Colors.
    println!("no file, or the file failed to load, create a new All_Colors\n*****\n*****\nThere was a problem \n*****\n*****" );

    let tc1 = TileColor { rgb: RGB(0,0,0), name: "black".to_owned() , number: "0".to_owned() };
    AllColors{name:"Hack".to_owned(),url:"none".to_owned(),description:"MadeUp".to_owned(), colors: vec![tc1] }
}

// save the tile color data to an output file
// todo need throw exception for any output errors
pub fn save_all_colors<P: AsRef<Path>>(path: P, all_colors: AllColors) -> Result<()> {
    let mut f = File::create(path)?;
    let buf = serde_json::to_vec(&all_colors)?;
    f.write_all(&buf[..])?;
    Ok(())
}

// save the tile color data to an output file
// todo need throw exception for any output errors
pub fn save_config<P: AsRef<Path>>(path: P, config: &Config) -> Result<()> {
    let mut f = File::create(path)?;
    let buf = serde_json::to_vec(&config)?;
    f.write_all(&buf[..])?;
    Ok(())
}
