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
trait Defaultable {
    fn default_with_params(lat: u32,long:u32) -> Self;
}
struct Threaded {
    pub refer: std::sync::mpsc::Receiver<Vec<Vec<f32>>>,
    pub thread:JoinHandle<()>,
}
impl Defaultable for Threaded {
    fn default_with_params(lat:u32,long:u32) -> Self {
        let (tx, rx) = mpsc::channel();
        let thread = thread::spawn(move||{
            let mut map :Vec<Vec<f32>> = vec![];
            let mut height_min = f32::MAX;
            let mut height_max = f32::MIN;
            let worldmap: Tile = Tile::from_file("src/Scotlandhgt/N".to_owned() + &*lat.to_string() +"W00"+ &*long.to_string() +".hgt").unwrap_or(Tile::from_file("src/Scotlandhgt/N00W000.hgt").unwrap());

            for x in 0..3600{
                let mut p1:Vec<f32> = vec![];
                for z in 0..3600{
                    let y =  Tile::get(&worldmap, x as u32, z as u32) as f32;
                    height_min = if y  < height_min { y } else { height_min };
                    height_max = if y  > height_max { y } else { height_max };
                    p1.push(y);

                }
                map.push(p1);
            }

            for x in 0..3600 as usize {
                for z in 0..3600 as usize {
                    map[x][z] = (map[x][z] as f32 - height_min)/(height_max - height_min);
                }
            }
            tx.send(map).unwrap();
        });
        Self {
            refer: rx,
            thread: thread,
        }
    }
}

impl Threaded {
    fn transferwithret(&mut self, lat:u32,long:u32){
        let transfer = Threaded::default_with_params(lat,long);
        self.thread=transfer.thread;
        self.refer=transfer.refer;
    }
}


pub struct Terrain {
    pub offsets: [f32; 2],
    pub moves: [f32; 2],
    pub back: [i32; 2],
    pub level_of_detail: u32,
    pub water_level: f32,
    mapdata: Vec<Vec<f32>>,
    mapdatanextx: Vec<Vec<f32>>,
    mapdatanextz: Vec<Vec<f32>>,
    mapdatanextxz: Vec<Vec<f32>>,
    doneinit :u32,
    doneinitx :u32,
    doneinitz :u32,
    doneinitxz: u32,
    donexe :u32,
    donexw :u32,
    donezn :u32,
    donezs :u32,
    north: bool,
    east: bool,
    south: bool,
    west: bool,
    pub lat :u32,
    pub long :u32,
    pub chunksize:u32,
    nthread: Threaded,
    ethread: Threaded,
    sthread: Threaded,
    wthread: Threaded,
    nethread: Threaded,
    esthread: Threaded,
    swthread: Threaded,
    wnthread: Threaded,

}

impl Default for Terrain {
        fn default() -> Self {
            let mut lat =54;
            let mut long = 3;
            lat +=1;
            let norththread = Threaded::default_with_params(lat,long);
            long -=1;
            let northeastthread = Threaded::default_with_params(lat,long);
            lat -=1;
            let eastthread = Threaded::default_with_params(lat,long);
            lat -=1;
            let eastsouththread = Threaded::default_with_params(lat,long);
            long +=1;
            let souththread = Threaded::default_with_params(lat,long);
            long +=1;
            let southwestthread = Threaded::default_with_params(lat,long);
            lat +=1;
            let westthread = Threaded::default_with_params(lat,long);
            lat +=1;
            let westnorththread = Threaded::default_with_params(lat,long);
        Self {
            offsets: [0.0, 0.0],
            moves:[1800.0,1800.0],
            back:[0,0],
            level_of_detail: 5,
            water_level: 0.1,
            mapdata: vec![],
            mapdatanextx: vec![],
            mapdatanextz: vec![],
            mapdatanextxz: vec![],
            doneinit:0,
            doneinitx :0,
            doneinitz :0,
            doneinitxz :0,
            donexe:0,
            donexw:0,
            donezn:0,
            donezs:0,
            north: false,
            east: false,
            south: false,
            west: false,
            chunksize:241,
            lat:54,
            long:3,
            nthread: norththread,
            ethread: eastthread,
            sthread: souththread,
            wthread: westthread,
            nethread: northeastthread,
            esthread: eastsouththread,
            swthread: southwestthread,
            wnthread: westnorththread,
        }
    }
}

impl Terrain {

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
        let worldmap: Tile =  Tile::from_file("src/Scotlandhgt/N".to_owned() + &*self.lat.to_string() +"W00"+ &*self.long.to_string() +".hgt").unwrap_or(Tile::from_file("src/Scotlandhgt/N00W000.hgt").unwrap());

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

        if self.doneinit == 0 {
            self.find_world_map();
            self.doneinit +=1
        }


        let mut data:Vec<Vertex> = vec![];

