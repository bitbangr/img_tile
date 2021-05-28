use printpdf::*;
use std::fs::File;
use std::ops::Range;
use std::io::BufWriter;
use regex::Regex;
use std::option::Option::Some;use crate::modtile;

pub(crate) fn generate_color_swatch(all_colors: &crate::modtile::AllColors) -> Result<(), String> {

    let x_mm = 215.9;
    let y_mm = 279.4;
    let (doc, page1, layer1) =
        PdfDocument::new(&all_colors.name.to_owned(), Mm(x_mm), Mm(y_mm), "Layer 1");

    let(page2, layer2) = doc.add_page(Mm(x_mm), Mm(y_mm), "Layer 2");
    let(page3, layer3) = doc.add_page(Mm(x_mm), Mm(y_mm), "Layer 3");

    let layer1 = doc.get_page(page1).get_layer(layer1);
    let layer2 = doc.get_page(page2).get_layer(layer2);
    let layer3 = doc.get_page(page3).get_layer(layer3);

    let swatch_name = &all_colors.name;
    let swatch_url= &all_colors.url;
    let swatch_desc = &all_colors.description;

    // println!("swatch desc {:?}", swatch_desc);
    // let swatch_desc = "Artist's quality colours made with permanent light fast pigments in 100% acrylic resin,\n made by hand in Vancouver, Canada since 1970. No fillers or extenders are added.";

    let font1 = doc
        .add_external_font(File::open("/System/Library/Fonts/NewYork.ttf").unwrap())
        .unwrap();

    // text, font size, x from left edge, y from bottom edge, font
    layer1.use_text(swatch_name, 14.0, Mm(40.0), Mm(260.0), &font1);
    layer1.use_text(swatch_url, 11.0, Mm(40.0), Mm(254.0), &font1);
    // layer1.use_text(swatch_desc, 11.0, Mm(40.0), Mm(248.0), &font1);

    layer1.begin_text_section();
        // setup the general fonts.
        // see the docs for these functions for details
        layer1.set_font(&font1, 10.0);
        layer1.set_text_cursor(Mm(40.0), Mm(250.0));
        layer1.set_line_height(10.0);

        for line in swatch_desc.lines() {
            layer1.write_text(line, &font1);
            layer1.add_line_break(); // <---
        }
    layer1.end_text_section();  // <- important!

    layer2.use_text(swatch_name, 14.0, Mm(40.0), Mm(260.0), &font1);
    layer3.use_text(swatch_name, 14.0, Mm(40.0), Mm(260.0), &font1);

    let step = 4;  // This value is the number of columns (4) to be displayed per line in PDF doc
    let len = &all_colors.colors.len();
    println!("Total number of colour swatches: {}", &len);

    let mut row :i32 = 0;
    // loop over the colors array in steps of step size
    // for each slice create a row of color swatch squares
    for st in (0..*len).step_by(step) {
        // println!("Step Start {} End {}", st, st+step);

        let mut e = st+step; // end of range
        if (e) >= *len {     // if end is past the limit of elements and adjust end to len
            e = *len;
            // println!("end of array so trim last slice to {}" , &e);
        }

        let rnge = Range { start: st , end: e };  // range for current slice
        let slice = &all_colors.colors[rnge];     // grab the slice of TileColors

        // todo remove magic nos below.
        // draw swatches to pdf layer/page.
        // Currrently Each layer/page can accomodate 5 rows of 4 swatches
        if row <= 4 {
                draw_layer1_swatches(&layer1, slice, &font1, row);
        } else if row > 4 && row <= 9 {
                draw_layer1_swatches(&layer2, slice, &font1, row - 5 );
        } else {
                draw_layer1_swatches(&layer3, slice, &font1, row - 10 );
        }

        // increment row count to display next row
        row += 1;
    }

    // create output file name from name field hyphenated
    let tile_name =  &all_colors.name.to_owned();

    // remove all the spaces and replace with underscores
    let re = Regex::new(r"([\s])").unwrap();
    let output = re.replace_all(tile_name, "_");
    let fileout = format!("./images/output/{}.pdf", output);
    println!("Output instructions found in {}", &fileout);

    doc.save(&mut BufWriter::new(
        File::create(fileout).unwrap(),
    )).unwrap();

    // mgj todo some proper error handling
    Ok(())
    // Err("something broke".to_owned())
}

