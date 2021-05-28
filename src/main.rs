#[macro_use]
extern crate clap;

mod kd_tree;
mod pdf_util;
mod modtile;

use clap::{App, Arg};
use euclid::{Point2D,Box2D};
use image::{ GenericImage, GenericImageView, RgbImage,Rgb};
use image::{DynamicImage,ImageResult};

use std::path::Path;
use std::collections::HashMap;

use kd_tree::{construct_kd_tree,query_nearest_neighbor};

fn main() {
    let matches = App::new("Image Play")
        .version("0.1")
        .author("bitbangr <mgj000@hotmail.com>")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("config settings for tiling")
                .takes_value(true)
                .required(true),
        ).get_matches();

    // load all the config settings from JSON file
    let configs : modtile::Config = modtile::load_configs(matches.value_of("config").unwrap());

    println!("Sucessfully Loaded -> {:?}", configs);

    let tile_colors = configs.tile_colors;
    let input = configs.input;
    let output = configs.output;
    let output_width = configs.output_width;
    let output_height = configs.output_height;
    let aspect_x = configs.aspect_x;
    let aspect_y = configs.aspect_y;
    let tile_size_x = configs.tile_size_x;
    let tile_size_y = configs.tile_size_y;
    let tile_space_x = configs.tile_space_x;
    let tile_space_y = configs.tile_space_y;
    let tiles_per_pane_width =  configs.tiles_per_pane_width;
    let tiles_per_pane_height = configs.tiles_per_pane_height;

    // round to closest integer.
    // Less than .5 rounds down, More than .5 rounds up
    // so if less than half a tile it is left out
    // if more than half a tile it is included
    let output_width_tile_count : usize = (output_width/(tile_size_x )).round() as usize; // Should account for spacing of tiles
    let output_height_tile_count : usize = (output_height/(tile_size_y)).round() as usize;
    // let output_width_tile_count : usize = (output_width/(tile_size_x + tile_space_x)).round() as usize; // Should account for spacing of tiles
    // let output_height_tile_count : usize = (output_height/(tile_size_y + tile_space_y)).round() as usize;


    println!("tile size x:{}\ntile size y:{}", tile_size_x, tile_size_y);
    println!("output image width: {} , width tile count: {}\noutput image height: {} , height tile count: {}", output_width,
                                                                                                        output_width_tile_count,
                                                                                                        output_height,
                                                                                                        output_height_tile_count);

    // Grab the input image
    // TODO Get some proper error handling here or in function for missing image
    let input_img = get_image(input).unwrap();
    let (img_width, img_height) = input_img.dimensions();
    let input_img_width = img_width as f64;
    let input_img_height = img_height as f64;

    println!("input image width: {}\ninput image height: {}", &input_img_width,&input_img_height );

    // create the input image buffer for use later
    let input_image_buffer = &input_img.to_rgb8();

    // TODO - Now that we have all the info lets start processing!
    // divide input image into output_width_tile_count X output_height_tile_count boxes
    // Store these boxes as a vector of Window panes left to right, top to bottom
    // Each window pane consists of tiles - a vector of Box2D co-ords (TopLeft Corner, BottomRight Corner) again stored left to right top to bottom ordered
    // May add a box color to this as well

    let window: Vec<Vec<Box2D<i32,i32>>> = create_window_panes (input_img_width,
                                                            input_img_height,
                                                            output_width_tile_count,
                                                            output_height_tile_count,
                                                            tiles_per_pane_width,
                                                            tiles_per_pane_height);




    println!("************************");
    println!("***** Input Window *****");
    println!("************************\n");
    println!("number of tiles in first window pane: {:?}", &window[0].len());
    println!("number of window panes: {:?}", &window.len());
    println!("number of tiles in first window pane: {:?}", &window[0].len());
    println!("************************\n");
    println!("Tile Coords in each window pane");
    for (i, pane) in window.iter().enumerate() {
        println!("**** Window pane {} ****", i+1);
        for (j, tile_coords) in pane.iter().enumerate(){
            println!("Tile {} Coords: {:?}", j+1, tile_coords);
        }
    }

    // // this holds all the info necesary to build the output image
    // let output_window: Vec<Vec<Box2D<i32,i32>>> = create_window_panes (output_width,
    //                                                         output_height,
    //                                                         output_width_tile_count,
    //                                                         output_height_tile_count,
    //                                                         tiles_per_pane_width,
    //                                                         tiles_per_pane_height);

    // this holds all the info necesary to build the output image
    let output_window: Vec<Vec<(Box2D<i32,i32>,modtile::RGB)>> = create_out_panes (output_width,
                                                            output_height,
                                                            output_width_tile_count,
                                                            output_height_tile_count,
                                                            tiles_per_pane_width,
                                                            tiles_per_pane_height);

    // TESTING setting rgb values in 
    // this holds all the info necesary to build the output image
    let window: Vec<Vec<(Box2D<i32,i32>,modtile::RGB)>> = create_out_panes (output_width,
                                                            output_height,
                                                            output_width_tile_count,
                                                            output_height_tile_count,
                                                            tiles_per_pane_width,
                                                            tiles_per_pane_height);



    println!("************************");
    println!("***** Output Window ****");
    println!("************************\n");

    println!("Output - window panes: {:?}", &output_window.len());
    println!("Output - number of tiles in first window pane: {:?}", &window[0].len());
    println!("Output Tile Coords in each output window pane");
    println!("************************\n");
    for (i, pane) in output_window.iter().enumerate() {
        println!("**** Output Window pane {} ****", i+1);
        for (j, tile_coords) in pane.iter().enumerate(){
            println!("Output Tile {} Coords: {:?}", &j+1, &tile_coords.0);
            println!("Output Tile {} rgb: {:?}",&j+1, &tile_coords.1);

            // mgj todo try to update this RGB value to a new value
            // let &tile_coords.1 = modtile::RGB(10,10,10);
        }
    }
    println!("************************\n");
    println!("************************\n");

    // load the tile colors_path
    let all_colors: modtile::AllColors = modtile::load_all_colors(&tile_colors.to_string().to_owned());
    for (j, tile_color) in all_colors.colors.iter().enumerate() {
        println!("tile color:{}, {:?}",j+1, tile_color);
    }

    let mut color_vec: Vec<Vec<u8>> = build_color_vec(&all_colors);  // Create a Vector Array of elements of type u8

    // construct our KD tree with the desired color vec
    let kd_tree = construct_kd_tree(&mut color_vec[..], 3);

    // create the output image
    let mut out_img : DynamicImage = DynamicImage::new_rgb8(output_width as u32, output_height as u32);

