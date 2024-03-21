mod common;


fn main(){
    let mut sample_count = 1 as u32;
    let mut width = 200u32;
    let mut height = 200u32;
    let mut colormap_name = "mountain";
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        sample_count = args[1].parse::<u32>().unwrap();
    }
    if args.len() > 2 {
        width = args[2].parse::<u32>().unwrap();
    }
    if args.len() > 3 {
        height = args[3].parse::<u32>().unwrap();
    }
    if args.len() > 4 {
        colormap_name = &args[4];
    }
    //let data: Tile = Tile::from_file("src/N03E021.hgt").unwrap();
    //create vertex data from common rs file and using the function from mathfunc.rs file
    //let vertex_data = common::create_vertices(colormap_name, 0.0, 3600.0, 0.0, 3600.0, 30, 30, 2.0, 0.9);//set scale to two and aspect ratio to .7 to .9 for more precise
    //let light_data = common::light([1.0, 1.0, 1.0], 0.1, 0.8, 0.4, 30.0, 1);//1,1,1 for specular light color and set light intensity
    common::run(width, height, colormap_name);//create sinc surface now peaks
}