// draw swatches to pdf layer/page.
// Currrently Each layer/page can accomodate 5 rows of 4 swatches
fn draw_layer1_swatches(layer1: &PdfLayerReference,
                                slice: &[crate::modtile::TileColor],
                                font: &IndirectFontRef, row: i32) -> () {

    // println!("draw_layer1_swatches - cur_row {}", row);

    // Todo Do we make these global vars? Constants?
    let swatch_font_size: f64 = 11.0; // size 10.0 might be ok
    let x_space = 10.0;  // horizontal spacing between color swatches in MM
    let y_space = 7.0; // vertical spacing between color swatches in MM
    let square_size: f64 = 30.0;
    let x_start_pos: f64 = 30.0;
    let y_font_line_height: f64 = 4.0;
    let y_first_row_pos: f64 = 214.0;
    let y_start_pos: f64 = y_first_row_pos - row as f64 * (square_size + 3.0 * y_font_line_height + y_space );

    for column in 1..= slice.len() {
        // draw a swatch box with the color
        // Convert Mm into Pt function.
        let size_x_rect : Pt = Mm(square_size).into();
        let size_y_rect : Pt = Mm(square_size).into();
        let offset_x_rect : Pt = Mm((x_start_pos + x_space) * column as f64).into();
        let offset_y_rect : Pt = Mm(y_start_pos).into();

        let line = Line {
            points: get_points_for_rect(size_x_rect, size_y_rect, offset_x_rect, offset_y_rect),
            is_closed: true,
            has_fill: true,
            has_stroke: false,
            is_clipping_path: false,
        };

        // Grab the next color tile
        let tc : &crate::modtile::TileColor = &slice[column-1];

        // Set the fill color here to current TileColor
        let fill_color = Color::Rgb(Rgb::new(*&tc.rgb.0 as f64/255.0, *&tc.rgb.1 as f64/255.0, *&tc.rgb.2 as f64/255.0, None));
        layer1.set_fill_color(fill_color);
        layer1.add_shape(line);

        // TileColor info as black text below box shape
        let fill_color_black = Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None));
        layer1.set_fill_color(fill_color_black);

        let x_in_mm: Mm = Mm((x_start_pos + x_space) * column as f64);
        let y_in_mm: Mm = Mm(y_start_pos - 1.0 * y_font_line_height);     // First line

        let r_rgb = String::from (&tc.rgb.0.to_string().to_owned());
        let g_rgb = String::from (&tc.rgb.1.to_string().to_owned());
        let b_rgb = String::from (&tc.rgb.2.to_string().to_owned());

        let rgb_str = format!("rgb ({}, {}, {})", r_rgb, g_rgb, b_rgb);
        layer1.use_text(rgb_str, swatch_font_size, x_in_mm, y_in_mm, font);

        let y_in_mm: Mm = Mm(y_start_pos - 2.0 * y_font_line_height);     // Second line
        layer1.use_text(&tc.name, swatch_font_size, x_in_mm, y_in_mm, font);

        let y_in_mm: Mm = Mm(y_start_pos - 3.0 * y_font_line_height);     // Third line
        layer1.use_text(&tc.number, swatch_font_size, x_in_mm, y_in_mm, font);
    }

}

