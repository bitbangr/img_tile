# img_tile
  
img_tile converts an input image into a tiled image and produces an output "map" pdf file that can be used to construct a mosaic with actual ceramic tiles.
  Tile colours are specified in json format in a color file.
    
  Program parameters are specified in a json config file passed as a command line arg. 
    
This program is run via command line. For example

  workingdir%target/debug/img_tile --config ./config/config_200px_kroma_2x2.json 
  
The input config file is in json format. 
  
{ "tile_colors":"./tile_json/kroma_colors.json",  
  "input":"./images/4x4_Kroma_16.png",  
  "output":"./images/output/2x2_kroma.jpg",  
  "output_width":200.0,  
  "output_height":200.0,  
  "tile_size_x":50,  
  "tile_size_y":50,  
  "tile_space_x":0.0,  
  "tile_space_y":0.0,  
  "tiles_per_pane_width":2,  
  "tiles_per_pane_height":2}
  
img_tile is written in Rust.  
