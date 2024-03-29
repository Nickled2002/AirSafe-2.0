@group(0) @binding(0) var<storage, read_write> indices: array<u32>;
@group(0) @binding(1) var<uniform> resolution: u32;

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(workgroup_id) wid : vec3u, @builtin(local_invocation_id) lid: vec3u) {
    let i = lid.x + wid.x * 8u;
    let j = lid.y + wid.y * 8u;  

    if(i >= resolution - 1u || j >= resolution - 1u ) { return; } 

    let idx = (i + j * (resolution - 1u)) * 6u;

    // first triangle
    indices[idx] = i + j * resolution;
    indices[idx + 1u] = i + (j + 1u) * resolution;
    indices[idx + 2u] = i + 1u + j * resolution;

    // second triangle
    indices[idx + 3u] = i + 1u + j * resolution;
    indices[idx + 4u] = i + (j + 1u) * resolution;
    indices[idx + 5u] = i + 1u + (j  + 1u) * resolution;

}