// Calculate and return the points for a rectangle, given a horizontal and vertical offset,
// and an offset into the page from the lower left corner.
pub fn get_points_for_rect<P: Into<Pt>>(
    size_x: P,
    size_y: P,
    offset_x: P,
    offset_y: P,
) -> Vec<(Point, bool)> {
    let (size_x, size_y, offset_x, offset_y) = (
        size_x.into(),
        size_y.into(),
        offset_x.into(),
        offset_y.into(),
    );

    let top = Pt(offset_y.0 + size_y.0);
    let bottom = Pt(offset_y.0);
    let left = Pt(offset_x.0);
    let right = Pt(offset_x.0 + size_x.0);

    let top_left_pt = Point { x: left, y: top };
    let top_right_pt = Point { x: right, y: top };
    let bottom_right_pt = Point { x: right, y: bottom };
    let bottom_left_pt = Point { x: left, y: bottom };

    vec![
        (top_left_pt, false),
        (top_right_pt, false),
        (bottom_right_pt, false),
        (bottom_left_pt, false),
    ]
}

// Create the output document containing all the info necessary to construct the mosaic
// based off Created by the LEGO Art Mosaics shiny app. See https://github.com/joachim−gassen/legoartmosaic for more.
//  1. Create output Swatch for tiles used?
//  2. output image at 50% opacity overlayed with a grid showing tile color.
//  3. Draw Grid and number each grouping of nXn tiles
//  4. Create a page for each Super grid with Tile color and number
// Layout details for each Tile Grouping

