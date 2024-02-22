#![allow(dead_code)]

use cgmath::*;
use srtm::Tile;

mod colormap;
//can make code more efficient by combining functions and only requiring two for loops
//also use four indexed vertex data instead of six vertices
pub fn simple_surface_colors(pts: &Vec<Vec<[f32; 3]>>, nx:usize, nz: usize, yrange:[f32; 2], colormap_name: &str) -> Vec<[f32; 3]> {
    let mut colors: Vec<[f32; 3]> = Vec::with_capacity((4* (nx - 1)*(nz -1)) as usize);
    for i in 0..nx - 1 {
        for j in 0.. nz - 1 {
            let p0 = pts[i][j];
            let p1 = pts[i][j+1];
            let p2 = pts[i+1][j+1];
            let p3 = pts[i+1][j];

            let c0 = colormap::color_interp(colormap_name, yrange[0], yrange[1], p0[1]);//calculate colors from color map based on y level
            let c1 = colormap::color_interp(colormap_name, yrange[0], yrange[1], p1[1]);
            let c2 = colormap::color_interp(colormap_name, yrange[0], yrange[1], p2[1]);
            let c3 = colormap::color_interp(colormap_name, yrange[0], yrange[1], p3[1]);
            //triangle 1
            colors.push(c0);//add color map to vertixes
            colors.push(c1);
            colors.push(c2);
            //triangle 2
            colors.push(c2);
            colors.push(c3);
            colors.push(c0);
        }
    }
    colors
}

pub fn simple_surface_normals(pts: &Vec<Vec<[f32; 3]>>, nx:usize, nz: usize) -> Vec<[f32;3]> {
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity((4* (nx - 1)*(nz -1)) as usize);
    for i in 0..nx - 1 {
        for j in 0.. nz - 1 {
            //four vertices for unit cell
            let p0 = pts[i][j];
            let p1 = pts[i][j+1];
            let p2 = pts[i+1][j+1];
            let p3 = pts[i+1][j];

            let ca = Vector3::new(p2[0]-p0[0], p2[1]-p0[1], p2[2]-p0[2]);//one vec
            let db = Vector3::new(p3[0]-p1[0], p3[1]-p1[1], p3[2]-p1[2]);//two vec
            let cp = (ca.cross(db)).normalize();//cross product of two diagonal vectors and normalise

            normals.push([cp[0], cp[1], cp[2]]);
            normals.push([cp[0], cp[1], cp[2]]);
            normals.push([cp[0], cp[1], cp[2]]);
            normals.push([cp[0], cp[1], cp[2]]);
            normals.push([cp[0], cp[1], cp[2]]);
            normals.push([cp[0], cp[1], cp[2]]);
        }
    }
    normals
}

pub fn simple_surface_positions(pts: &Vec<Vec<[f32; 3]>>, nx:usize, nz: usize) -> Vec<[f32;3]> {//accepts the points as input parameters from below
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity((4* (nx - 1)*(nz -1)) as usize);
    for i in 0..nx - 1 {
        for j in 0.. nz - 1 {
            //specify 4 points define a unit grade cell or
            let p0 = pts[i][j];
            let p1 = pts[i][j+1];
            let p2 = pts[i+1][j+1];
            let p3 = pts[i+1][j];
            //two triangles therefore 6 vertexes two triangles one square grid
            positions.push(p0);
            positions.push(p1);
            positions.push(p2);
            positions.push(p2);
            positions.push(p3);
            positions.push(p0);
        }
    }
    positions
}

