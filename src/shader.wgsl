// vertex shader

struct Uniforms {//3 uniform matrixes
    model_mat : mat4x4<f32>,
    view_project_mat : mat4x4<f32>,
    normal_mat : mat4x4<f32>,//transformation matrix for the normal data
};

@binding(0) @group(0) var<uniform> uniforms : Uniforms;

struct Output {//output structure contains three fields
    @builtin(position) position : vec4<f32>,
    @location(0) v_position : vec4<f32>,//vertex data
    @location(1) v_normal : vec4<f32>,//normal data
};

@vertex
fn vs_main(@location(0) pos: vec4<f32>, @location(1) normal: vec4<f32>) -> Output {//only perform model transformation on the v_position used in light calculation
    var output: Output;
    let m_position:vec4<f32> = uniforms.model_mat * pos;
    output.v_position = m_position;
    output.v_normal =  uniforms.normal_mat * normal;//normal matrix times the original normal data
    output.position = uniforms.view_project_mat * m_position;
    return output;
}


// fragment shader

struct FragUniforms {//used to pass light position and eye position
    light_position : vec4<f32>,
    eye_position : vec4<f32>,
};
@binding(1) @group(0) var<uniform> frag_uniforms : FragUniforms;

struct LightUniforms {//pass parameters for the light calculation
    color : vec4<f32>,
    specular_color : vec4<f32>,
    ambient_intensity: f32,
    diffuse_intensity :f32,
    specular_intensity: f32,
    specular_shininess: f32,
};
@binding(2) @group(0) var<uniform> light_uniforms : LightUniforms;

@fragment
fn fs_main(@location(0) v_position: vec4<f32>, @location(1) v_normal: vec4<f32>) ->  @location(0) vec4<f32> {//calculate the light model
    let N:vec3<f32> = normalize(v_normal.xyz);//normal vector
    let L:vec3<f32> = normalize(frag_uniforms.light_position.xyz - v_position.xyz);//calc light direction
    let V:vec3<f32> = normalize(frag_uniforms.eye_position.xyz - v_position.xyz);//view direction
    let H:vec3<f32> = normalize(L + V);//calc half angle
    let diffuse:f32 = light_uniforms.diffuse_intensity * max(dot(N, L), 0.0);
    let specular: f32 = light_uniforms.specular_intensity * pow(max(dot(N, H),0.0), light_uniforms.specular_shininess);
    let ambient:f32 = light_uniforms.ambient_intensity;
    return light_uniforms.color*(ambient + diffuse) + light_uniforms.specular_color * specular;
}