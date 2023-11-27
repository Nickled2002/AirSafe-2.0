#![allow(dead_code)]
pub fn peaks (x:f32, z:f32) -> [f32; 3] {
    //new two input function finding y by translating and scaling Gaussian distribution will not be required with geographicaldata since XYZ will be given
    let y = 3.0*(1.0-x)*(1.0-x)*(-(x*x)-(z+1.0)*(z+1.0)).exp()-
        10.0*(x/5.0-x*x*x-z*z*z*z*z)*(-x*x-z*z).exp() - 1.0/3.0*(-(x+1.0)*(x+1.0)-z*z).exp();
    [x, y, z]
}
pub fn sinc (x:f32, z:f32) -> [f32; 3] {
    //sinc function takes xz and calcs y so outputs xyz
    let r = (x*x + z*z).sqrt();
    let y = if r == 0.0 { 1.0 } else { r.sin()/r };
    [x, y, z]
}
