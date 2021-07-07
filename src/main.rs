extern crate clap;

mod kd_tree;
mod pdf_util;
mod modtile;

use clap::{App, Arg};
use euclid::{Point2D,Box2D};
use image::{GenericImage, GenericImageView, RgbImage,Rgb};
use image::DynamicImage;

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
        )
        .arg(
            Arg::with_name("swatch")
                .short("s")
                .long("swatch")
                .value_name("SWATCH")
                .help("Used to generate a color swatch pdf")
                .takes_value(false)
                .required(false),
        )
        .get_matches();

    // load all the config settings from JSON file
    let cfg : modtile::Config = modtile::load_configs(matches.value_of("config").unwrap());

    println!();
    println!("Successfully Loaded Config File -> {:?}", cfg);

    // Grab the input image
    // TODO Get some proper error handling here or in function for missing image
    let input_img = get_image(cfg.input).unwrap();
    let (img_width, img_height) = input_img.dimensions();
    let input_img_width = img_width as f64;
    let input_img_height = img_height as f64;

    println!();
    println!("input image width: {}\ninput image height: {}", &input_img_width,&input_img_height );

    // ********** todo May 28th currently does not handle all cases of bad aspect ratio
    // determine the largest output box dimensions that will maintain the input image aspect ratio.
    // and resize the output image accordingly
    // strange behaviour noted so use with care.  Best to make sure op is some close ratio to input
    let (output_width, output_height) =  get_max_box(input_img_width, input_img_height , cfg.output_width, cfg.output_height);
    println!();
    println!("output image width: {}\noutput image height: {}", &output_width,&output_height );

    // round to closest integer.
    // Less than .5 rounds down, More than .5 rounds up
    // so if less than half a tile it is left out
    // if more than half a tile it is included
    let output_width_tile_count : usize = (output_width/(cfg.tile_size_x )).round() as usize; // Should account for spacing of tiles
    let output_height_tile_count : usize = (output_height/(cfg.tile_size_y)).round() as usize;
    println!();
    println!("tile size x:{}\ntile size y:{}", cfg.tile_size_x, cfg.tile_size_y);
    println!();
    println!("output image width: {} , width tile count: {}\noutput image height: {} , height tile count: {}", output_width,
                                                                                                        output_width_tile_count,
                                                                                                        output_height,
                                                                                                        output_height_tile_count);

    // create the input image buffer for use later
    let input_image_buffer = &input_img.to_rgb8();

    // TODO - Now that we have all the info lets start processing!
    // divide input image into output_width_tile_count X output_height_tile_count boxes
    // ***
    // *** mgj TODO need to figure out what to do if aspect ratio of input image is not the same as output
    // ***
    // Store these boxes as a vector of Window panes left to right, top to bottom
    // Each window pane consists of tiles - a vector of Box2D co-ords (TopLeft Corner, BottomRight Corner) again stored left to right top to bottom ordered
    //                                    - and and RGB for storing the color of the tile. Defaults to black for newly created window
    //
    let mut input_window: Vec<Vec<(Box2D<i32,i32>,modtile::RGB)>> = create_out_panes (input_img_width,
                                                            input_img_height,
                                                            output_width_tile_count,
                                                            output_height_tile_count,
                                                            cfg.tiles_per_pane_width,
                                                            cfg.tiles_per_pane_height);


    // println!("************************");
    // println!("***** Input Window *****");
    // println!("************************\n");
    // println!("number of tiles in first window pane: {:?}", &input_window[0].len());
    // println!("number of window panes: {:?}", &input_window.len());
    // println!("number of tiles in first window pane: {:?}", &input_window[0].len());
    // println!("************************\n");
    // println!("Tile Coords in each window pane");
    // for (i, pane) in input_window.iter().enumerate() {
    //     println!("**** Window pane {} ****", i+1);
    //     for (j, tile_coords) in pane.iter().enumerate(){
    //         println!("Tile {} Coords: {:?}", j+1, tile_coords);
    //     }
    // }

    // this holds all the info necesary to build the output image
    let mut output_window: Vec<Vec<(Box2D<i32,i32>,modtile::RGB)>> = create_out_panes (output_width,
                                                            output_height,
                                                            output_width_tile_count,
                                                            output_height_tile_count,
                                                            cfg.tiles_per_pane_width,
                                                            cfg.tiles_per_pane_height);

    // println!("************************");
    // println!("***** Output Window ****");
    // println!("************************\n");
    //
    // println!("Output - window panes: {:?}", &output_window.len());
    // println!("Output - number of tiles in first window pane: {:?}", &output_window[0].len());
    // println!("Output Tile Coords in each output window pane");
    // println!("************************\n");
    // for (i, pane) in output_window.iter().enumerate() {
    //     println!("**** Output Window pane {} ****", i+1);
    //     for (j, tile_coords) in pane.iter().enumerate(){
    //         println!("Output Tile {} Coords: {:?}", &j+1, &tile_coords.0);
    //         println!("Output Tile {} rgb: {:?}",&j+1, &tile_coords.1);
    //     }
    // }
    // println!("************************\n");
    // println!("************************\n");

    // load the tile colors_path
    let all_colors: modtile::AllColors = modtile::load_all_colors(&cfg.tile_colors.to_string().to_owned());
    // for (j, tile_color) in all_colors.colors.iter().enumerate() {
    //     println!("tile color:{}, {:?}",j+1, tile_color);
    // }

    // if swatch flag present on command line then generate color swatch file
    if matches.is_present("swatch"){
        // generate a color swatch file
            match pdf_util::generate_color_swatch(&all_colors){
                Err(v) => panic!(
                    "Could not create color swatch file: {}",
                    v.to_string()
                ),
                Ok(r) => println!("generate_color_swatch() Success {:?}",r),              // return a bufReader
            };
    }

    let mut color_vec: Vec<Vec<u8>> = build_color_vec(&all_colors);  // Create a Vector Array of elements of type u8

    // construct our KD tree with the desired color vec
    let kd_tree = construct_kd_tree(&mut color_vec[..], 3);

    // keep count for number of times each color used as a tile
    // let mut tile_color_count: HashMap<&Vec<u8>, i32> = HashMap::new();
    let mut tile_color_count: HashMap<Vec<u8>, i32> = HashMap::new();

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
    for (_i, pane) in input_window.iter_mut().enumerate() {
        let mut pane_colours : Vec<(u8,u8,u8)> = Vec::new();
        // println!("**** Window pane {} ****", i+1);
        for (_j, mut tile) in pane.iter_mut().enumerate(){
            // println!("Tile {}: {:?}", j+1, tile);

            let avg_col = get_avg_col(&input_image_buffer, &tile.0);

            // get the closes color match
            let color_tup :Vec<u8> = query_nearest_neighbor(&[avg_col[0],avg_col[1],avg_col[2]], &kd_tree, 3, kd_tree.root()).value().clone();
            // println!("color_tup {:?}", &color_tup);

            // keep running count of each tile color used
            *tile_color_count.entry(color_tup.clone()).or_insert(0) += 1;

            let cm_red = color_tup[0] as u8;
            let cm_yellow = color_tup[1] as u8;
            let cm_green = color_tup[2] as u8;

            pane_colours.push((cm_red.clone(),cm_yellow.clone(),cm_green.clone()));

            // let closest_match = Rgb([cm_red,cm_yellow,cm_green]);

            // store the closest match in the tile
            tile.1 = modtile::RGB(cm_red.clone(),cm_yellow.clone(),cm_green.clone());
            // println!("Closest Match ---> {:?} \n", tile);

        }
        window_pane_colors.push(pane_colours);
    }


    // see if we can zip window_pane_colors and window with transform function on window coords`.
    // let it = window.iter().zip(window_pane_colors.iter());
    //
    // for (i, (x, y)) in it.enumerate() {
    //    println!("w {:#?}: (wpc {:?}, wpc{:?})", i, x, y);
    // }

    // zip input_window and output_window and copy input rgb value to output
    let wit = input_window.iter().zip(output_window.iter_mut());
    for (_i, (ip,op)) in wit.enumerate() {
       // println!("Pane {:?}: \ninput: {:?} \noutput: {:?})", i, ip,op);
       // println!{"\n"};

       let pit = ip.iter().zip(op.iter_mut());
       for (_j, (itp,otp)) in pit.enumerate() {
          // println!("Tile {:?}: \ninput: {:?} \noutput: {:?})", j, itp,otp);
          // println!{"\n"};

          // set the output tile color to be the same as the imput tile color
          otp.1 = itp.1;
      }
    }
    // println!("******/n *** Output Window a fter Zipped iterator ***\n******/n");
    // println!("output window {:?}", &output_window);

    // create the output image
    let out_img : DynamicImage = create_output_image(&output_window, output_width, output_height);

    // Save the resulting image.  We'll also want to use this to create our ouptput PDF instructions doc
    // Add proper error handling for image
    let save_path = Path::new(&cfg.output);
    println!("save_path {:#?}", &save_path.to_str() );
    out_img.save(save_path).unwrap();

    // create a vector of output colors and sort it by usage count
    let mut tile_color_count_vec: Vec<(Vec<u8>, i32)> = tile_color_count.into_iter().collect();
    tile_color_count_vec.sort_by(|a, b| b.1.cmp(&a.1));

    // we want to print out detailed TileColor info (not just rgb value and count)
    println!();
    println!("List of tiles used in this mosaic ordered by count of tiles") ;
    for bc in &tile_color_count_vec {
        let var_rgb : modtile::RGB = modtile::RGB(bc.0[0],bc.0[1],bc.0[2]);

        for tc in &all_colors.colors {
            if var_rgb == tc.rgb {
                println!("Count: {}, \t {:?}, ", bc.1,  tc );
            }
        };
     }

    // println!("Window Pane Colors {:#?}", window_pane_colors);

    // Create the output instructions doc
    // pdf_util::build_output_pdf(&save_path,&all_colors,&tile_color_count_vec,&output_window);
    // Changed from output window to input window to simplify PDF to image space cooridinates translation
    pdf_util::build_output_pdf(&save_path,&all_colors,tile_color_count_vec,&input_window);

} // end main

