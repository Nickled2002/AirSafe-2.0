mod common;
mod vertex_data;
mod transforms;

fn vertex(p:[i8; 3], n: [i8; 3]) -> common::Vertex {
    common::Vertex {
        position: [p[0] as f32, p[1] as f32, p[2] as f32, 1.0],
        normal: [n[0] as f32, n[1] as f32, n[2] as f32, 1.0],
    }
}

fn create_vertices() -> Vec<common::Vertex> {
    let pos = vertex_data::cube_positions();
    let normal= vertex_data::cube_normals();

    let mut data:Vec<common::Vertex> = Vec::with_capacity(pos.len());
    for i in 0..pos.len() {
        data.push(vertex(pos[i], normal[i]));
    }
    data.to_vec()
}

fn main(){
    let vertex_data = create_vertices();
    let light_data = common::light([1.0,0.0,0.0], [1.0, 1.0, 0.0], 0.1, 0.6, 0.3, 30.0);
    common::run(&vertex_data, light_data, "cube");
}