//**********************
// start cut and paste from img_play
//*********************
    // keep count for number of times each color used as a tile
    let mut tile_color_count: HashMap<&Vec<u8>, i32> = HashMap::new();

    // save a window pane ordered list of output tile colours
    // top left hand corner is first tile.
    // Progesses from left to to right
    // The from top to bottom
    // let mut pane_colours : Vec<(u8,u8,u8)> = Vec::new();
    let mut window_pane_colors : Vec<Vec<(u8,u8,u8)>> = Vec::new();

    // mgj New TODO
    // interate through each window pane
    //     iterate though each tile in the pane
    //
    //     get the average color for tile
    //         build the output image with closest color match to the average color of that tile
    //         ___ or ___
    //         store that average color into output "window" struct.
    //
    //      Output window struct can then be used to
    //         create the output image
    //         create the output pdf instructions doc
    for (i, pane) in window.iter().enumerate() {
        let mut pane_colours : Vec<(u8,u8,u8)> = Vec::new();
        println!("**** Window pane {} ****", i+1);
        for (j, tile_coords) in pane.iter().enumerate(){
            println!("Tile {} Coords: {:?}", j+1, tile_coords);

            let avg_col = get_avg_col(&input_image_buffer, &tile_coords);

            // get the closes color match
            let color_tup = query_nearest_neighbor(&[avg_col[0],avg_col[1],avg_col[2]], &kd_tree, 3, kd_tree.root()).value();
            // println!("color_tup {:?}", &color_tup);

            // keep running count of each tile color used
            *tile_color_count.entry(color_tup).or_insert(0) += 1;

            let cm_red = color_tup[0] as u8;
            let cm_yellow = color_tup[1] as u8;
            let cm_green = color_tup[2] as u8;

            pane_colours.push((cm_red.clone(),cm_yellow.clone(),cm_green.clone()));

            let closest_match = Rgb([cm_red,cm_yellow,cm_green]);

            // store the closest match
            // iterate over the pixels and paint with closest match color
            for iy in tile_coords.min.y..=tile_coords.max.y{
                for ix in tile_coords.max.x..=tile_coords.max.x{
                    // need to figure out the to_rgba call as we don't use it
                    // println!( "bx.min.x: {} bx.min.y: {} ix: {} iy: {}", bx.min.x,bx.min.y, ix, iy );
                    out_img.put_pixel(ix as u32, iy as u32, image::Pixel::to_rgba(&closest_match));

                    // TODO mgj we want to save this color info into some structure that can be passed to build_output_pdf call
                    // println!( "bx.min.x: {} bx.min.y: {} ix: {} iy: {}", bx.min.x,bx.min.y, ix, iy );
                }
            }
        }
        window_pane_colors.push(pane_colours);
    }

    // Save the resulting image.  We'll also want to use this to create our ouptput PDF instructions doc
    let save_path = Path::new(&output);
    println!("save_path {:#?}", &save_path.to_str() );
    out_img.save(save_path).unwrap();

    // create a vector of output colors and sort it by usage count
    let mut tile_color_count_vec: Vec<(&&Vec<u8>, &i32)> = tile_color_count.iter().collect();
    tile_color_count_vec.sort_by(|a, b| b.1.cmp(a.1));

    // we want to print out detailed TileColor info (not just rgb value and count)
    println!("List of tiles used in this mosaic ordered by count of tiles") ;
    for bc in &tile_color_count_vec {
        let var_rgb : modtile::RGB = modtile::RGB(bc.0[0],bc.0[1],bc.0[2]);

        for tc in &all_colors.colors {
            if var_rgb == tc.rgb {
                println!("Count: {}, \t {:?}, ", bc.1,  tc );
            }
        };
     }