// create the output image
fn create_output_image(output_window: &Vec<Vec<(Box2D<i32, i32>, modtile::RGB)>>, output_width: f64, output_height: f64) -> DynamicImage {

    let mut out_img : DynamicImage = DynamicImage::new_rgb8(output_width as u32, output_height as u32);
    for (_i, pane) in output_window.iter().enumerate() {
        for (_j, tile) in pane.iter().enumerate(){

            let cm_r = tile.1.0 as u8;
            let cm_y = tile.1.1 as u8;
            let cm_g = tile.1.2 as u8;
            let tile_color = Rgb([cm_r,cm_y,cm_g]);

            // iterate over the pixels and paint with tile rgb value
            // look into using Box2D x_range and y_range
            for iy in tile.0.min.y..=tile.0.max.y{
                for ix in tile.0.min.x..=tile.0.max.x{
                    out_img.put_pixel(ix as u32, iy as u32, image::Pixel::to_rgba(&tile_color));
                }
            }
        }
    }
    out_img
}

fn get_image(input: String) -> Result<DynamicImage,image::ImageError>{

    let image_path = Path::new(&input);
    Ok(image::open(image_path)?)
}

// this function will take an image, iterate over pixel dimensions defined by
// pixel_box.min and pixel_box.max and return the average color of all pixels contained
// using the mean squares method
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

    println!();
    println!("input_img_width: {:?}", input_img_width);
    println!("input_img_height: {:?}", input_img_height);
    println!();
    println!("output_width_tile_count: {:?}", output_width_tile_count);
    println!("output_height_tile_count: {:?}", output_height_tile_count);
    println!();
    println!("tiles_per_pane_width: {:?}", tiles_per_pane_width);
    println!("tiles_per_pane_height: {:?}", tiles_per_pane_height);

    let mut window_grid: Vec<Vec<(Box2D<i32, i32>,modtile::RGB)>> = Vec::new();

    let window_pane_rows = output_height_tile_count / tiles_per_pane_height;
    let window_pane_cols = output_width_tile_count / tiles_per_pane_width;
    println!();
    println!("window_pane_rows: {:?}", &window_pane_rows);
    println!("window_pane_cols: {:?}", &window_pane_cols);

    // Cannot have fractional pixels so round and convert to usize
    // TODO mgj add some more error checking
    //    i.e. if output hieght tile count is 3 and tiles per pane hieght is 4
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
                        let rgb = modtile::RGB(0,0,0);
                        pane_grid.push((tile_box,rgb));
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


