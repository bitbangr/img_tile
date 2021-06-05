use printpdf::*;
use printpdf::utils::calculate_points_for_circle;
// use printpdf::utils::calculate_points_for_rect;
use std::{collections::HashMap, fs::File};
use std::ops::Range;
use std::io::BufWriter;
use regex::Regex;
use std::option::Option::Some;use crate::modtile;
use euclid::{Point2D,Box2D};

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
// pub(crate) fn build_output_pdf(save_path: &&std::path::Path,
//                                 mosaic_colours: Vec<(u8, u8, u8)>,
//                                 all_colors: &crate::modtile::AllColors,
//                                 tile_color_count_vec: &Vec<(&&Vec<u8>, &i32)>,
//                                 cb: &Vec<euclid::Box2D<i32, i32>>) -> Result<(), String> {
pub(crate) fn build_output_pdf(save_path: &std::path::Path,
                               all_colors: &modtile::AllColors,
                               _tile_color_count_vec: &Vec<(&&Vec<u8>, &i32)>,
                               output_window: &Vec<Vec<(euclid::Box2D<i32, i32>, modtile::RGB)>>) -> () {


    let doc_width_mm = 279.4;
    let doc_height_mm = 215.9;
    let (doc, page1, layer1) =
        PdfDocument::new(&all_colors.name.to_owned(), Mm(doc_width_mm), Mm(doc_height_mm), "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);

    let font1 = doc
        .add_external_font(File::open("/System/Library/Fonts/NewYork.ttf").unwrap())
        .unwrap();

    let fill_color = Color::Cmyk(Cmyk::new(0.0, 0.23, 0.0, 0.0, None));
    let outline_color = Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None));
    let mut dash_pattern = LineDashPattern::default();
    dash_pattern.dash_1 = Some(20);

    current_layer.set_fill_color(fill_color);
    current_layer.set_outline_color(outline_color);
    current_layer.set_outline_thickness(2.0);

    // construct a grid (window panes) on current layer
    construct_window_panes(&current_layer, doc_width_mm,doc_height_mm , font1 , output_window);

    // save build instructions to same output file name but with pdf extension
    let fileout = save_path.with_extension("pdf");
    doc.save(&mut BufWriter::new(
        File::create(fileout).unwrap(),
    )).unwrap();

    // mgj todo some proper error handling
    // Ok(())
    // Err("something broke".to_owned())

}

