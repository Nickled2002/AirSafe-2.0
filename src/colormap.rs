#![allow(dead_code)]
pub fn color_interp(colormap_name: &str, min:f32, max:f32, mut t:f32) -> [f32; 3]{
    if t < min {
        t = min;
    }
    if t > max {
        t = max;

    }
    let tn = (t-min)/(max - min);
    let colors = colormap_data(colormap_name);
    let indx = (10.0 * tn).floor() as usize;

    if indx as f32 == 10.0 * tn {
        colors[indx]
    } else {
        let tn1 = (tn - 0.1 * indx as f32)*10.0; // rescale
        let a = colors[indx];
        let b = colors[indx+1];
        let color_r = a[0] + (b[0] - a[0]) * tn1;
        let color_g = a[1] + (b[1] - a[1]) * tn1;
        let color_b = a[2] + (b[2] - a[2]) * tn1;
        [color_r, color_g, color_b]
    }
}

pub fn colormap_data(colormap_name: &str) -> [[f32; 3]; 11] {
    let colors = match colormap_name {

        "mountain" => [[0.0,0.0,1.0],[0.0,0.0,1.0],[0.0,0.98,1.0],[0.0,0.98,1.0],[0.7,0.7,0.5],[0.0,1.0,0.0],
            [0.25,0.16,0.1],[0.25,0.16,0.1],[0.25,0.16,0.1],[1.0,1.0,1.0],[1.0,1.0,1.0]],

        // "jet" as default
        _ => [[0.0,0.0,0.51],[0.0,0.24,0.67],[0.01,0.49,0.78],[0.01,0.75,0.89],[0.02,1.0,1.0],
            [0.51,1.0,0.5],[1.0,1.0,0.0],[0.99,0.67,0.0],[0.99,0.33,0.0],[0.98,0.0,0.0],[0.5,0.0,0.0]],
    };

    colors
}