// Construct all elements of output PDF
// This method does not embed image into output docs
pub(crate) fn build_output_pdf(save_path: &&std::path::Path,
                                mosaic_colours: Vec<(u8, u8, u8)>,
                                all_colors: &crate::modtile::AllColors,
                                tile_color_count_vec: &Vec<(&&Vec<u8>, &i32)>,
                                cb: &Vec<euclid::Box2D<i32, i32>>) -> Result<(), String> {

    let doc_width_mm = 279.4;
    let doc_height_mm = 215.9;
    let (doc, page1, layer1) =
        PdfDocument::new(&all_colors.name.to_owned(), Mm(doc_width_mm), Mm(doc_height_mm), "Layer 1");

    let font1 = doc
        .add_external_font(File::open("/System/Library/Fonts/NewYork.ttf").unwrap())
        .unwrap();

    // Construct image instead of embedding as this will allow precise placement of grid and circles
    // write image to pdf file.
    let current_layer = doc.get_page(page1).get_layer(layer1);
    // let image3 = Image::from_dynamic_image(&out_img);
    // image3.add_to_layer(current_layer.clone(), Some(Mm(30.0)), Some(Mm(30.0)), None, Some(3.5), Some(3.5), None);

    let fill_color = Color::Cmyk(Cmyk::new(0.0, 0.23, 0.0, 0.0, None));
    let outline_color = Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None));
    let mut dash_pattern = LineDashPattern::default();
    dash_pattern.dash_1 = Some(20);

    current_layer.set_fill_color(fill_color);
    current_layer.set_outline_color(outline_color);
    current_layer.set_outline_thickness(2.0);


    // draw grid on current layer
    construct_square_grid(&current_layer, doc_width_mm,doc_height_mm , font1 , &mosaic_colours);

    let major_grid_count = 12;


    // fn example(width: usize, height: usize) {
    //     // Base 1d array
    //     let mut grid_raw = vec![0; width * height];
    //
    //     // Vector of 'width' elements slices
    //     let mut grid_base: Vec<_> = grid_raw.as_mut_slice().chunks_mut(width).collect();
    //
    //     // Final 2d array `&mut [&mut [_]]`
    //     let grid = grid_base.as_mut_slice();
    //
    //     // Accessing data
    //     grid[0][0] = 4;
    // }


    let width: usize = 9;
    let height: usize = 9;
    // Base 1d array is mosaic_colours

    // TODO mgj May 23 . Want to collect slices of colours for each major quandrant to make the pdf construction page for that quadrant
    // Vector of 'width' elements slices
    // let grid_base: Vec<(u8, u8, u8)> = mosaic_colours.as_slice().chunks(width).collect();
    let mosaic_slice: Vec<_> = mosaic_colours.as_slice().chunks(width).collect();
    // println!("mc_slice {:?}", mc_slice);

    // perhaps time to consider using NDArray so that we can use slices?

    // // Final 2d array `&mut [&mut [_]]`
    // Construct 2D array
    let grid = mosaic_slice.as_slice();
    for row in 0..=8{
        for col in 0..=8 {
            println!("row:{} col:{} grid {:?}", row, col, grid[row][col]);
        }
        println!("Row");
    }

    // println!("grid {:?}", grid[0][1]);
    // println!("grid {:?}", grid[0][2]);
    // println!("grid {:?}", grid[0][3]);
    // println!("grid {:?}", grid[0][4]);
    // println!("grid {:?}", grid[0][5]);
    // println!("grid {:?}", grid[0][6]);
    // println!("grid {:?}", grid[0][7]);
    // println!("grid {:?}", grid[0][8]);
    // println!("test1");
    // println!("grid {:?}", grid[1][0]);
    // println!("grid {:?}", grid[1][1]);
    // println!("grid {:?}", grid[1][2]);
    // println!("grid {:?}", grid[1][3]);
    // println!("grid {:?}", grid[1][4]);
    // println!("grid {:?}", grid[1][5]);
    // println!("grid {:?}", grid[1][6]);
    // println!("grid {:?}", grid[1][7]);
    // println!("grid {:?}", grid[1][8]);
    // println!("test2");
    // println!("grid {:?}", grid[2][0]);
    // println!("grid {:?}", grid[2][1]);
    // println!("grid {:?}", grid[2][2]);
    // println!("grid {:?}", grid[2][3]);
    // println!("grid {:?}", grid[2][4]);
    // println!("grid {:?}", grid[2][5]);
    // println!("grid {:?}", grid[2][6]);
    // println!("grid {:?}", grid[2][7]);
    // println!("grid {:?}", grid[2][8]);
    // println!("test3");
    // println!("grid {:?}", grid[3][0]);
    // println!("grid {:?}", grid[3][1]);
    // println!("grid {:?}", grid[3][2]);
    // println!("grid {:?}", grid[3][3]);
    // println!("grid {:?}", grid[3][4]);
    // println!("grid {:?}", grid[3][5]);
    // println!("grid {:?}", grid[3][6]);
    // println!("grid {:?}", grid[3][7]);
    // println!("grid {:?}", grid[3][8]);
    // println!("test4");
    // println!("grid {:?}", grid[3][0]);
    // println!("grid {:?}", grid[3][1]);
    // println!("grid {:?}", grid[3][2]);
    // println!("grid {:?}", grid[3][3]);
    // println!("grid {:?}", grid[3][4]);
    // println!("grid {:?}", grid[3][5]);
    // println!("grid {:?}", grid[3][6]);
    // println!("grid {:?}", grid[3][7]);
    // println!("grid {:?}", grid[3][8]);
    // println!("test4");



    //
    //
    // // Final 2d array `&mut [&mut [_]]`
    // let grid = grid_base.as_mut_slice();

    let row :usize = 0;
    let col :usize = 0;

    // Accessing data
    // println!("Color at row: {} col:{} {:?}", row, col, grid[row][col]);

    // for each major grid
    // get corresponding slice from mosaic_colours
    // build grid and circles for this page
    // for count in 1..major_grid_count {
    //         build_page(&current_layer, font1 , mosaic_colour_slice);
    // }


    // save build instructions to same output file name but with pdf extension
    let fileout = save_path.with_extension("pdf");
    doc.save(&mut BufWriter::new(
        File::create(fileout).unwrap(),
    )).unwrap();

    // mgj todo some proper error handling
    Ok(())
    // Err("something broke".to_owned())

}