// Draw main pdf window with panes (i.e. grid) to match output photo window panes
// Layout of panes, tiles and colors are all contained within passed output_window
//
// Each pane dimension a Box2d(start, end) with values as px (not mm)
//      is derivable from First tile min (x,y) and Last Tile max(x,y)
//
//  As an example:
//   a sample input image size 225x50 px, determined from physical file
//
//  The following are specified in the supplied input Config file
//         output units are unimportant mm, cm, inches, feet etc
//    output_width: 675.0  (happens to be 3 times input image width)
//   output_height: 150.0  (happens to be 3 times input image height)
//     tile_size_x: 225    (happens to be 1/3 of output width)
//     tile_size_y: 75     (happens to be 1/3 of output height)
//
//  Results in the following output:
//      1 row of of 3 panes (columns)
//      each pane contains 2 rows (vertical) and 1 column (horizontal) of tiles
//
//     |---------------max---------------|----------------|
//     |                |                |                |  75 units high
//     |****************|****************|****************|
//     |                |                |                |  75 units high
//    min---------------|----------------|----------------|
//       225 units wide   225 units wide   225 units wide
//
// Given a physical PDF output document "letter size" in landscape orientation
//    doc_width_mm = 279.4mm and doc_height_mm = 215.9mm
//    horizontal and vertical page margins of 20mm each
//
//    And that the output PDF window has the same aspect ratio as the input image
// then tile width and height in MM used in output PDF can be determined with basic math.
//
fn construct_window_panes(current_layer: &PdfLayerReference,
                         doc_width_mm: f64,
                         doc_height_mm: f64,
                         grid_font: IndirectFontRef,
                         output_window: &Vec<Vec<(Box2D<i32, i32>, modtile::RGB)>>) -> () {

    println!("construct_window_panes number of panes: {}", output_window.len());
    println!("construct_window_panes number of tiles per pane: {}", output_window[0].len());

    // draw a simple quarter arc at (0,0). Leave as a "makers mark"
    draw_quarter_arc(&current_layer);

    // draw some marks to aid in element placement
    draw_page_marks(&current_layer,doc_width_mm,doc_height_mm);

    // PDF Coord based on lower bottom left as being origin
    // Get pane_pdf coord adust the Box2D min max values accordingly
    let (img_max_x_px, img_max_y_px, win_pane_row_count, win_pane_col_count, pane_tile_row_count, pane_tile_col_count, window_panes_coords) : (i32,i32, // max_x, max_y
                                                                                                   i32,i32, // pane_count_x, pane_count_y
                                                                                                   i32,i32, // tile_x_count, tile_y_count
                                                                                                   Vec<Box2D<i32,i32>>) = get_pane_pdf_coords(output_window);

    // return a PDF output window where all Box2D coords translated from image coord space to PDF coord space
    let pdf_output_window :Vec<Vec<(Box2D<i32, i32>, modtile::RGB)>> = get_pdf_coords(output_window,img_max_y_px);

    let page_margin_ver_mm = 20.0; // size of top bottom margin
    let page_margin_hor_mm = 20.0;  // size of left right margin

    let imgtile_wid_px :f64 = img_max_x_px as f64/ win_pane_col_count as f64 / pane_tile_col_count as f64;
    let imgtile_hgt_px :f64 = img_max_y_px as f64/ win_pane_row_count as f64 / pane_tile_row_count as f64;

    let pdftile_wid_mm = (doc_width_mm - (2.0 * page_margin_hor_mm)) / win_pane_col_count as f64 / pane_tile_col_count as f64;

    // want pdf tile height and width to remain proportional to original input imagetile height and width
    let pdftile_hgt_mm = pdftile_wid_mm * imgtile_hgt_px/imgtile_wid_px;  // <----- Hopefully this works!
    // let tile_hgt_mm = 26.5;  // Harded value actually produces correct output box

    println!("?? ---> img_max_x_px {}, img_max_y_px {}", img_max_x_px, img_max_y_px );
    println!("??---> imgtile_wid_px: {}, imgtile_hgt_px {}", imgtile_wid_px, imgtile_hgt_px );
    println!("??---> pdftile_wid_mm: {}, pdftile_hgt_mm {}", pdftile_wid_mm, pdftile_hgt_mm );

    let grid_origin_x_mm :f64 = page_margin_hor_mm;  // Origin point (lower left corner of grid)
    let grid_origin_y_mm :f64 = page_margin_ver_mm;

    let outline_color = Color::Rgb(Rgb::new(0.5, 0.5, 0.5, None)); // gray
    current_layer.set_outline_color(outline_color);
    current_layer.set_outline_thickness(1.5);

    draw_summary_circles(pdf_output_window,
                        &current_layer,
                        doc_width_mm,
                        doc_height_mm,
                        page_margin_hor_mm,
                        page_margin_ver_mm,
                        grid_origin_x_mm,
                        grid_origin_y_mm,
                        pdftile_wid_mm,
                        pdftile_hgt_mm,
                        img_max_x_px,
                        img_max_y_px);

    let outline_color = Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)); // black
    current_layer.set_outline_color(outline_color);
    current_layer.set_outline_thickness(2.0);

    // for each window pane column draw a vertical line
    for column in 0..=win_pane_col_count // 0,1,2,3 , zero based
    {
        // Convert Mm into Pt function.
        let start_x : Pt = Mm(grid_origin_x_mm + column as f64 * pane_tile_col_count as f64 * pdftile_wid_mm as f64).into();
        let start_y : Pt = Mm(grid_origin_y_mm).into();
        let end_x : Pt = start_x.clone();  // drawing a vertical line so x remains the same
        let end_y : Pt = Mm(grid_origin_y_mm + win_pane_row_count as f64 * pane_tile_row_count as f64 * pdftile_hgt_mm as f64).into();

        let line = Line {
            points: get_points_for_line(start_x, start_y, end_x, end_y),
            is_closed: false,
            has_fill: false,
            has_stroke: true,
            is_clipping_path: false,
        };
        current_layer.add_shape(line);
    }

    // for each window pane row draw a horizontal line
    for row in 0..=win_pane_row_count // 0,1,2,3 , zero based
    {
        println!("---->start_y_pt = ({:.2})+({:.2}) * ({:.2}) * ({:.2}) \ngrid_origin_y_mm + row * pane_tile_row_count * pdftile_hgt_mm", grid_origin_y_mm, row , pane_tile_row_count , pdftile_hgt_mm) ;  //  <--- something wrong here)
        // Convert Mm into Pt function.
        let start_x_pt : Pt = Mm(grid_origin_x_mm).into();
        let start_y_pt : Pt = Mm(grid_origin_y_mm + row as f64 * pane_tile_row_count as f64 * pdftile_hgt_mm as f64).into();  //  <--- something wrong here
        let end_x_pt : Pt = Mm(grid_origin_x_mm + win_pane_col_count as f64 * pane_tile_col_count as f64 * pdftile_wid_mm as f64).into();
        let end_y_pt : Pt = start_y_pt.clone();  // drawing a horizontal line so y remains the same

        let line = Line {
            points: get_points_for_line(start_x_pt, start_y_pt, end_x_pt, end_y_pt),
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

    // // Write out the major grid numbers
    // let text_loc: Vec<(f64,f64,String)> = get_grid_text_loc( grid_origin_x, grid_origin_y, grid_major, grid_minor,  grid_div);
    // for item in text_loc {
    //     // println!("text location {:?}", item);
    //     current_layer.use_text(item.2, 60.0, Mm(item.0), Mm(item.1), &grid_font);
    // }

}


fn draw_page_marks(current_layer: &&PdfLayerReference, doc_width_as_mm: f64, doc_height_as_mm: f64) -> () {

    current_layer.set_outline_thickness(0.5);

    let doc_min_point: Point = Point {x: Pt(0.0), y: Pt(0.0)};
    let doc_max_point: Point = Point::new(Mm(doc_width_as_mm), Mm(doc_height_as_mm));

    let  doc_width_pt:Pt = Mm(doc_width_as_mm).into();
    let doc_height_pt:Pt = Mm(doc_height_as_mm).into();

    // A cross is made up from 1 horizontal line and 1 vertical line (8 points)
    let hor_min_x: Pt = Mm(-2.5).into();
    let hor_min_y: Pt = Mm(0.0).into();
    let hor_max_x: Pt = Mm(2.5).into();
    let hor_max_y: Pt = Mm(0.0).into();

    let ver_min_x: Pt = Mm(0.0).into();
    let ver_min_y: Pt = Mm(-2.5).into();
    let ver_max_x: Pt = Mm(0.0).into();
    let ver_max_y: Pt = Mm(2.5).into();

    // draw multiple crosses vertically and horizontally spaced across the page at 10 mm invervals
    let step = 10;  // mm
    let step_pt: Pt = Mm(10.0).into();
    let mut rcount: f64 = 0.0 ;
    for row in (0..=doc_height_as_mm as i32).step_by(step){
        // let row64 = row as f64;

        let mut ccount: f64 = 0.0;
        for col in (0..=doc_width_as_mm as i32).step_by(step){
        // println!("Did we get here?{:?} {:?} {:?} {:?}", row, rcount, col, ccount);

            let hmin_x: Pt = Pt(hor_min_x.0 + ccount * step_pt.0); // Mm(-2.5).into();
            let hmin_y: Pt = Pt(hor_min_y.0 + rcount * step_pt.0); // Mm(0.0).into();
            let hmax_x: Pt = Pt(hor_max_x.0 + ccount * step_pt.0); // Mm(2.5).into();
            let hmax_y: Pt = Pt(hor_max_y.0 + rcount * step_pt.0); // Mm(0.0).into();

            let vmin_x: Pt = Pt(ver_min_x.0 + ccount * step_pt.0); // Mm(0.0).into();
            let vmin_y: Pt = Pt(ver_min_y.0 + rcount * step_pt.0); // Mm(-2.5).into();
            let vmax_x: Pt = Pt(ver_max_x.0 + ccount * step_pt.0); // Mm(0.0).into();
            let vmax_y: Pt = Pt(ver_max_y.0 + rcount * step_pt.0); // Mm(2.5).into();

            let hmin = Point { x: hmin_x,  y: hmin_y };
            let hmax = Point { x: hmax_x,  y: hmax_y};
            let hline_pts = vec![(hmin, false),(hmax, false)];
            let hline_line = Line {
                points: hline_pts,
                is_closed: false,
                has_fill: false,
                has_stroke: true,
                is_clipping_path: false,
            };
            current_layer.add_shape(hline_line);

            let vmin = Point { x: vmin_x,  y: vmin_y };
            let vmax = Point { x: vmax_x,  y: vmax_y};
            let vline_pts = vec![(vmin, false),(vmax, false)];
            let vline_line = Line {
                points: vline_pts,
                is_closed: false,
                has_fill: false,
                has_stroke: true,
                is_clipping_path: false,
            };
            current_layer.add_shape(vline_line);
            ccount += 1.0;
        }
        rcount += 1.0;
    }

}

fn draw_summary_circles(pdf_output_window: Vec<Vec<(Box2D<i32, i32>,
                        modtile::RGB)>>,
                        current_layer: &&PdfLayerReference,
                        doc_width_mm: f64,
                        doc_height_mm: f64,
                        page_margin_hor_mm: f64,
                        page_margin_ver_mm: f64,
                        grid_origin_x_mm: f64,
                        grid_origin_y_mm: f64,
                        tile_wid_mm: f64,
                        tile_hgt_mm: f64,
                        max_x_px: i32,
                        max_y_px: i32) -> () {

    // lets convert everything to Pt for strong typing
    let doc_width_pt: Pt = Mm(doc_width_mm).into();
    let doc_height_pt: Pt = Mm(doc_height_mm).into();
    let page_margin_hor_pt: Pt = Mm(page_margin_hor_mm).into();
    let page_margin_ver_pt: Pt = Mm(page_margin_ver_mm).into();
    let grid_origin_x_pt: Pt = Mm(grid_origin_x_mm).into();
    let grid_origin_y_pt: Pt = Mm(grid_origin_y_mm).into();
    // let tile_wid_pt: Pt = Mm(tile_wid_mm).into();
    // let tile_hgt_pt: Pt = Mm(tile_hgt_mm).into();

    // radius is based of the smaller of the x or y divisions
    let radius_mm = if tile_wid_mm < tile_hgt_mm {
        // tile_wid_mm / 2.0 - 0.30
        tile_wid_mm / 2.0
    } else {
        // tile_hgt_mm / 2.0 - 0.30
        tile_hgt_mm / 2.0
    };

    let radius_pt :Pt = Mm(radius_mm).into();

    // create an input image box that is size of our image
    // for testing  Specifically hard coded to size of  "input":"./images/Kroma_6_2x3_nonsquare.png",
    // origin is assumed to be (0,0)
    // let imgbox_wid_pt: Pt = Mm(225.0).into();
    // let imgbox_hgt_pt: Pt = Mm(50.0).into();
    // let imgbox_wid_pt: Pt = Mm(675.0).into();   // < >---output image  Changing this value changes the circle locations. It should only act as scale
    // let imgbox_hgt_pt: Pt = Mm(150.0).into();

    // create output box scaled to fit PDF page width.
    // i.e. OP box is not actual real live size
    //   This output box has same aspect ratio of img_box image width and height
    // Specifically PDF box width is coded to be pdf doc_width_mm minus both margins.
    let pdf_start_x_pt: Pt = Mm(0.0).into();
    let pdf_start_y_pt: Pt = Mm(0.0).into();

    let pdf_end_x_pt: Pt = Mm(doc_width_mm - 2.0 * page_margin_hor_mm).into() ;
    let pdf_img_width_pt: Pt = Pt(pdf_end_x_pt.0 - pdf_start_x_pt.0);

    // scale pdf y to match input image aspect ratio
    // want div_x and div_y to remain proportional to original image and not depend out output size
    // let pdf_end_y_pt: Pt =  Pt (imgbox_hgt_pt.0 / imgbox_wid_pt.0 * pdf_img_width_pt.0);
    // let pdf_end_y_pt: Pt =  Pt (150.0 / 675.0 * doc_width_mm - 2.0 * page_margin_hor_mm);

    // println!("***** pdf_start_x_pt: {:?}, pdf_start_y_pt: {:?},\npdf_end_x_pt: {:?}, pdf_end_y_pt: {:?}", pdf_start_x_pt, pdf_start_y_pt, pdf_end_x_pt ,pdf_end_y_pt );
    // println!("pdf_end_x_pt / pdf_end_y_pt: {}" , pdf_end_x_pt / pdf_end_y_pt);
    //
    // dbg!(&pdf_end_x_pt,  &pdf_end_y_pt );

// *****
    // todo - mgj check this code out
    println!("Need to examine draw summary circles red line shows output PDF dimension properly scaled\n*****\n*****\n");
    let outline_color = Color::Rgb(Rgb::new(1.0, 0.0, 0.0, None)); // red
    current_layer.set_outline_color(outline_color);

    // // draw a diagonal line representing output pdf
    // let hmin = Point { x: pdf_start_x_pt,  y: pdf_start_y_pt };
    // let hmax = Point { x: pdf_end_x_pt,  y: pdf_end_y_pt};
    // let hline_pts = vec![(hmin, false),(hmax, false)];
    // let hline_line = Line {
    //     points: hline_pts,
    //     is_closed: false,
    //     has_fill: false,
    //     has_stroke: true,
    //     is_clipping_path: false,
    // };
    // current_layer.add_shape(hline_line);
// *****

    let outline_color = Color::Rgb(Rgb::new(0.5, 0.5, 0.5, None)); // gray
    current_layer.set_outline_color(outline_color);
    for (_i, pane) in pdf_output_window.iter().enumerate() {
        for (_j, tile) in pane.iter().enumerate(){

            let tile_box = tile.0;
            let tile_rgb = tile.1;

            let red = tile_rgb.0 as f64;
            let green = tile_rgb.1 as f64;
            let blue = tile_rgb.2 as f64;

            let fill_color = Color::Rgb(Rgb::new(red/255.0, green/255.0,blue/255.0, None));
            current_layer.set_fill_color(fill_color);

            // // create an iterator over a vector of points and translate each point by some x,y offset
            let mut rect_points = get_points_for_rect(Pt(tile_box.width() as f64), Pt(tile_box.height() as f64), Pt(tile_box.min.x as f64), Pt(tile_box.min.y as f64));
            // for pt in rect_points.into_iter().map(|translate| { px});

            // let mut c = 0;
            // for pair in vec!['a', 'b', 'c'].into_iter().map(|letter| { c += 1; (letter, c) }) {
            //     println!("{:?}", pair);
            // }

            // translate this image size box into pdf size box

            println!("Rect_points {:?}", &rect_points);
            let line = Line {
                points: rect_points,
                is_closed: true,
                has_fill: true,
                has_stroke: true,
                is_clipping_path: false
            };

            current_layer.add_shape(line);


            // Try drawing rectangles
            // let box_center = tile_box.center();
            //
            // //this one works-ish
            // // let center_x_pt: Pt = Pt( box_center.x as f64 * (imgbox_wid_pt/pdf_end_x_pt) + page_margin_hor_pt.0 );
            // // let center_y_pt: Pt = Pt( box_center.y as f64 * (imgbox_hgt_pt/pdf_end_y_pt) + page_margin_ver_pt.0);
            // let trans_x_pt = Mm( tile_wid_mm /2.0);
            // let trans_y_pt = Mm( tile_hgt_mm /2.0);
            //
            // let center_x_pt: Pt = Pt( box_center.x as f64 + trans_x_pt.0 + page_margin_hor_pt.0 );
            // let center_y_pt: Pt = Pt( box_center.y as f64 + trans_y_pt.0 + page_margin_ver_pt.0);
            //
            // let center_x_mm: Mm = center_x_pt.into();
            // let center_y_mm: Mm = center_y_pt.into();
            //
            // println!("Tile Box {:?}", tile);
            // println!("Box Center{:?}", box_center);
            // println!("circle loc pt: x: {:.2?}, y: {:.2?}", center_x_pt, center_y_pt);
            // println!("circle loc mm: x: {:.2?}, y: {:.2?}\n*****\n", center_x_mm, center_y_mm);
            //
            // draw_circle_with_pts(&current_layer, center_x_pt, center_y_pt, radius_pt) ;
        }
    }

    // Manually drawing small dot to mark where circles should be drawn
    dbg!(page_margin_hor_mm, page_margin_ver_mm);
    let radi:Pt = Mm(1.5).into();
    let x1:Pt = Mm(40.0 + page_margin_hor_mm).into();  // half of tile width
    let x2:Pt = Mm(120.0+ page_margin_hor_mm).into();  // next position is 1 tile width away 80mm
    let x3:Pt = Mm(200.0+ page_margin_hor_mm).into();  // next position is 1 tile width away 80mm

    let y1:Pt = Mm(13.25 + page_margin_ver_mm).into();
    let y2:Pt = Mm(3.0*13.25 + page_margin_ver_mm).into();

    println!("Pt (x1,y1):({:.2?},{:.2?})", x1,y1);
    println!("Pt (x2,y1):({:.2?},{:.2?})", x2,y1);
    println!("Pt (x3,y1):({:.2?},{:.2?})", x3,y1);
    println!("Pt (x1,y2):({:.2?},{:.2?})", x1,y2);
    println!("Pt (x2,y2):({:.2?},{:.2?})", x2,y2);
    println!("Pt (x3,y2):({:.2?},{:.2?})", x3,y2);

    draw_circle_with_pts(&current_layer, x1, y1, radi) ;
    draw_circle_with_pts(&current_layer, x2, y1, radi) ;
    draw_circle_with_pts(&current_layer, x3, y1, radi) ;

    draw_circle_with_pts(&current_layer, x1, y2, radi) ;
    draw_circle_with_pts(&current_layer, x2, y2, radi) ;
    draw_circle_with_pts(&current_layer, x3, y2, radi) ;

}

// TODO mgj Figure out why these are not the same and if it affecting placement of the circles
// seem to be a difference of  0.071428571428571 between two X/Y ratio -- Using (0,49) instead of (1,50) is causing rounding errors
// ---> tile_wid_mm: 79.8, tile_hgt_mm 17.45625, x/y 4.571428571428571
// ***** pdf_start_x_pt: Pt(0.0), pdf_start_y_pt: Pt(0.0),
// pdf_end_x_pt: Pt(678.6142523999999), pdf_end_y_pt: Pt(150.8031672)
// pdf_end_x_pt / pdf_end_y_pt: 4.5



// Convert all the Box2D coords from image coord space into PDF coord space.
// see get_pane_pdf_coords() below for explanation of how this code works
fn get_pdf_coords(output_window: &Vec<Vec<(Box2D<i32, i32>, modtile::RGB)>>, max_y: i32) -> Vec<Vec<(Box2D<i32, i32>, modtile::RGB)>> {
    // construct array to let us get equivalent y PDF coord from Image Coord
    let range = 0..=max_y;
    let mut y_pdf: Vec<i32> = Vec::new();
    for value in range.rev() {
      y_pdf.push(value);
    }

    let mut pdf_window : Vec<Vec<(Box2D<i32,i32>,modtile::RGB)>> = Vec::new();
    for (_i, pane) in output_window.iter().enumerate() {
        let mut pdf_pane : Vec<(Box2D<i32,i32>, modtile::RGB)> = Vec::new();
        for (j, tile) in pane.iter().enumerate(){
            let pdf_tile_rgb = tile.1;
            // see get_pane_pdf_coords() for explanation of how this code works
            // create pdf min/max box with image space coords
            let pdf_tile_min_x = tile.0.min.x;
            let pdf_tile_min_y = y_pdf[tile.0.max.y as usize];
            let pdf_tile_max_x = tile.0.max.x;
            let pdf_tile_max_y = y_pdf[tile.0.min.y as usize];

            // println!("pdf_tile_min (x,y) ({:?},{:?})", &pdf_tile_min_x, &pdf_tile_min_y);
            // println!("pdf_tile_max (x,y) ({:?},{:?})", &pdf_tile_max_x, &pdf_tile_max_y);

            let pdf_tile_min_pt: Point2D<i32,i32> = Point2D::new(pdf_tile_min_x,pdf_tile_min_y);
            let pdf_tile_max_pt: Point2D<i32,i32> = Point2D::new(pdf_tile_max_x,pdf_tile_max_y);

            let pdf_tile_box = Box2D {min: pdf_tile_min_pt, max: pdf_tile_max_pt};

            &pdf_pane.push ((pdf_tile_box,pdf_tile_rgb));
        }
        pdf_window.push(pdf_pane);
    }
    pdf_window
}

// PDF Coordinate system is based on lower bottom left as being origin
// adust the Box2D min max values accordingly
// Get the PX cooridinates of each window pane.
// esentially constructing a Box2D using first tile min loc and last tile max location.
fn get_pane_pdf_coords(output_window: &Vec<Vec<(Box2D<i32, i32>, modtile::RGB)>> ) -> (i32,i32, i32,i32, i32,i32, Vec<Box2D<i32,i32>>) {

    // grab the max x y dimensions
    let mut win_max_x : i32 = 0;
    let mut win_max_y : i32 = 0;
    let mut tile_row_count :i32 = 0;
    let mut tile_col_count :i32 = 0;

    for (i, pane) in output_window.iter().enumerate() {
        let tile_end = pane.last().unwrap().0;
        if tile_end.max.x > win_max_x {
            win_max_x = tile_end.max.x;
        }
        if tile_end.max.y > win_max_y {
            win_max_y = tile_end.max.y;
        }
        // for the first pane find out the number of tile columns and tile rows
        if i == 0 {
            // println!("get_pane_pdf_coords pane {}, {:?}", i, pane) ;
            get_xy_tile_count(&pane,&mut tile_row_count, &mut tile_col_count);
        }
    }

    println!("get_pane_pdf_coords- Each pane is {} row(s) by {} col(s) of tiles", &tile_row_count, &tile_col_count);

    // construct array to let us get PDF coord from Image Coord
    let range = 0..=win_max_y;
    let mut y_pdf: Vec<i32> = Vec::new();
    for value in range.rev() {
      y_pdf.push(value);
    }

    // keep count for number of times each pane x and y used as a tile
    let mut pane_x_coords: HashMap<i32, i32> = HashMap::new();
    let mut pane_y_coords: HashMap<i32, i32> = HashMap::new();

    let mut ret : Vec<Box2D<i32,i32>> = Vec::new();
    for (_i, pane) in output_window.iter().enumerate() {
        let tile_start = pane.first().unwrap().0;
        let tile_end = pane.last().unwrap().0;

        // Tiles are the same physical box in PDF space or image space.
        // The min max points describing that box in PDF coords are opposite to those in image coords

        // Image space px
        //  min(0,0)*******
        //      *         *
        //      *         *
        //      *         *
        //      ********(1,1)max

        //  derived PDF min/max coords in Image Space
        //      ********(1,0)max
        //      *         *
        //      *         *
        //      *         *
        //  min(0,1)*******

        // Translate pdf min max values from image space coords into PDF space coords
        //  x values are 1 to 1 mapping as x increases left to right for both images and pdf files
        //  y values are opposite for pdf and image space coords
        // for example if image space y axis has 4 elements 0,1,2,3 the corressponding pdf space axis will be 3,2,1,0
        //    image space y axis: 0,1,2,3
        //      pdf space y axis: 3,2,1,0

        //   PDF min/max coords after translation into PDF coord Space
        //      ********(1,3)max
        //      *         *
        //      *         *
        //      *         *
        //  min(0,2)*******

        // Using this information we can create our new PDF equivalent set of coords
        // create pdf min/max box with image space coords
        let pdf_min_x = tile_start.min.x;
        let pdf_min_y = y_pdf[tile_end.max.y as usize];
        let pdf_max_x = tile_end.max.x;
        let pdf_max_y = y_pdf[tile_start.min.y as usize];

        // println!("pdf_min (x,y) ({:?},{:?})", &pdf_min_x, &pdf_min_y);
        // println!("pdf_max (x,y) ({:?},{:?})", &pdf_max_x, &pdf_max_y);
        // println!("Window max_y {:?}", &win_max_y);
        // println!("Window max_x {:?}", &win_max_x);

        *pane_x_coords.entry(pdf_min_x).or_insert(0) += 1;
        *pane_x_coords.entry(pdf_min_x).or_insert(0) += 1;
        *pane_x_coords.entry(pdf_max_x).or_insert(0) += 1;
        *pane_x_coords.entry(pdf_max_x).or_insert(0) += 1;

        *pane_y_coords.entry(pdf_min_y).or_insert(0) += 1;
        *pane_y_coords.entry(pdf_min_y).or_insert(0) += 1;
        *pane_y_coords.entry(pdf_max_y).or_insert(0) += 1;
        *pane_y_coords.entry(pdf_max_y).or_insert(0) += 1;

        let pdf_min_pt: Point2D<i32,i32> = Point2D::new(pdf_min_x,pdf_min_y);
        let pdf_max_pt: Point2D<i32,i32> = Point2D::new(pdf_max_x,pdf_max_y);

        let pane_box = Box2D {min: pdf_min_pt, max: pdf_max_pt};
        &ret.push (pane_box);
    }

    // all panes do not overlap so (i.e 99 vs 100) so divide by 2 to get actual number of rows and columns
    // number of rows corresponds to number of discrete y coords values
    // number of cols corresponds to number of discrete x coords values
    let win_pane_row_count :i32 = pane_y_coords.len() as i32 / 2 ;
    let win_pane_col_count :i32 = pane_x_coords.len() as i32 / 2 ;

    println!{"width pane coords {:?}", &pane_x_coords}; //  width pane coords {0: 4, 99: 4, 100: 4, 199: 4}
    println!{"height pane coords {:?}", &pane_y_coords} // height pane coords {0: 4, 99: 4, 100: 4, 199: 4}

    println!("window pane row count: {:?}", &win_pane_row_count);  // pane row count is 1 correct
    println!("window pane col count: {:?}", &win_pane_col_count);  // pane col count is 3 incorrect... should be something is wrong here

    (win_max_x,win_max_y,win_pane_row_count, win_pane_col_count, tile_row_count, tile_col_count, ret)
}

// Get the number of tile rows and tile columns in a pane
// these values are identical for all panes
// number of rows corresponds to number of discrete y coords values
// number of cols corresponds to number of discrete x coords values
fn get_xy_tile_count(pane: &&Vec<(Box2D<i32, i32>, modtile::RGB)>, tile_row_count: &mut i32, tile_col_count: &mut i32) -> () {

    // keep count for number of times each pane x and y used as a tile
    let mut tile_x_coords: HashMap<i32, i32> = HashMap::new();
    let mut tile_y_coords: HashMap<i32, i32> = HashMap::new();

    for (_i, tile) in pane.iter().enumerate() {

        let tbox = tile.0;

        let tile_min_x = tbox.min.x;
        let tile_min_y = tbox.min.y;
        let tile_max_x = tbox.max.x;
        let tile_max_y = tbox.max.y;

        *tile_x_coords.entry(tile_min_x).or_insert(0) += 1;
        *tile_x_coords.entry(tile_max_x).or_insert(0) += 1;

        *tile_y_coords.entry(tile_min_y).or_insert(0) += 1;
        *tile_y_coords.entry(tile_max_y).or_insert(0) += 1;
    }
    *tile_row_count = tile_y_coords.len() as i32 /2 ;
    *tile_col_count = tile_x_coords.len() as i32 /2 ;
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

fn draw_circle(current_layer: &&PdfLayerReference, x_mm: f64, y_mm: f64, radius_mm: f64) -> () {

    // let xoff = location.x.0;
    // let yoff = location.y.0;
    // let circle_points_mm = get_points_for_circle(x_mm,y_mm,radius_mm);

    let radius = Pt(radius_mm);
    let offset_x = Pt(x_mm);
    let offset_y = Pt(y_mm);

    let circle_points = calculate_points_for_circle(radius, offset_x, offset_y);

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

fn draw_circle_with_pts(current_layer: &&PdfLayerReference, offsetx_pt: Pt, offsety_pt: Pt, radius_pt: Pt) -> () {

    let circle_points = calculate_points_for_circle(radius_pt, offsetx_pt, offsety_pt);

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