//*********************
// end cut and paste from img_play
//********************

    // println!("Window Pane Colors {:#?}", window_pane_colors);

    // see if we can zip window_pane_colors and window with transform function on window coords`.
    let it = window.iter().zip(window_pane_colors.iter());

    for (i, (x, y)) in it.enumerate() {
       println!("w {:#?}: (wpc {:?}, wpc{:?})", i, x, y);
    }

}

fn get_image(input: String) -> Result<DynamicImage,image::ImageError>{

    let image_path = Path::new(&input);
    Ok(image::open(image_path)?)
}



// this function will take an image, iterate over pixel dimensions defined by
// pixel_box.min and pixel_box.max and return the average color of all pixels contained
// using the mean squares method
// fn get_avg_col(img: &RgbImage, pixel_box :&Box2D<i32,i32>  ) -> String {
fn get_avg_col(img: &RgbImage, pixel_box :&Box2D<i32,i32>  ) -> Rgb <u8>{

    let mut count = 0.0;

    let x_start = pixel_box.min.x;
    let y_start = pixel_box.min.y;
    let x_end = pixel_box.max.x;
    let y_end = pixel_box.max.y;

    // sum of all squared RGB colors in nxn square
    let mut sq_sum_red = 0.0;
    let mut sq_sum_blu = 0.0;
    let mut sq_sum_gre = 0.0;

    for iy in y_start..=y_end{
        for ix in x_start..=x_end {
            let [r, g, b] = img.get_pixel(ix as u32, iy as u32).0;
            // println!("r: {} g: {} b:{}", r,g,b );
            sq_sum_red += r as f64 * r as f64 ;
            sq_sum_blu += b as f64 * b as f64 ;
            sq_sum_gre += g as f64 * g as f64 ;
            count += 1.0;
        }
    }
    let avg_red = (sq_sum_red/count).sqrt();
    let avg_gre = (sq_sum_gre/count).sqrt();
    let avg_blu = (sq_sum_blu/count).sqrt();

    // want to return an rgb tuple here instead of a string
    // let s = format!("[r,g,b]: [{},{},{}]", avg_red as u8, avg_gre as u8, avg_blu  as u8);
    let rgb_tup = Rgb([avg_red as u8, avg_gre as u8, avg_blu  as u8]);

    rgb_tup
}

