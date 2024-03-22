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
    pub octaves: u32,
    pub persistence: f32,
    pub lacunarity: f32,
    pub offsets: [f32; 2],
    pub scale: f32,
    pub colormap_name: String,
    pub chunk_size: u32,
    pub level_of_detail: u32,
    pub normalize_mode: String,
}

impl Default for ITerrain {
    fn default() -> Self {
        Self {
            width: 3600,
            height: 3600,
            octaves: 5,
            persistence: 0.5,
            lacunarity: 2.0,
            offsets: [0.0, 0.0],
            scale: 10.0,
            colormap_name: "mountain".to_string(),
            chunk_size: 241,
            level_of_detail: 0,
            normalize_mode: "local".to_string(),
        }
    }
}

impl ITerrain {
    pub fn new() -> Self {
        Default::default()
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
    pub fn find_world_map(&mut self, width: u32, height: u32) -> Vec<Vec<f32>>{
        let mut map :Vec<Vec<f32>> = vec![];
        let mut height_min = f32::MAX;
        let mut height_max = f32::MIN;
        let worldmap: Tile = Tile::from_file("src/S05E024.hgt").unwrap();
        for x in 0..width {
            let mut p1:Vec<f32> = vec![];
            for z in 0..height {
                let usex = x as f32 + self.offsets[0];
                let usez = z as f32 + self.offsets[1];
                let y =  Tile::get(&worldmap, usex as u32, usez as u32) as f32;
                height_min = if y < height_min { y } else { height_min };
                height_max = if y > height_max { y } else { height_max };
                p1.push(y);
            }
            map.push(p1);
        }
        if self.normalize_mode == "global" {
            height_min = -1.0;
            height_max = 1.0;
        }

        for x in 0..width as usize {
            for z in 0..height as usize {
                map[x][z] = (map[x][z] - height_min)/(height_max - height_min);
            }
        }

        map
    }

    pub fn create_terrain_data(&mut self) -> Vec<Vertex> {
        let cdata = colormap::colormap_data(&self.colormap_name);
        let world_map = self.find_world_map(self.width, self.height);

        let mut data:Vec<Vertex> = vec![];

        for x in 0..self.width as usize {
            for z in 0..self.height as usize {
                let y = world_map[x][z] ;
                //let y=0.0;
                let position = [x as f32, y, z as f32];
                let color = colormap::color_interp(cdata, 0.0, 1.0, y);

                data.push(Vertex { position, color });
            }
        }
        data
    }

    fn terrian_colormap_data(&mut self) -> (Vec<[f32; 3]>, Vec<f32>) {
        let cdata = vec![
            [0.055f32, 0.529, 0.8],
            [0.761, 0.698, 0.502],
            [0.204, 0.549, 0.192],
            [0.353, 0.302, 0.255],
            [1.0, 0.98, 0.98]
        ];
        let ta = vec![0.0f32, 0.3, 0.35, 0.6, 0.9, 1.0];
        (cdata, ta)
    }


    fn color_lerp(&mut self, color:&Vec<[f32;3]>, ta:&Vec<f32>, t:f32) -> [f32;3] {
        let len = 6usize;
        let mut res = [0f32;3];
        for i in 0..len - 1 {
            if t >= ta[i] && t < ta[i + 1] {
                res = color[i];
            }
        }
        if t == ta[len-1] {
            res = color[len-2];
        }
        res
    }

    fn add_terrain_colors(&mut self, color:&Vec<[f32;3]>, ta:&Vec<f32>, tmin:f32, tmax:f32, t:f32) -> [f32;3] {
        let mut tt = if t < tmin { tmin } else if t > tmax { tmax } else { t };
        tt = (tt - tmin)/(tmax - tmin);

        self.color_lerp(color, ta, tt)
    }



    pub fn create_terrain_data_chunk(&mut self) -> (Vec<Vertex>, u32){
        let increment_count = if self.level_of_detail <= 5 { self.level_of_detail + 1} else { 2*(self.level_of_detail - 2)};

        let vertices_per_row = (self.chunk_size - 1)/increment_count + 1;

        let (cdata, ta) = self.terrian_colormap_data();



        let mut data:Vec<Vertex> = vec![];
        let world_map=self.find_world_map(self.chunk_size, self.chunk_size);
        for x in (0..self.chunk_size as usize).step_by(increment_count as usize) {
            for z in (0..self.chunk_size as usize).step_by(increment_count as usize) {
                let  y = world_map[x][z] ;

                let position = [x as f32, y, z as f32];
                let color = self.add_terrain_colors(&cdata, &ta, 0.0, 1.0, y);

                data.push(Vertex { position, color });
            }
        }
        (data, vertices_per_row)
    }


    pub fn create_terrain_data_multiple_chunks(&mut self, x_chunks:u32, z_chunks:u32, translations:&Vec<[f32;2]>)
                                               -> (Vec<Vec<Vertex>>, u32) {
        let mut data:Vec<Vec<Vertex>> = vec![];
        let mut vertices_per_row = 0u32;

        let mut k:u32 = 0;
        for _i in 0..x_chunks {
            for _j in 0..z_chunks {
                self.offsets = translations[k as usize];
                let dd = self.create_terrain_data_chunk();
                data.push(dd.0);
                vertices_per_row = dd.1;
                k += 1;
            }
        }
        (data, vertices_per_row)
    }
}