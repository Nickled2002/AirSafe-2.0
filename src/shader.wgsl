struct VertexInput {
    @location(0) pos: vec2<f32>,//position input at location 0 vec2 x,y
    @location(1) color: vec3<f32>,//color input at location 1 vec3 r,g,b
    //must be consistant with the shader location define rust code
    //set relationship between shader and buffer
};
//havent defined any vertices and colors we will store that data in gpu buffers

struct VertexOutput {//output for pos and color
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.color = vec4<f32>(in.color, 1.0);//process color data
    out.position = vec4<f32>(in.pos, 0.0, 1.0);//process position data to vec 4
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}