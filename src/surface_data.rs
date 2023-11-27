#![allow(dead_code)]
use cgmath::*;
mod colormap;
// code more efficient only requiring two for loops
pub fn simple_surface_colors(pts: &Vec<Vec<[f32; 3]>>, nx:usize, nz: usize, yrange:[f32; 2], colormap_name: &str) -> Vec<[f32; 3]> {
    let mut colors: Vec<[f32; 3]> = Vec::with_capacity((4* (nx - 1)*(nz -1)) as usize);
    for i in 0..nx - 1 {
        for j in 0.. nz - 1 {
            let p0 = pts[i][j];
            let p1 = pts[i][j+1];
            let p2 = pts[i+1][j+1];
            let p3 = pts[i+1][j];

            let c0 = colormap::color_interp(colormap_name, yrange[0], yrange[1], p0[1]);
            let c1 = colormap::color_interp(colormap_name, yrange[0], yrange[1], p1[1]);
            let c2 = colormap::color_interp(colormap_name, yrange[0], yrange[1], p2[1]);
            let c3 = colormap::color_interp(colormap_name, yrange[0], yrange[1], p3[1]);

            colors.push(c0);//add color map to vertixes
            colors.push(c1);
            colors.push(c2);
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
            let p0 = pts[i][j];
            let p1 = pts[i][j+1];
            let p2 = pts[i+1][j+1];
            let p3 = pts[i+1][j];

            let ca = Vector3::new(p2[0]-p0[0], p2[1]-p0[1], p2[2]-p0[2]);
            let db = Vector3::new(p3[0]-p1[0], p3[1]-p1[1], p3[2]-p1[2]);
            let cp = (ca.cross(db)).normalize();//normal vector

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

pub fn simple_surface_positions(pts: &Vec<Vec<[f32; 3]>>, nx:usize, nz: usize) -> Vec<[f32;3]> {//accepts the points as input parameters
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity((4* (nx - 1)*(nz -1)) as usize);
    for i in 0..nx - 1 {
        for j in 0.. nz - 1 {
            let p0 = pts[i][j];
            let p1 = pts[i][j+1];
            let p2 = pts[i+1][j+1];
            let p3 = pts[i+1][j];

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

pub fn simple_surface_points(f: &dyn Fn(f32, f32) -> [f32; 3], xmin:f32, xmax:f32, zmin:f32, zmax:f32,
                             nx:usize, nz: usize, scale:f32, aspect:f32) -> (Vec<Vec<[f32; 3]>>, [f32; 2]) {

    let dx = (xmax-xmin)/(nx as f32-1.0);
    let dz = (zmax-zmin)/(nz as f32-1.0);
    let mut ymin: f32 = 0.0;
    let mut ymax: f32 = 0.0;

    let mut pts:Vec<Vec<[f32; 3]>> = vec![vec![Default::default(); nz]; nx];
    for i in 0..nx {
        let x = xmin + i as f32 * dx;
        let mut pt1:Vec<[f32; 3]> = Vec::with_capacity(nz);
        for j in 0..nz {
            let z = zmin + j as f32 * dz;
            let pt = f(x, z);
            pt1.push(pt);
            ymin = if pt[1] < ymin { pt[1] } else { ymin };
            ymax = if pt[1] > ymax { pt[1] } else { ymax };
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

    return (pts, [cmin, cmax]);
}

fn normalize_point(pt:[f32;3], xmin:f32, xmax:f32, ymin:f32, ymax:f32, zmin:f32, zmax:f32, scale:f32) -> [f32;3] {//normalise 3d points into region 1 and -1
    let px = scale * (-1.0 + 2.0 * (pt[0] - xmin) / (xmax - xmin));
    let py = scale * (-1.0 + 2.0 * (pt[1] - ymin) / (ymax - ymin));
    let pz = scale * (-1.0 + 2.0 * (pt[2] - zmin) / (zmax - zmin));
    [px, py, pz]
}