// Draw a main grid shape to match output photo
// Need to do grid math - for example
//   sample input image size 553x553 px,
//   We want Grid Major=3 and Grid Minor=4 gives 12 tiles vertically and 12 tiles horizontally
// So 553/12 => gives 46.08 pixels per division.
// Output image is resized to closest integer match so div gets rounded to 46*12 for output image size of 552x552 px
//
// Tile dimension = 46x46px.
//
// output image is 12 tiles x 12 tiles or 144 tiles in total

// create new method method that handles different major row and major column count
// i.e handle none-square grid
// need to handle none square div i.e. grid_div_x, grid_div_y

// new grid for fully constructed output pdf
fn construct_square_grid(current_layer: &PdfLayerReference,
                         doc_width_mm: f64,
                         doc_height_mm: f64,
                         grid_font: IndirectFontRef,
                         mosaic_colours: &Vec<(u8, u8, u8)>) -> () {

    let grid_major= 4;
    let grid_minor = 9;

    let page_margin_ver_mm = 20.0; // size of top bottom margin
    let page_margin_hor_mm = 20.0;  // size of left right margin

    // let img_width = grid_div * grid_major * grid_minor;  // 640x640 - remember that we are rounding up
    // let grid_div = 2.55; // 2.55 matches height and width of image
    let grid_div = (doc_height_mm - (2.0 * page_margin_ver_mm)) / grid_major as f64 / grid_minor as f64;

    let grid_origin_x :f64 = page_margin_hor_mm;  // Origin point (lower left corner of grid)
    let grid_origin_y :f64 = page_margin_ver_mm;

    // draw a simple quarter arc at (0,0)
    draw_quarter_arc(&current_layer);

    let outline_color = Color::Rgb(Rgb::new(0.5, 0.5, 0.5, None)); // gray
    current_layer.set_outline_color(outline_color);
    current_layer.set_outline_thickness(1.5);

    let mut mosaic_itr = mosaic_colours.into_iter();

    // for bc in &mosaic_colours {
    //     let var_rgb = bc.0; }

    // draw the circles first
    let radius = grid_div / 2.0 - 0.30;
    let div_count = grid_major * grid_minor;
    for row in (0..div_count).rev() {
        for col in 0..div_count{
            // set the fill colour to current tile as determined by row/col
            // random fill
            // TODO here mgj how do the colors get stored?
            let tile_color = mosaic_itr.next();
            // println!("mosaic_itr next {:?}",tile_color );

            let red = tile_color.unwrap().0 as f64;
            let green = tile_color.unwrap().1 as f64;
            let blue = tile_color.unwrap().2 as f64;

            // println!("mosaic_itr next red {:?}",red );

            let fill_color = Color::Rgb(Rgb::new(red/255.0, green/255.0,blue/255.0, None));
            current_layer.set_fill_color(fill_color);

            draw_circle(&current_layer,
                        grid_origin_x + grid_div/2.0 + grid_div * col as f64,
                        grid_origin_y + grid_div/2.0 + grid_div * row as f64,
                        radius);
        }
    }

    let outline_color = Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)); // black
    current_layer.set_outline_color(outline_color);
    current_layer.set_outline_thickness(2.0);

    // for each major grid column draw a vertical line
    for column in 0..=grid_major // 0,1,2,3 , zero based
    {
        // Convert Mm into Pt function.
        let start_x : Pt = Mm(grid_origin_x + column as f64 * grid_minor as f64 * grid_div as f64).into();
        let start_y : Pt = Mm(grid_origin_y).into();
        let end_x : Pt = start_x.clone();  // drawing a vertical line so x remains the same
        let end_y : Pt = Mm(grid_origin_y + grid_major as f64 * grid_minor as f64 * grid_div as f64).into();

        let line = Line {
            points: get_points_for_line(start_x, start_y, end_x, end_y),
            is_closed: false,
            has_fill: false,
            has_stroke: true,
            is_clipping_path: false,
        };
        current_layer.add_shape(line);
    }

    // for each major grid row draw a horizontal line
    for row in 0..=grid_major // 0,1,2,3 , zero based
    {
        // Convert Mm into Pt function.
        let start_x : Pt = Mm(grid_origin_x).into();
        let start_y : Pt = Mm(grid_origin_y + row as f64 * grid_minor as f64 * grid_div as f64).into();
        let end_x : Pt = Mm(grid_origin_x + grid_major as f64 * grid_minor as f64 * grid_div as f64).into();
        let end_y : Pt = start_y.clone();  // drawing a horizontal line so y remains the same

        let line = Line {
            points: get_points_for_line(start_x, start_y, end_x, end_y),
            is_closed: false,
            has_fill: false,
            has_stroke: true,
            is_clipping_path: false,
        };
        current_layer.add_shape(line);
    }

    let outline_color = Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)); // black
    let fill_color = Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)); // black
    current_layer.set_outline_color(outline_color);
    current_layer.set_outline_thickness(2.0);

    current_layer.set_fill_color(fill_color);

    // Write out the major grid numbers
    let text_loc: Vec<(f64,f64,String)> = get_grid_text_loc( grid_origin_x, grid_origin_y, grid_major, grid_minor,  grid_div);
    for item in text_loc {
        // println!("text location {:?}", item);
        current_layer.use_text(item.2, 60.0, Mm(item.0), Mm(item.1), &grid_font);
    }

}


