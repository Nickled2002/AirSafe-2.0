mod common;
mod math_func;

fn main(){
    let mut colormap_name = "jet";
    let mut is_two_side:i32 = 1;
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        colormap_name = &args[1];
    }
    if args.len() > 2 {
        is_two_side = args[2].parse().unwrap();
    }

    let vertex_data = common::create_vertices(&math_func::sinc, colormap_name, -8.0, 8.0, -8.0, 8.0,
                                              30, 30, 2.0, 0.7);
    let light_data = common::light([1.0, 1.0, 1.0], 0.1, 0.8, 0.4, 30.0, is_two_side);
    common::run(&vertex_data, light_data, colormap_name, "sinc");
}