pub fn simple_surface_points(f: &dyn Fn(f32, f32, f32) -> [f32; 3], xmin:f32, xmax:f32, zmin:f32, zmax:f32,
                             nx:usize, nz: usize, scale:f32, aspect:f32) -> (Vec<Vec<[f32; 3]>>, [f32; 2]) {

    let dx = (xmax-xmin)/(nx as f32-1.0);
    let dz = (zmax-zmin)/(nz as f32-1.0);
    let mut ymin: f32 = 0.0;
    let mut ymax: f32 = 0.0;
    //2D ARRAY NORMALISE THE POINT WITH FUNC
    let mut pts:Vec<Vec<[f32; 3]>> = vec![vec![Default::default(); nz]; nx];
    let mut intcentern = 58;
    let mut intcentere = 5;
    let data: Tile = Tile::from_file("src/Scotlandhgt/N".to_owned() + &*intcentern.to_string() +"W00"+ &*intcentere.to_string() +".hgt").unwrap();
    //intcentern -=1;
    intcentere=4;
    let data2: Tile = Tile::from_file("src/Scotlandhgt/N".to_owned() + &*intcentern.to_string() +"W00"+ &*intcentere.to_string() +".hgt").unwrap();
    //intcentern +=1;
    //intcentere -=1;
    intcentere=3;
    let data3: Tile = Tile::from_file("src/Scotlandhgt/N".to_owned() + &*intcentern.to_string() +"W00"+ &*intcentere.to_string() +".hgt").unwrap();
    //intcentern -=1;
    intcentern=57;
    intcentere=5;
    let data4: Tile = Tile::from_file("src/Scotlandhgt/N".to_owned() + &*intcentern.to_string() +"W00"+ &*intcentere.to_string() +".hgt").unwrap();
    intcentere=4;
    let data5: Tile = Tile::from_file("src/Scotlandhgt/N".to_owned() + &*intcentern.to_string() +"W00"+ &*intcentere.to_string() +".hgt").unwrap();
    intcentere=3;
    let data6: Tile = Tile::from_file("src/Scotlandhgt/N".to_owned() + &*intcentern.to_string() +"W00"+ &*intcentere.to_string() +".hgt").unwrap();
    intcentern=56;
    intcentere=5;
    let data7: Tile = Tile::from_file("src/Scotlandhgt/N".to_owned() + &*intcentern.to_string() +"W00"+ &*intcentere.to_string() +".hgt").unwrap();
    intcentere=4;

    let data8: Tile = Tile::from_file("src/Scotlandhgt/N".to_owned() + &*intcentern.to_string() +"W00"+ &*intcentere.to_string() +".hgt").unwrap();
    intcentere=3;
    let data9: Tile = Tile::from_file("src/Scotlandhgt/N".to_owned() + &*intcentern.to_string() +"W00"+ &*intcentere.to_string() +".hgt").unwrap();
    for i in 0..nx {//Add x div 2 to get more detailed x to have all hgt rather thsn half
        let x = xmin + i as f32 * dx;
        let mut pt1:Vec<[f32; 3]> = Vec::with_capacity(nz);
        for j in 0..nz {
            let z = zmin + j as f32 * dz;
            match z{
                0.0 ..=3600.0 =>{match x {
                    0.0 ..=3600.0 => {
                        let y:f32 = (Tile::get(&data, x as u32, z as u32)) as f32;
                        let pt = f(x, z, y);
                        pt1.push(pt);
                        ymin = if pt[1] < ymin { pt[1] } else { ymin };
                        ymax = if pt[1] > ymax { pt[1] } else { ymax };
                    }
                    3600.1 ..= 7200.0=>{
                        let xnow = x -3600.0;
                        let y:f32 = (Tile::get(&data2, xnow as u32, z as u32)) as f32;
                        let pt = f(x, z, y);
                        pt1.push(pt);
                        ymin = if pt[1] < ymin { pt[1] } else { ymin };
                        ymax = if pt[1] > ymax { pt[1] } else { ymax };
                    }
                    7200.1..=10800.0 => {
                        let xnow = x -7200.0;
                        let y:f32 = (Tile::get(&data3, xnow as u32, z as u32)) as f32;
                        let pt = f(x, z, y);
                        pt1.push(pt);
                        ymin = if pt[1] < ymin { pt[1] } else { ymin };
                        ymax = if pt[1] > ymax { pt[1] } else { ymax };

                    }
                    _ => {}
                }}
                3600.1 ..= 7200.0=>{let znow = z-3600.0;
                    match x {
                    0.0 ..=3600.0 => {
                        let y:f32 = (srtm::Tile::get(&data4, x as u32, znow as u32)) as f32;
                        let pt = f(x, z, y);
                        pt1.push(pt);
                        ymin = if pt[1] < ymin { pt[1] } else { ymin };
                        ymax = if pt[1] > ymax { pt[1] } else { ymax };
                    }
                    3600.1 ..= 7200.0=>{
                        let xnow = x -3600.0;
                        let y:f32 = (srtm::Tile::get(&data5, xnow as u32, znow as u32)) as f32;
                        let pt = f(x, z, y);
                        pt1.push(pt);
                        ymin = if pt[1] < ymin { pt[1] } else { ymin };
                        ymax = if pt[1] > ymax { pt[1] } else { ymax };
                    }
                    7200.1..=10800.0 => {
                        let xnow = x -7200.0;
                        let y:f32 = (srtm::Tile::get(&data6, xnow as u32, znow as u32)) as f32;
                        let pt = f(x, z, y);
                        pt1.push(pt);
                        ymin = if pt[1] < ymin { pt[1] } else { ymin };
                        ymax = if pt[1] > ymax { pt[1] } else { ymax };

                    }
                    _ => {}
                }}
                7200.1..=10800.0 => {let znow = z-7200.0;
                    match x {
                    0.0 ..=3600.0 => {
                        let y:f32 = (srtm::Tile::get(&data7, x as u32, znow as u32)) as f32;
                        let pt = f(x, z, y);
                        pt1.push(pt);
                        ymin = if pt[1] < ymin { pt[1] } else { ymin };
                        ymax = if pt[1] > ymax { pt[1] } else { ymax };
                    }
                    3600.1 ..= 7200.0=>{
                        let xnow = x -3600.0;
                        let y:f32 = (srtm::Tile::get(&data8, xnow as u32, znow as u32)) as f32;
                        let pt = f(x, z, y);
                        pt1.push(pt);
                        ymin = if pt[1] < ymin { pt[1] } else { ymin };
                        ymax = if pt[1] > ymax { pt[1] } else { ymax };
                    }
                    7200.1..=10800.0 => {
                        let xnow = x -7200.0;
                        let y:f32 = (srtm::Tile::get(&data9, xnow as u32, znow as u32)) as f32;
                        let pt = f(x, z, y);
                        pt1.push(pt);
                        ymin = if pt[1] < ymin { pt[1] } else { ymin };
                        ymax = if pt[1] > ymax { pt[1] } else { ymax };

                    }
                    _ => {}
                }}
                _ => {}
            }
           /* if z <= zmax/3.0 && x <= xmax/3.0{
                let y:f32 = (srtm::Tile::get(&data, x as u32, z as u32)) as f32;
                let pt = f(x, z, y);
                pt1.push(pt);
                ymin = if pt[1] < ymin { pt[1] } else { ymin };
                ymax = if pt[1] > ymax { pt[1] } else { ymax };
            }
            if z <= (zmax/3.0)*2.0 && z > zmax/3.0 && x <= xmax/3.0{
                let znow = z -3600.0;
                let y:f32 = (srtm::Tile::get(&data2, x as u32, znow as u32)) as f32;
                let pt = f(x, z, y);
                pt1.push(pt);
                ymin = if pt[1] < ymin { pt[1] } else { ymin };
                ymax = if pt[1] > ymax { pt[1] } else { ymax };
            }
            if z > (zmax/3.0)*2.0 && z > zmax/3.0 && x <= xmax/3.0{
                let znow = z -7200.0;
                let y:f32 = (srtm::Tile::get(&data2, x as u32, znow as u32)) as f32;
                let pt = f(x, z, y);
                pt1.push(pt);
                ymin = if pt[1] < ymin { pt[1] } else { ymin };
                ymax = if pt[1] > ymax { pt[1] } else { ymax };
            }
            if z <= zmax/3.0 && x > xmax/3.0 && x <= (xmax/3.0)*2{
                let y:f32 = (srtm::Tile::get(&data, x as u32, z as u32)) as f32;
                let pt = f(x, z, y);
                pt1.push(pt);
                ymin = if pt[1] < ymin { pt[1] } else { ymin };
                ymax = if pt[1] > ymax { pt[1] } else { ymax };
            }
            if z <= (zmax/3.0)*2.0 && z > zmax/3.0 && x <= xmax/3.0 && x <= (xmax/3.0)*2{
                let znow = z -3600.0;
                let y:f32 = (srtm::Tile::get(&data2, x as u32, znow as u32)) as f32;
                let pt = f(x, z, y);
                pt1.push(pt);
                ymin = if pt[1] < ymin { pt[1] } else { ymin };
                ymax = if pt[1] > ymax { pt[1] } else { ymax };
            }
            if z > (zmax/3.0)*2.0 && z > zmax/3.0 && x <= xmax/3.0{
                let znow = z -7200.0;
                let y:f32 = (srtm::Tile::get(&data2, x as u32, znow as u32)) as f32;
                let pt = f(x, z, y);
                pt1.push(pt);
                ymin = if pt[1] < ymin { pt[1] } else { ymin };
                ymax = if pt[1] > ymax { pt[1] } else { ymax };
            }
            if z <= zmax/2.0 && x > xmax/2.0 {
                let xnow = x -3600.0;
                let y:f32 = (srtm::Tile::get(&data3, xnow as u32, z as u32)) as f32;
                let pt = f(x, z, y);
                pt1.push(pt);
                ymin = if pt[1] < ymin { pt[1] } else { ymin };
                ymax = if pt[1] > ymax { pt[1] } else { ymax };
            }
            if z > zmax/2.0 && x > xmax/2.0{
                let xnow = x -3600.0;
                let znow = z -3600.0;
                let y:f32 = (srtm::Tile::get(&data4, xnow as u32, znow as u32)) as f32;
                let pt = f(x, z, y);
                pt1.push(pt);
                ymin = if pt[1] < ymin { pt[1] } else { ymin };
                ymax = if pt[1] > ymax { pt[1] } else { ymax };
            }
            */

        }
        pts[i] = pt1;
    }

    let ymin1 = ymin - (1.0 - aspect) * (ymax - ymin);
    let ymax1 = ymax + (1.0 - aspect) * (ymax - ymin);

    for i in 0..nx {
        for j in 0..nz {
            pts[i][j] = normalize_point(pts[i][j], xmin, xmax, ymin1, ymax1, zmin, zmax, scale);
        }
    }
    //add colormap to the y value
    let cmin = normalize_point([0.0, ymin, 0.0], xmin, xmax, ymin1, ymax1, zmin, zmax, scale)[1];
    let cmax = normalize_point([0.0, ymax, 0.0], xmin, xmax, ymin1, ymax1, zmin, zmax, scale)[1];

    return (pts, [cmin, cmax]);//returns points and colors with them
}

// private function allows for the normalisation of a 3d point into the region of -1 to 1
fn normalize_point(pt:[f32;3], xmin:f32, xmax:f32, ymin:f32, ymax:f32, zmin:f32, zmax:f32, scale:f32) -> [f32;3] {//scale => size of the surfaces
    let px = scale * (-1.0 + 2.0 * (pt[0] - xmin) / (xmax - xmin));
    let py = scale * (-1.0 + 2.0 * (pt[1] - ymin) / (ymax - ymin));
    let pz = scale * (-1.0 + 2.0 * (pt[2] - zmin) / (zmax - zmin));
    [px, py, pz]
}