// return maximum possible output dimensions (width height) for input box of a given size
// and desired output dimensions while maintaining aspect ratio of the input box
// All dimensions must be greater than 1.0 or function panics
// This function also does not handle the case where the input box has bigger dimensions that the output box
fn get_max_box(ip_width: f64, ip_height: f64, op_width: f64, op_height: f64) -> (f64,f64) {

    // we need to do this check as RUST will happily divide by float zero
    if ip_width < 1.0 || ip_height < 1.0 || op_width < 1.0 || op_height < 1.0 {
        panic!("get_max_box() - all supplied dimensions must be greater than 1.0");
    }

    // println!("get max box {} {} {} {}", ip_width, ip_height,op_width, op_height);

    if op_width/ip_width > op_height/ip_height {
        let trans_height = &ip_height * (&op_height/&ip_height);
        let trans_width = &ip_width * (op_height/ip_height);

        // println!("1 op_width/ip_width > op_height/ip_height");
        // println!("1 trans_width: {:?}, trans_height: {:?}", &trans_width,&trans_height);
        (trans_width,trans_height)

    } else if op_width/ip_width < op_height/ip_height {
        let trans_height = &ip_height * (op_width/ip_width);
        let trans_width = &ip_width * (op_width/ip_width);

        // println!("2 op_width/ip_width < op_height/ip_height");
        // println!("2 trans_width: {:?}, trans_height: {:?}", &trans_width,&trans_height);
        (trans_width,trans_height)
    } else
    {
        let trans_height = op_height;
        let trans_width = op_width;
        // println!("3 op_width/ip_width = op_height/ip_height");
        // println!("3 trans_width: {:?}, trans_height: {:?}", trans_width,trans_height);
        (trans_width,trans_height)
    }

}