// Return location cooridinates and string to display for major grid
fn get_grid_text_loc(grid_origin_x: f64, grid_origin_y: f64, grid_major: i32, grid_minor: i32, grid_div: f64) -> Vec<(f64, f64, String)> {

    let mut res: Vec<(f64,f64,String)> = Vec::new();
    let mut grid_no = 1;
    for row in (0..grid_major).rev() {
        let loc_y = grid_origin_y - grid_div / 2.0 + grid_minor as f64 / 2.0 * grid_div + row as f64 * grid_div*grid_minor as f64;

            for col in 0..grid_major{
                let loc_x = grid_origin_x - grid_div / 2.0 + grid_minor as f64 / 2.0 * grid_div + col as f64 * grid_div*grid_minor as f64;
                res.push((loc_x,loc_y,grid_no.to_string().to_owned()));
                grid_no+=1;
            }
    }
    res
}

fn draw_circle(current_layer: &&PdfLayerReference, x: f64, y: f64, radius: f64) -> () {

    // let xoff = location.x.0;
    // let yoff = location.y.0;
    let circle_points = get_points_for_circle(x,y,radius);
    let circle1 = Line {
       points: circle_points,
       is_closed: true,
       has_fill: true,
       has_stroke: true,
       is_clipping_path: false,
    };

    // Draw the circle
    current_layer.add_shape(circle1);
}

// formula for creating a bezier arc
// via https://spencermortensen.com/articles/bezier-circle/
// A good cubic Bézier approximation to a circular arc is:
// P0​=(0,1), P1​=(c,1), P2​=(1,c), P3​=(1,0) with c=0.551915024494
// This yields an arc on the unit circle centered about the origin,
// starting at P0​ and ending at P3, with the least amount of radial drift.

fn draw_quarter_arc(current_layer: &&PdfLayerReference) -> () {

    // Quadratic shape. The "false" determines if the next (following)
    // point is a bezier handle (for curves)
    // If you want holes, simply reorder the winding of the points to be
    // counterclockwise instead of clockwise.

    // bezier constant for arc that will fit a circle
    let c= 0.551915024494;

    // simple 10mm arc
    let points2 = vec![(Point::new(Mm(0.0), Mm(10.0)), true),
                   (Point::new(Mm(c*10.0), Mm(10.0)), true),
                   (Point::new(Mm(10.0), Mm(c*10.0)), false),
                   (Point::new(Mm(10.0), Mm(0.0)), false)];

    // Is the shape stroked? Is the shape closed? Is the shape filled?
    let line1 = Line {
       points: points2,
       is_closed: false,
       has_fill: false,
       has_stroke: true,
       is_clipping_path: false,
    };

    // Draw first arc
    current_layer.add_shape(line1);
}

