#![allow(dead_code)]
use bytemuck:: {Pod, Zeroable};
use srtm::Tile;

mod colormap;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

pub struct ITerrain {
    pub width: u32,
    pub height: u32,
    pub offsets: [f32; 2],
    pub colormap_name: String,
    pub level_of_detail: u32,
    pub mapdata: Vec<Vec<f32>>,
    pub mapdata2: Vec<Vec<f32>>,
    pub done :u32,
}

impl Default for ITerrain {

    fn default() -> Self {
        Self {
            width: 3600,
            height: 3600,
            offsets: [0.0, 0.0],
            colormap_name: "mountain".to_string(),
            level_of_detail: 1000,
            mapdata: vec![],
            mapdata2: vec![],
            done:0,
        }
    }
}

impl ITerrain {

    pub async fn new(&mut self) -> Self {
        Self::default()
    }


    pub fn create_indices(&mut self, width: u32, height: u32) -> Vec<u32> {
        let n_vertices_per_row = height;
        let mut indices:Vec<u32> = vec![];

        for i in 0..width - 1 {
            for j in 0..height - 1 {
                let idx0 = j + i * n_vertices_per_row;
                let idx1 = j + 1 + i * n_vertices_per_row;
                let idx2 = j + 1 + (i + 1) * n_vertices_per_row;
                let idx3 = j + (i + 1) * n_vertices_per_row;
                indices.extend([idx0, idx1, idx2, idx2, idx3, idx0]);
            }
        }
        indices
    }
    pub fn find_world_map(&mut self) {
        let mut height_min = f32::MAX;
        let mut height_max = f32::MIN;
        let worldmap: Tile = Tile::from_file("src/Scotlandhgt/N56W003.hgt").unwrap();

        for x in 0..3600 {
            let mut p1:Vec<f32> = vec![];
            for z in 0..3600 {
                let y =  Tile::get(&worldmap, x as u32, z as u32) as f32;
                height_min = if y < height_min { y } else { height_min };
                height_max = if y > height_max { y } else { height_max };
                p1.push(y);
            }
            self.mapdata.push(p1);
        }

        for x in 0..3600 as usize {
            for z in 0..3600 as usize {
                self.mapdata[x][z] = (self.mapdata[x][z] - height_min)/(height_max - height_min);
            }
        }
    }


    pub fn create_terrain_data(&mut self) -> Vec<Vertex> {
        let cdata = colormap::colormap_data(&self.colormap_name);

        if self.done == 0 {
            self.find_world_map();
            self.done +=1
        }
        let mut data:Vec<Vertex> = vec![];

        for x in 0..self.width as usize {
            for z in 0..self.height as usize {
                let usex = x as f32 + self.offsets[0];
                let usez = z as f32 + self.offsets[1];
                let y = self.mapdata[usex as usize][usez as usize] ;
                //let y=0.0;
                let position = [x as f32, y, z as f32];
                let color = colormap::color_interp(cdata, 0.0, 1.0, y);

                data.push(Vertex { position, color });
            }
        }
        data
    }





}
