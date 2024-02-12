mod common;
mod math_func;

fn main(){
    let mut colormap_name = "mountain";//color map name
    let mut is_two_side:i32 = 0;//one sided no lighting on the under side
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        colormap_name = &args[1];
    }
    if args.len() > 2 {
        is_two_side = args[2].parse().unwrap();
    }
    //let data: Tile = Tile::from_file("src/N03E021.hgt").unwrap();
    //create vertex data from common rs file and using the function from mathfunc.rs file
    let vertex_data = common::create_vertices(&math_func::peaks, colormap_name, 0.0  , 7200.0, 0.0, 7200.0,
                                              60, 60, 2.0, 0.8);//set scale to two and aspect ratio to .7 to .9 for more precise
    let light_data = common::light([1.0, 1.0, 1.0], 0.1, 0.8, 0.4, 30.0, is_two_side);//1,1,1 for specular light color and set light intensity
    common::run(&vertex_data, light_data, colormap_name, "Peaks");//create sinc surface now peaks
}