        for x in (0..self.chunksize as usize).step_by(increment_count as usize) {
            for z in (0..self.chunksize as usize).step_by(increment_count as usize) {
                let usex : i32= (x as f32 + self.offsets[0] + self.moves[0])as i32;
                let usez : i32= (z as f32 + self.offsets[1] + self.moves[1])as i32;
                let mut y = 0.0;
                match usez as i32 {
                    m if m < -1800 =>{
                        println!("chunk");
                            self.moves[1] += 3600.0;
                            self.mapdata = vec![];
                            self.mapdata = self.mapdatanextz.clone();
                            self.lat += 1;
                            self.doneinitz = 2;
                            self.donezn = 0;
                            self.north= true;
                    }
                    -1800 ..=-1 => {
                        self.north = true;

                    }
                     0 ..=200 => {
                        if self.donezn==0 {
                            for received in &self.nthread.refer {
                                self.mapdatanextz = vec![];
                                self.mapdatanextz = received
                            }

                            self.donezn += 1;
                        }
                    }
                    201 ..=3400 => {
                        self.doneinitxz = 0;
                        if self.doneinitz ==2 {
                                Threaded::transferwithret(&mut self.sthread,self.lat-1,self.long);
                                Threaded::transferwithret(&mut self.nthread,self.lat+1,self.long);

                            self.doneinitz +=1;
                        }
                    }
                    3401 ..= 3599=>{
                        if self.donezs == 0 {
                            for received in &self.sthread.refer {
                                self.mapdatanextz = vec![];
                                self.mapdatanextz = received
                            }
                            self.donezs += 1;
                        }
                    }
                    3600 ..= 5400=>{
                        self.south= true;
                    }
                    _ => {
                         println!("chunk");
                        self.moves[1] -=3600.0;
                        self.mapdata = vec![];
                        self.mapdata = self.mapdatanextz.clone();
                        self.lat -= 1;
                        self.doneinitz=2;
                        self.donezs=0;
                        self.south= true;

                }}
                match usex as i32 {
                    n if n < -1800 =>{
                        println!("chunk");
                        self.moves[0] += 3600.0;
                        self.mapdata = vec![];
                        self.mapdata = self.mapdatanextx.clone();
                        self.long +=1;
                        self.doneinitx=2;
                        self.donexw=0;
                        self.west = true
                    }
                    -1800 ..=-1 => {
                        self.west = true;
                    }
                     0 ..=200 => {
                        if self.donexw==0 {
                            for received in &self.wthread.refer {
                                self.mapdatanextx = vec![];
                                self.mapdatanextx = received
                            }

                            self.donexw += 1;
                        }
                    }
                    201 ..=3400 =>{
                        self.doneinitxz = 0;
                        if self.doneinitx ==2 {
                            Threaded::transferwithret(&mut self.ethread,self.lat,self.long-1);
                            Threaded::transferwithret(&mut self.wthread,self.lat,self.long+1);

                            self.doneinitx +=1;
                        }
                    }
                    3401 ..= 3599=>{
                        if self.donexe == 0 {
                            for received in &self.ethread.refer {
                                self.mapdatanextx = vec![];
                                self.mapdatanextx = received
                            }
                            self.donexe += 1;
                        }
                    }
                    3600 ..= 5400=>{
                        self.east =true;
                    }
                    _ => {
                        println!("chunk");
                        self.moves[0] -=3600.0;
                        self.mapdata = vec![];
                        self.mapdata = self.mapdatanextx.clone();
                        if self.long >1 {
                            self.long -= 1;
                        }
                        self.doneinitx=2;
                        self.donexe=0;
                        self.east =true;

                }}

                if self.south || self.north || self.east || self.west {
                    if self.south {
                        if self.east{if self.doneinitxz ==0 {
                            for received in &self.esthread.refer {
                                self.mapdatanextxz = vec![];
                                self.mapdatanextxz = received
                            }

                            self.doneinitx +=1;
                        }
                            y = self.mapdatanextxz[(usex -3600) as usize][(usez - 3600) as usize];
                        }else if self.west{
                            if self.doneinitxz ==0 {
                            for received in &self.swthread.refer {
                                self.mapdatanextxz = vec![];
                                self.mapdatanextxz = received
                            }

                            self.doneinitx +=1;
                        }
                            y = self.mapdatanextxz[(usex +3600) as usize][(usez - 3600) as usize];
                        }else{
                            y = self.mapdatanextz[usex as usize][(usez - 3600) as usize];
                        }
                    }
                    if self.north {
                        if self.east{
                            if self.doneinitxz ==0 {
                            for received in &self.nethread.refer {
                                self.mapdatanextxz = vec![];
                                self.mapdatanextxz = received
                            }

                            self.doneinitx +=1;
                        }
                            y = self.mapdatanextxz[(usex - 3600) as usize][(usez + 3600) as usize];
                        }else if self.west{
                            if self.doneinitxz ==0 {
                            for received in &self.wnthread.refer {
                                self.mapdatanextxz = vec![];
                                self.mapdatanextxz = received
                            }

                            self.doneinitx +=1;
                        }
                            y = self.mapdatanextxz[(usex +3600) as usize][(usez + 3600) as usize];
                        }else{
                            y = self.mapdatanextz[usex as usize][(usez + 3600) as usize];
                        }
                    }
                    if self.east && y==0.0{
                        y = self.mapdatanextx[(usex - 3600) as usize][usez as usize];
                    }
                    if self.west && y==0.0{
                        y = self.mapdatanextx[(usex + 3600) as usize][usez as usize];
                    }
                }else{
                    y = self.mapdata[usex as usize][usez as usize];
                }
                self.north = false;
                self.east = false;
                self.south = false;
                self.west = false;
                if y < self.water_level {
                    y = self.water_level - 0.01;
                }
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
            //self.level_of_detail=5;
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
