// vertex shaders

struct Uniforms {
    model_mat : mat4x4<f32>,
    view_project_mat : mat4x4<f32>,
    normal_mat : mat4x4<f32>,
};
@binding(0) @group(0) var<uniform> uniforms : Uniforms;

struct Input {
    @location(0) pos : vec4<f32>,
    @location(1) normal : vec4<f32>,
    @location(2) color : vec4<f32>,
};

struct Output {
    @builtin(position) position : vec4<f32>,
    @location(0) v_position : vec4<f32>,
    @location(1) v_normal : vec4<f32>,
    @location(2) v_color : vec4<f32>,
};

@vertex
fn vs_main(in: Input) -> Output {
    var output: Output;
    let m_position:vec4<f32> = uniforms.model_mat * in.pos;
    output.v_position = m_position;
    output.v_normal =  uniforms.normal_mat * in.normal;
    output.v_color =  in.color;
    output.position = uniforms.view_project_mat * m_position;
    return output;
}

// fragment shader

struct FragUniforms {
    light_position : vec4<f32>,
    eye_position : vec4<f32>,
};
@binding(1) @group(0) var<uniform> frag_uniforms : FragUniforms;

struct LightUniforms {
    specular_color : vec4<f32>,
    ambient_intensity: f32,
    diffuse_intensity :f32,
    specular_intensity: f32,
    specular_shininess: f32,
    is_two_side: i32,
};
@binding(2) @group(0) var<uniform> light_uniforms : LightUniforms;

@fragment
fn fs_main(in:Output) -> @location(0) vec4<f32> {
    let N:vec3<f32> = normalize(in.v_normal.xyz);
    let L:vec3<f32> = normalize(frag_uniforms.light_position.xyz - in.v_position.xyz);
    let V:vec3<f32> = normalize(frag_uniforms.eye_position.xyz - in.v_position.xyz);
    let H:vec3<f32> = normalize(L + V);

    // front side
    var diffuse:f32 = light_uniforms.diffuse_intensity * max(dot(N, L), 0.0);
    var specular: f32 = light_uniforms.specular_intensity * pow(max(dot(N, H),0.0), light_uniforms.specular_shininess);

    // back side
    if(light_uniforms.is_two_side == 1) {
        diffuse = diffuse + light_uniforms.diffuse_intensity * max(dot(-N, L), 0.0);
        specular = specular + light_uniforms.specular_intensity * pow(max(dot(-N, H),0.0), light_uniforms.specular_shininess);
    }

    let ambient:f32 = light_uniforms.ambient_intensity;
    let final_color:vec3<f32> = in.v_color.xyz*(ambient + diffuse) + light_uniforms.specular_color.xyz * specular;
    return vec4<f32>(final_color, 1.0);
}