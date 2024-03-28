const MAGIC:f32 = 43758.5453123;

fn random2d(st:vec2<f32>) -> vec2<f32> {
    let x = dot(st, vec2(127.1, 311.7));
    let y = dot(st, vec2(269.5, 183.3));
    let s = vec2(x, y);
    return -1.0 + 2.0 * fract(sin(s) * MAGIC);
}

fn interpolate(t:f32) -> f32 {
    return t * t * t * (10.0 + t * (6.0 * t - 15.0)); // smoothstep
}

fn gradientNoise(p:vec2<f32>) -> vec4<f32> {
    let i = floor(p);
    let f = fract(p);

    let f11 = dot(random2d(i + vec2(0.0, 0.0)), f - vec2(0.0, 0.0));
    let f12 = dot(random2d(i + vec2(0.0, 1.0)), f - vec2(0.0, 1.0));
    let f21 = dot(random2d(i + vec2(1.0, 0.0)), f - vec2(1.0, 0.0));
    let f22 = dot(random2d(i + vec2(1.0, 1.0)), f - vec2(1.0, 1.0));
    
    return vec4(f11, f12, f21, f22);
}

fn perlinNoise(p:vec2f) -> f32 {
    let v = gradientNoise(p);

    let f = fract(p);
    let t = interpolate(f.x);
    let u = interpolate(f.y);
    
    return mix(
        mix(v.x, v.z, t),
        mix(v.y, v.w, t), 
        u
    ) * 0.5 + 0.5;
}

fn lerpColor(colors:array<vec3f,5>, ta1:array<f32,6>, t:f32) -> vec3f {
    var res = vec3(0.0);
    var rgbData = colors;
    var ta = ta1;

    for(var i = 0u; i < 5u; i = i + 1u){
        if(t >= ta[i] && t < ta[i+1u]){
            res = rgbData[i];
        }
    }

    if(t == ta[5u]){
        res = rgbData[4u];
    }
    return res;
}

fn addTerrainColors(rgbData:array<vec3f,5>, ta:array<f32,6>, tmin:f32, tmax:f32, t:f32, waterLevel:f32) -> vec3f {
    var tt = t;
    if(t < tmin){tt = tmin;}
    if(t > tmax){tt = tmax;}
    if(tmin == tmax) {return vec3(0.0);}
    tt = (tt-tmin)/(tmax-tmin);

    var t1 = array<f32, 6>(0.0,0.0,0.0,0.0,0.0,0.0);
    let r = (1.0 - waterLevel)/(1.0 - ta[1]);
    var del:f32 = 0.0;
    t1[1] = waterLevel;
    
    for(var i:u32 = 1u; i < 5u; i = i + 1u) {
        if (i == 1u) {
            del = ta[2u] - ta[1u];
            t1[2] = t1[1] + r * del;
        } else if (i == 2u) {
            del = ta[3u] - ta[2u];
            t1[3] = t1[2] + r * del;
        } else if (i == 3u) {
            del = ta[4u] - ta[3u];
            t1[4] = t1[3] + r * del;
        } else if (i == 4u) {
            del = ta[5u] - ta[4u];
            t1[5] = t1[4] + r * del;
        }
    }

    return lerpColor(rgbData, t1, tt);
}