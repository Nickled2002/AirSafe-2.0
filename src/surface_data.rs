#![allow(dead_code)]
use bytemuck:: {Pod, Zeroable};
use srtm::Tile;
use std::thread;
use std::sync::mpsc;
use std::thread::JoinHandle;
//mod colormap;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}
struct threaded {
        pub joining: JoinHandle<Vec<Vec<f32>>>,
}
impl Default for threaded {
    fn default() -> Self {
        let thread = thread::spawn(move||->Vec<Vec<f32>>{
                    let mut map :Vec<Vec<f32>> = vec![];
                    let mut height_min = f32::MAX;
                    let mut height_max = f32::MIN;
                    let worldmap: Tile = Tile::from_file("src/Scotlandhgt/N00W000.hgt").unwrap();

                    for x in 0..3600 {
                        let mut p1:Vec<f32> = vec![];
                        for z in 0..3600 {
                            let y =  Tile::get(&worldmap, x as u32, z as u32) as f32;
                            height_min = if y < height_min { y } else { height_min };
                            height_max = if y > height_max { y } else { height_max };
                            p1.push(y);
                        }
                    map.push(p1);
                    }

                    for x in 0..3600 as usize {
                        for z in 0..3600 as usize {
                            map[x][z] = (map[x][z] - height_min)/(height_max - height_min);
                        }
                    }
                    //tx.send(map).unwrap();
                    map
                });
        Self {
        joining: thread,
        }
    }
}
pub struct ITerrain {
    pub offsets: [f32; 2],
    pub moves: [f32; 2],
    pub level_of_detail: u32,
    pub water_level: f32,
    pub mapdata: Vec<Vec<f32>>,
    pub mapdata2: Vec<Vec<f32>>,
    pub done1 :u32,
    pub done2 :u32,
    pub chunksize:u32,
    pub refer: std::sync::mpsc::Receiver<Vec<Vec<f32>>>,

}

impl Default for ITerrain {
        fn default() -> Self {
        let (tx, rx) = mpsc::channel::<Vec<Vec<f32>>>();
        Self {
            offsets: [0.0, 0.0],
            moves:[600.0,600.0],
            level_of_detail: 5,
            water_level: 0.1,
            mapdata: vec![],
            mapdata2: vec![],
            done1:0,
            done2:0,
            chunksize:241,
            refer:rx,
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
        let worldmap: Tile = Tile::from_file("src/Scotlandhgt/N55W005.hgt").unwrap();

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
    pub fn find_another_world_map(&mut self)->JoinHandle<Vec<Vec<f32>>> {
        let (tx, rx) = mpsc::channel::<Vec<Vec<f32>>>();
        let thread = thread::spawn(move||->Vec<Vec<f32>>{
                    let mut map :Vec<Vec<f32>> = vec![];
                    let mut height_min = f32::MAX;
                    let mut height_max = f32::MIN;
                    let worldmap: Tile = Tile::from_file("src/Scotlandhgt/N00W000.hgt").unwrap();

                    for x in 0..3600 {
                        let mut p1:Vec<f32> = vec![];
                        for z in 0..3600 {
                            let y =  Tile::get(&worldmap, x as u32, z as u32) as f32;
                            height_min = if y < height_min { y } else { height_min };
                            height_max = if y > height_max { y } else { height_max };
                            p1.push(y);
                        }
                    map.push(p1);
                    }

                    for x in 0..3600 as usize {
                        for z in 0..3600 as usize {
                            map[x][z] = (map[x][z] - height_min)/(height_max - height_min);
                        }
                    }
                    //tx.send(map).unwrap();
                    map
                });
        self.refer=rx;
        thread
    }
    fn color_interp(&mut self, color:&Vec<[f32;3]>, ta:&Vec<f32>, t:f32) -> [f32;3] {
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
        let t1 = self.shift_water_level(ta);
        self.color_interp(color, &t1, tt)
    }
    fn shift_water_level(&mut self, ta:&Vec<f32>) -> Vec<f32> {
        let mut t1 = vec![0f32; 6];
        let r = (1.0 - self.water_level)/(1.0 - ta[1]);
        t1[1] = self.water_level;
        for i in 1..5usize {
            let del = ta[i+1] - ta[i];
            t1[i+1] = t1[i] + r * del;
        }
        t1
    }



    pub fn create_terrain_data(&mut self) -> (Vec<Vertex>, u32) {

        let increment_count = if self.level_of_detail <= 5 { self.level_of_detail + 1} else { 2*(self.level_of_detail - 2)};
        let vertices_per_row = (self.chunksize - 1)/increment_count + 1;
        let cdata =  vec![
            [0.055f32, 0.529, 0.8],
            [0.761, 0.698, 0.502],
            [0.204, 0.549, 0.192],
            [0.353, 0.302, 0.255],
            [1.0, 0.98, 0.98]
        ];
        let ta = vec![0.0f32, 0.3, 0.35, 0.6, 0.9, 1.0];

        if self.done1 == 0 {
            self.find_world_map();
            self.done1 +=1
        }


        let mut data:Vec<Vertex> = vec![];

        for x in (0..self.chunksize as usize).step_by(increment_count as usize) {
            for z in (0..self.chunksize as usize).step_by(increment_count as usize) {
                let mut usex = x as f32 + self.offsets[0] + self.moves[0];
                let mut usez = z as f32 + self.offsets[1] + self.moves[1];
                let mut y =0.0;


                match usex as usize {
                    0 ..=2600 => {

                        y = self.mapdata[usex as usize][usez as usize];
                    }
                    2601 ..= 4600=>{
                        //print!("{} ",usex);

                            //println!("imINasdvasdhsabdhasaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
                            //let mut my_thread = threaded::default();


                        if usex > 3599.0 {

                            if  usex == 3600.0 {
                                self.mapdata2 = my_thread.joining.join().unwrap();
                            }
                            usex -= 3600.0;
                            y = self.mapdata2[usex as usize][usez as usize];
                        }else{
                            y = self.mapdata[usex as usize][usez as usize];
                        }
                    }
                    _ => {}
                }
                
                if y < self.water_level {
                    y = self.water_level - 0.01;
                }
                //let y=0.0;
                let position = [x as f32, y, z as f32];
                let color = self.add_terrain_colors(&cdata, &ta, 0.0, 1.0, y);

                data.push(Vertex { position, color });
            }
        }
    (data,vertices_per_row)
    }
    pub fn create_collection_of_terrain_data(&mut self, x_chunks:u32, z_chunks:u32, translations:&Vec<[f32;2]>) -> (Vec<Vec<Vertex>>, u32) {
        let mut data:Vec<Vec<Vertex>> = vec![];
        let mut vertices_per_row = 0u32;

        let mut k:u32 = 0;
        for _i in 0..x_chunks {
            for _j in 0..z_chunks {
                self.offsets = translations[k as usize];
                let dd = self.create_terrain_data();
                data.push(dd.0);
                vertices_per_row = dd.1;
                k += 1;
            }
        }
        (data, vertices_per_row)
    }





}
