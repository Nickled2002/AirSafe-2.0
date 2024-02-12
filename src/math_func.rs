#![allow(dead_code)]


pub fn peaks (x:f32, z:f32, y:f32) -> [f32; 3] {
   // print!("{:?}", data);
    //let data = srtm::Tile::from_file("src/N03E021.hgt").unwrap();
    //let y1 :f32 = (srtm::Tile::get(&data, x as u32, z as u32) / 100) as f32;
    //new two input function finding y by translating and scaling Gaussian distribution will not be required with geographicaldata since XYZ will be given
   // let y = 3.0*(1.0-x)*(1.0-x)*(-(x*x)-(z+1.0)*(z+1.0)).exp()-
    //    10.0*(x/5.0-x*x*x-z*z*z*z*z)*(-x*x-z*z).exp() - 1.0/3.0*(-(x+1.0)*(x+1.0)-z*z).exp();
    [x, y, z]
}