fn create_out_panes(input_img_width: f64,
                    input_img_height: f64,
                    output_width_tile_count: usize,
                    output_height_tile_count: usize,
                    tiles_per_pane_width: usize,
                    tiles_per_pane_height: usize) -> Vec<Vec< (Box2D<i32, i32>,modtile::RGB) >> {

    println!("input_img_width: {:?}", input_img_width);
    println!("input_img_height: {:?}", input_img_height);
    println!("output_width_tile_count: {:?}", output_width_tile_count);
    println!("output_height_tile_count: {:?}", output_height_tile_count);
    println!("tiles_per_pane_width: {:?}", tiles_per_pane_width);
    println!("tiles_per_pane_height: {:?}", tiles_per_pane_height);

    let mut window_grid: Vec<Vec<(Box2D<i32, i32>,modtile::RGB)>> = Vec::new();

    let window_pane_rows = output_height_tile_count / tiles_per_pane_height;
    let window_pane_cols = output_width_tile_count / tiles_per_pane_width;

    println!("window_pane_rows: {:?}", &window_pane_rows);
    println!("window_pane_cols: {:?}", &window_pane_cols);

    // Cannot have fractional pixels so round and convert to usize
    let img_width_div  = (input_img_width / output_width_tile_count as f64).round() as usize;
    let img_height_div = (input_img_height / output_height_tile_count as f64).round() as usize;

    // ******** NOTE ********
    // TODO - mgj Output image size is MM and NOT PX so adjust window pane co-ords to account for this

    // Want to construct a series of Box2D coords for tiles in each of the window panes.
    // initialise our variables
    let mut tile_top_left_x;
    let mut tile_top_left_y;
    let mut tile_bot_right_x;
    let mut tile_bot_right_y;

    for pane_row in 0..window_pane_rows {
        // println!("pane_row: {}", &pane_row);
        for pane_col in 0..window_pane_cols{
                // println!("   pane_col: {}", &pane_col);
                let mut pane_grid: Vec<(Box2D<i32, i32>,modtile::RGB)> = Vec::new();
                for tile_row in 0..tiles_per_pane_height {
                    // println!("      tile_row: {}", &tile_row);
                    for tile_col in 0..tiles_per_pane_width{
                        // println!("         tile_col: {}", &tile_col);
                        tile_top_left_x = tile_col * img_width_div +  pane_col * tiles_per_pane_width * img_width_div;
                        tile_top_left_y = tile_row * img_height_div + pane_row * tiles_per_pane_height * img_height_div;
                        tile_bot_right_x = tile_top_left_x + img_width_div - 1;   // pixel dimensions are zero based so subtract 1
                        tile_bot_right_y = tile_top_left_y + img_height_div - 1 ; // pixel dimensions are zero based so subtract 1

                        // println!("\t\t tile_top_left_x: {}", & tile_top_left_x);
                        // println!("\t\t tile_top_left_y: {}", & tile_top_left_y);
                        // println!("\t\ttile_bot_right_x: {}", & tile_bot_right_x);
                        // println!("\t\ttile_bot_right_y: {}", & tile_bot_right_y);

                        let p_start : Point2D<i32,i32> = Point2D::new(tile_top_left_x as i32, tile_top_left_y as i32);
                        let p_end : Point2D<i32,i32> = Point2D::new(tile_bot_right_x as i32, tile_bot_right_y as i32);
                        let tile_box = Box2D { min: p_start, max: p_end };
                        let mut rgb = modtile::RGB(0,0,0);
                        pane_grid.push((tile_box,rgb));
                    }
                }
                window_grid.push(pane_grid);
        }
    }
    // return the grid
    window_grid
}




