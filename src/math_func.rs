#![allow(dead_code)]
pub fn sinc (x:f32, z:f32) -> [f32; 3] {
    //sinc function takes xz and calcs y so outputs xyz
    let r = (x*x + z*z).sqrt();
    let y = if r == 0.0 { 1.0 } else { r.sin()/r };
    [x, y, z]
}