// Determining values for circles
// via stackeoverflow https://stackoverflow.com/questions/1734745/how-to-create-circle-with-bézier-curves
// The following creates 4 bezier arcs for a unit circle
// P0​=(0,1),P1=(c,1),P2​=(1,c),P3​=(1,0)
// P0​=(1,0),P1​=(1,−c),P2​=(c,−1),P3​	=(0,−1)
// P0​=(0,−1),P1​=(−c,−1),P2=(−1,−c),P3​=(−1,0)
// P0​=(−1,0),P1​=(−1,c),P2​=(−c,1),P3​=(0,1)
// with c=0.551915024494
// Return the point values for a circle
fn get_points_for_circle(xoff: f64, yoff: f64, scale: f64) -> Vec<(Point, bool)> {

    // bezier constant for arc that will fit a circle
    let c= 0.551915024494;

    // first unity arc scaled and translated
    let p0_1 = Point::new(Mm(0.0*scale+xoff), Mm(1.0*scale+yoff));
    let p1_1 = Point::new(Mm(c*scale+xoff), Mm(1.0*scale+yoff));
    let p2_1 = Point::new(Mm(1.0*scale+xoff), Mm(c*scale+yoff));
    let p3_1 = Point::new(Mm(1.0*scale+xoff), Mm(0.0*scale+yoff));

    // second unity arc scaled and translated
    let p0_2 = Point::new(Mm(1.0*scale+xoff), Mm(0.0*scale+yoff));
    let p1_2 = Point::new(Mm(1.0*scale+xoff), Mm(-c*scale+yoff));
    let p2_2 = Point::new(Mm(c*scale+xoff), Mm(-1.0*scale+yoff));
    let p3_2 = Point::new(Mm(0.0*scale+xoff), Mm(-1.0*scale+yoff));

    // third unity arc scaled and translated
    let p0_3 = Point::new(Mm(0.0*scale+xoff), Mm(-1.0*scale+yoff));
    let p1_3 = Point::new(Mm(-c*scale+xoff), Mm(-1.0*scale+yoff));
    let p2_3 = Point::new(Mm(-1.0*scale+xoff), Mm(-c*scale+yoff));
    let p3_3 = Point::new(Mm(-1.0*scale+xoff), Mm(0.0*scale+yoff));

    // fourth unity arc scaled and translated
    let p0_4 = Point::new(Mm(-1.0*scale+xoff), Mm(0.0*scale+yoff));
    let p1_4 = Point::new(Mm(-1.0*scale+xoff), Mm(c*scale+yoff));
    let p2_4 = Point::new(Mm(-c*scale+xoff), Mm(1.0*scale+yoff));
    let p3_4 = Point::new(Mm(0.0*scale+xoff), Mm(1.0*scale+yoff));

    // return the vector of points
    // p0, p1 'true' values indicate that the following points are bezier control points
    vec![(p0_1,true),(p1_1,true),(p2_1,false),(p3_1,false),
         (p0_2,true),(p1_2,true),(p2_2,false),(p3_2,false),
         (p0_3,true),(p1_3,true),(p2_3,false),(p3_3,false),
         (p0_4,true),(p1_4,true),(p2_4,false),(p3_4,false)]

}

// return start and and points for a line,
// and an offset into the page from the lower left corner.
pub fn get_points_for_line<P: Into<Pt>>(
    start_x: P,
    start_y: P,
    end_x: P,
    end_y: P,
) -> Vec<(Point, bool)> {
    let (start_x, start_y, end_x, end_y) = (
        start_x.into(),
        start_y.into(),
        end_x.into(),
        end_y.into(),
    );

    let start_pt = Point { x: start_x, y: start_y };
    let end_pt = Point { x: end_x, y: end_y };
    // println!("Start Point: {:?} End Point{:?}", &start_pt, &end_pt);

    vec![
        (start_pt, false),
        (end_pt, false),
    ]

}