fn create_window_panes(input_img_width: f64,
                    input_img_height: f64,
                    output_width_tile_count: usize,
                    output_height_tile_count: usize,
                    tiles_per_pane_width: usize,
                    tiles_per_pane_height: usize) -> Vec<Vec<Box2D<i32, i32>>> {

    println!("input_img_width: {:?}", input_img_width);
    println!("input_img_height: {:?}", input_img_height);
    println!("output_width_tile_count: {:?}", output_width_tile_count);
    println!("output_height_tile_count: {:?}", output_height_tile_count);
    println!("tiles_per_pane_width: {:?}", tiles_per_pane_width);
    println!("tiles_per_pane_height: {:?}", tiles_per_pane_height);

    let mut window_grid: Vec<Vec<Box2D<i32, i32>>> = Vec::new();

    let window_pane_rows = output_height_tile_count / tiles_per_pane_height;
    let window_pane_cols = output_width_tile_count / tiles_per_pane_width;

    println!("window_pane_rows: {:?}", &window_pane_rows);
    println!("window_pane_cols: {:?}", &window_pane_cols);

    // Cannot have fractional pixels so round and convert to usize
    let img_width_div  = (input_img_width / output_width_tile_count as f64).round() as usize;
    let img_height_div = (input_img_height / output_height_tile_count as f64).round() as usize;

    // Want to construct a series of Box2D coords for tiles in each of the window panes.
    // initialise our variables
    let mut tile_top_left_x;
    let mut tile_top_left_y;
    let mut tile_bot_right_x;
    let mut tile_bot_right_y;

    for pane_row in 0..window_pane_rows {
        // println!("pane_row: {}", &pane_row);
        for pane_col in 0..window_pane_cols{
                // println!("   pane_col: {}", &pane_col);
                let mut pane_grid: Vec<Box2D<i32, i32>> = Vec::new();
                for tile_row in 0..tiles_per_pane_height {
                    // println!("      tile_row: {}", &tile_row);
                    for tile_col in 0..tiles_per_pane_width{
                        // println!("         tile_col: {}", &tile_col);
                        tile_top_left_x = tile_col * img_width_div +  pane_col * tiles_per_pane_width * img_width_div;
                        tile_top_left_y = tile_row * img_height_div + pane_row * tiles_per_pane_height * img_height_div;
                        tile_bot_right_x = tile_top_left_x + img_width_div - 1;   // pixel dimensions are zero based so subtract 1
                        tile_bot_right_y = tile_top_left_y + img_height_div - 1 ; // pixel dimensions are zero based so subtract 1

                        // println!("\t\t tile_top_left_x: {}", & tile_top_left_x);
                        // println!("\t\t tile_top_left_y: {}", & tile_top_left_y);
                        // println!("\t\ttile_bot_right_x: {}", & tile_bot_right_x);
                        // println!("\t\ttile_bot_right_y: {}", & tile_bot_right_y);

                        let p_start : Point2D<i32,i32> = Point2D::new(tile_top_left_x as i32, tile_top_left_y as i32);
                        let p_end : Point2D<i32,i32> = Point2D::new(tile_bot_right_x as i32, tile_bot_right_y as i32);
                        let tile_box = Box2D { min: p_start, max: p_end };
                        pane_grid.push(tile_box);
                    }
                }
                window_grid.push(pane_grid);
        }
    }
    // return the grid
    window_grid
}

// Construct a vector of rgb values from AllColors
fn build_color_vec(all_colors: &modtile::AllColors) -> Vec<Vec<u8>> {

    let mut color_vec: Vec<Vec<u8>> = Vec::new();  // Create a Vector Array of elements of type u8
    for tc in &all_colors.colors {
        let color: Vec<u8> = vec!(tc.rgb.0,tc.rgb.1,tc.rgb.2);
        color_vec.push(color);     // add the color to the Vector Array
        // println!("{:?}" , tc);
    };

    // return the color vec
    color_vec
}
