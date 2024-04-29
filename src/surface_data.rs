use bytemuck:: {Pod, Zeroable};
use srtm::Tile;
use std::thread;
use std::sync::mpsc;
use std::thread::JoinHandle;
//mod colormap;
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {//Vertex struct containing color and position
    pub position: [f32; 3],
    pub color: [f32; 3],
}
trait Defaultable {//Default function for creating a thread  with latitude and longitude parameters
    fn default_with_params(lat: u32,long:u32,minimised:bool) -> Self;
}
struct Threaded {//Struct Threaded containing thread receiver and the thread itself
    pub refer: std::sync::mpsc::Receiver<Vec<Vec<f32>>>,
    pub thread:JoinHandle<()>,
}
impl Defaultable for Threaded {
    fn default_with_params(lat:u32,long:u32,minimised:bool) -> Self {//Calculating srtm tile based on lat and long and outputting in a vector of vectors
        let (tx, rx) = mpsc::channel();
        let thread = thread::spawn(move||{
            let mut map :Vec<Vec<f32>> = vec![];
            let mut height_min = f32::MAX;
            let mut height_max = f32::MIN;
            let worldmap: Tile = Tile::from_file("src/Scotlandhgt/N".to_owned() + &*lat.to_string() +"W00"+ &*long.to_string() +".hgt").unwrap_or(Tile::from_file("src/Scotlandhgt/N00W000.hgt").unwrap());
            let mut total = 3600;
            if minimised == true{
                total /=4
            }
            for x in 0..total{
                let mut p1:Vec<f32> = vec![];
                for z in 0..total{
                    let y =  Tile::get(&worldmap, x as u32, z as u32) as f32;
                    height_min = if y  < height_min { y } else { height_min };
                    height_max = if y  > height_max { y } else { height_max };
                    p1.push(y);

                }
                map.push(p1);
            }

            for x in 0..total as usize {//normalisation on the entie srtm tile
                for z in 0..total as usize {
                    map[x][z] = (map[x][z] as f32 )/(height_max - height_min);
                }
            }
            tx.send(map).unwrap();//Send data to the receiver
        });
        Self {
            refer: rx,
            thread: thread,
        }
    }
}

impl Threaded {
    fn transferwithret(&mut self, lat:u32,long:u32,min:bool){
        //Transfering ownership from an old thread to a new thread clears memory
        let transfer = Threaded::default_with_params(lat,long,min);
        self.thread=transfer.thread;
        self.refer=transfer.refer;
    }
}


pub struct Terrain {//Public Terrain struct
    pub offsets: [f32; 2],//Chunk offsets
    pub moves: [f32; 2],//moving by the keyboard input
    pub level_of_detail: u32,//varrying level of detail higher level_of_detail larger increments of rendering lower render quality
    pub water_level: f32,
    //srtm tile adapted to vectors of vectors
    mapdata: Vec<Vec<f32>>,
    //loading new srtm tiles
    mapdatanextx: Vec<Vec<f32>>,
    mapdatanextz: Vec<Vec<f32>>,
    mapdatanextxz: Vec<Vec<f32>>,
    minmapdata: Vec<Vec<f32>>,
    minmapdatanextx: Vec<Vec<f32>>,
    minmapdatanextz: Vec<Vec<f32>>,
    minmapdatanextxz: Vec<Vec<f32>>,
    pub doneinit :u32,
    pub mindoneinit :u32,
    doneinitx :u32,
    doneinitz :u32,
    doneinitxz: u32,
    donexe :u32,
    donexw :u32,
    donezn :u32,
    donezs :u32,
    donese :u32,
    donesw :u32,
    donene :u32,
    donenw :u32,
    mindoneinitx :u32,
    mindoneinitz :u32,
    mindoneinitxz: u32,
    mindonexe :u32,
    mindonexw :u32,
    mindonezn :u32,
    mindonezs :u32,
    mindonese :u32,
    mindonesw :u32,
    mindonene :u32,
    mindonenw :u32,
    north: bool,
    east: bool,
    south: bool,
    west: bool,
    //latitude and longitude coordinates
    pub lat :u32,
    pub long :u32,
    pub chunksize:u32,
    //threads used
    initthread: Threaded,
    nthread: Threaded,
    ethread: Threaded,
    sthread: Threaded,
    wthread: Threaded,
    nethread: Threaded,
    esthread: Threaded,
    swthread: Threaded,
    wnthread: Threaded,
    mininitthread: Threaded,
    minnthread: Threaded,
    minethread: Threaded,
    minsthread: Threaded,
    minwthread: Threaded,
    minnethread: Threaded,
    minesthread: Threaded,
    minswthread: Threaded,
    minwnthread: Threaded,
    pub minimised: bool

}

impl Default for Terrain {
        fn default() -> Self {
            let lat =55;
            let long = 5;
            //initialise all threads at runtime correspondng to the correct layout
            let mininitialthread = Threaded::default_with_params(lat,long,true);
            let minnorththread = Threaded::default_with_params(lat+1,long,true);
            let minnortheastthread = Threaded::default_with_params(lat+1,long-1,true);
            let mineastthread = Threaded::default_with_params(lat,long-1,true);
            let mineastsouththread = Threaded::default_with_params(lat-1,long-1,true);
            let minsouththread = Threaded::default_with_params(lat-1,long,true);
            let minsouthwestthread = Threaded::default_with_params(lat-1,long+1,true);
            let minwestthread = Threaded::default_with_params(lat,long+1,true);
            let minwestnorththread = Threaded::default_with_params(lat+1,long+1,true);
            let initialthread = Threaded::default_with_params(lat,long,false);
            let norththread = Threaded::default_with_params(lat+1,long,false);
            let northeastthread = Threaded::default_with_params(lat+1,long-1,false);
            let eastthread = Threaded::default_with_params(lat,long-1,false);
            let eastsouththread = Threaded::default_with_params(lat-1,long-1,false);
            let souththread = Threaded::default_with_params(lat-1,long,false);
            let southwestthread = Threaded::default_with_params(lat-1,long+1,false);
            let westthread = Threaded::default_with_params(lat,long+1,false);
            let westnorththread = Threaded::default_with_params(lat+1,long+1,false);
        Self {
            offsets: [0.0, 0.0],
            moves:[1800.0,1800.0],//start in the middle of the srtm tile
            level_of_detail: 0,
            water_level: 0.001,
            mapdata: vec![],
            mapdatanextx: vec![],
            mapdatanextz: vec![],
            mapdatanextxz: vec![],
            minmapdata: vec![],
            minmapdatanextx: vec![],
            minmapdatanextz: vec![],
            minmapdatanextxz: vec![],
            doneinit:0,
            mindoneinit:0,
            doneinitx :0,
            doneinitz :0,
            doneinitxz :0,
            donexe:0,
            donexw:0,
            donezn:0,
            donezs:0,
            donese:0,
            donesw:0,
            donene:0,
            donenw:0,
            mindoneinitx :0,
            mindoneinitz :0,
            mindoneinitxz :0,
            mindonexe:0,
            mindonexw:0,
            mindonezn:0,
            mindonezs:0,
            mindonese:0,
            mindonesw:0,
            mindonene:0,
            mindonenw:0,
            north: false,
            east: false,
            south: false,
            west: false,
            chunksize:241,
            lat:lat,
            long:long,
            initthread: initialthread,
            nthread: norththread,
            ethread: eastthread,
            sthread: souththread,
            wthread: westthread,
            nethread: northeastthread,
            esthread: eastsouththread,
            swthread: southwestthread,
            wnthread: westnorththread,
            mininitthread: mininitialthread,
            minnthread: minnorththread,
            minethread: mineastthread,
            minsthread: minsouththread,
            minwthread: minwestthread,
            minnethread: minnortheastthread,
            minesthread: mineastsouththread,
            minswthread: minsouthwestthread,
            minwnthread: minwestnorththread,
            minimised: false,
        }
    }
}

impl Terrain {

    pub fn create_indices(&mut self, width: u32, height: u32) -> (Vec<u32>, Vec<u32>) {
        //creating the indices based on the height and width of each chunk
        let n_vertices_per_row = height;
        let mut indices:Vec<u32> = vec![];
        let mut texindices:Vec<u32> = vec![];
        for i in 0..width - 1 {
            for j in 0..height - 1 {
                let idx0 = j + i * n_vertices_per_row;
                let idx1 = j + 1 + i * n_vertices_per_row;
                let idx2 = j + 1 + (i + 1) * n_vertices_per_row;
                let idx3 = j + (i + 1) * n_vertices_per_row;
                indices.extend([idx0, idx1, idx2, idx2, idx3, idx0]);
                texindices.extend([idx0, idx1, idx0, idx3]);
                if i == width - 2 || j == height - 1 {
                    texindices.extend([idx1, idx2, idx2, idx3]);
                }
            }
        }
        (indices, texindices)
    }
    //Adapting srtm tile to vector of vectors moved in theads
    /*pub fn find_world_map(&mut self) {
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
                self.mapdata[x][z] = (self.mapdata[x][z])/(height_max - height_min);
            }
        }
    }*/
    fn color_interp(&mut self, color:&Vec<[f32;3]>, ta:&Vec<f32>, t:f32) -> [f32;3] {
        //interpreting color based on the y value
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
        //adding the terrain color
        let mut tt = if t < tmin { tmin } else if t > tmax { tmax } else { t };
        tt = (tt - tmin)/(tmax - tmin);
        let t1 = self.shift_water_level(ta);
        self.color_interp(color, &t1, tt)
    }
    fn shift_water_level(&mut self, ta:&Vec<f32>) -> Vec<f32> {
        //adding the water level
        let mut t1 = vec![0f32; 6];
        let r = (1.0 - self.water_level)/(1.0 - ta[1]);
        t1[1] = self.water_level;
        for i in 1..5usize {
            let del = ta[i+1] - ta[i];
            t1[i+1] = t1[i] + r * del;
        }
        t1
    }



    pub fn create_terrain_data(&mut self) -> (Vec<Vertex>,Vec<Vertex>, u32) {
        //level of detail used to calculate the increment count
        let increment_count = if self.level_of_detail <= 5 { self.level_of_detail + 1} else { 2*(self.level_of_detail - 2)};
        let vertices_per_row = (self.chunksize - 1)/increment_count + 1;
        //colormap
        let cdata =  vec![
            [0.055f32, 0.529, 0.8],
            [0.761, 0.698, 0.502],
            [0.204, 0.549, 0.192],
            [0.353, 0.302, 0.255],
            [1.0, 0.98, 0.98]
        ];
        let tdata = vec![[1f32, 1.0, 1.0]; 5];
        let ta = vec![0.0f32, 0.3, 0.35, 0.7, 0.9, 1.0];
        //receive the initial srtm tile from the thread at runtime only once
        if self.minimised == false{
        if self.doneinit == 0 {
            for received in &self.initthread.refer {
                self.mapdata = received;
                self.doneinit+=1;
            }
        }}else{if self.mindoneinit == 0 {
            for received in &self.mininitthread.refer {
                self.minmapdata = received;
                self.mindoneinit+=1;
            }
        }

        }


        let mut data:Vec<Vertex> = vec![];
        let mut texturedata:Vec<Vertex> = vec![];
        if self.minimised == false {
            for x in (0..self.chunksize as usize).step_by(increment_count as usize) {
                for z in (0..self.chunksize as usize).step_by(increment_count as usize) {
                    //calculating new x and z values based on the chunk and the how much the user moves
                    let usex: i32 = (x as f32 + self.offsets[0] + self.moves[0]) as i32;
                    let usez: i32 = (z as f32 + self.offsets[1] + self.moves[1]) as i32;
                    let mut y = 0.0;
                    match usez as i32 {//retieving other srtm data from tiles depending on dirction making sure its only done once
                        m if m < -1800 => {
                            println!("chunk");
                            self.moves[1] += 3600.0;
                            self.mapdata = vec![];
                            self.mapdata = self.mapdatanextz.clone();
                            self.lat += 1;
                            self.doneinitxz = 2;
                            self.doneinitz = 2;
                            self.doneinitx = 2;
                            self.donezn = 0;
                            self.donene = 0;
                            self.donenw = 0;
                            self.north = true;
                        }
                        -1800..=-1 => {
                            self.north = true;
                        }
                        0..=200 => {
                            if self.donezn == 0 {
                                for received in &self.nthread.refer {
                                    self.mapdatanextz = vec![];
                                    self.mapdatanextz = received
                                }

                                self.donezn += 1;
                            }
                        }
                        201..=3400 => {
                            self.doneinitxz = 0;
                            if self.doneinitz == 2 {//new srtm tiles to be created
                                Threaded::transferwithret(&mut self.sthread, self.lat - 1, self.long, false);
                                Threaded::transferwithret(&mut self.nthread, self.lat + 1, self.long, false);

                                self.doneinitz += 1;
                            }
                            if self.doneinitxz == 2 {
                                Threaded::transferwithret(&mut self.nethread, self.lat + 1, self.long - 1, false);
                                Threaded::transferwithret(&mut self.wnthread, self.lat + 1, self.long + 1, false);
                                Threaded::transferwithret(&mut self.esthread, self.lat - 1, self.long - 1, false);
                                Threaded::transferwithret(&mut self.swthread, self.lat - 1, self.long + 1, false);

                                self.doneinitz += 1;
                            }
                        }
                        3401..=3599 => {
                            if self.donezs == 0 {
                                for received in &self.sthread.refer {
                                    self.mapdatanextz = vec![];
                                    self.mapdatanextz = received
                                }
                                self.donezs += 1;
                            }
                        }
                        3600..=5400 => {
                            self.south = true;
                        }
                        _ => {
                            println!("chunk");
                            self.moves[1] -= 3600.0;
                            self.mapdata = vec![];
                            self.mapdata = self.mapdatanextz.clone();
                            self.lat -= 1;
                            self.doneinitxz = 2;
                            self.doneinitz = 2;
                            self.doneinitx = 2;
                            self.donezs = 0;
                            self.donesw = 0;
                            self.donese = 0;
                            self.south = true;
                        }
                    }
                    match usex as i32 {
                        n if n < -1800 => {
                            println!("chunk");
                            self.moves[0] += 3600.0;
                            self.mapdata = vec![];
                            self.mapdata = self.mapdatanextx.clone();
                            self.long += 1;
                            self.doneinitxz = 2;
                            self.doneinitz = 2;
                            self.doneinitx = 2;
                            self.donexw = 0;
                            self.donesw = 0;
                            self.donenw = 0;
                            self.west = true
                        }
                        -1800..=-1 => {
                            self.west = true;
                        }
                        0..=200 => {
                            if self.donexw == 0 {
                                for received in &self.wthread.refer {
                                    self.mapdatanextx = vec![];
                                    self.mapdatanextx = received
                                }

                                self.donexw += 1;
                            }
                        }
                        201..=3400 => {
                            self.doneinitxz = 0;
                            if self.doneinitx == 2 {
                                Threaded::transferwithret(&mut self.ethread, self.lat, self.long - 1, false);

                                Threaded::transferwithret(&mut self.wthread, self.lat, self.long + 1, false);

                                self.doneinitx += 1;
                            }
                            if self.doneinitxz == 2 {
                                Threaded::transferwithret(&mut self.nethread, self.lat + 1, self.long - 1, false);
                                Threaded::transferwithret(&mut self.wnthread, self.lat + 1, self.long + 1, false);
                                Threaded::transferwithret(&mut self.esthread, self.lat - 1, self.long - 1, false);
                                Threaded::transferwithret(&mut self.swthread, self.lat - 1, self.long + 1, false);

                                self.doneinitz += 1;
                            }
                        }
                        3401..=3599 => {
                            if self.donexe == 0 {
                                for received in &self.ethread.refer {
                                    self.mapdatanextx = vec![];
                                    self.mapdatanextx = received
                                }
                                self.donexe += 1;
                            }
                        }
                        3600..=5400 => {
                            self.east = true;
                        }
                        _ => {
                            println!("chunk");
                            self.moves[0] -= 3600.0;
                            self.mapdata = vec![];
                            self.mapdata = self.mapdatanextx.clone();
                            if self.long > 1 {
                                self.long -= 1;
                            }
                            self.doneinitxz = 2;
                            self.doneinitz = 2;
                            self.doneinitx = 2;
                            self.donexe = 0;
                            self.donene = 0;
                            self.donese = 0;
                            self.east = true;
                        }
                    }
                    //finding the correct map data to use based on the direction moving in
                    if self.south || self.north || self.east || self.west {
                    if self.south {
                        if self.east{if self.donese ==0 {
                            for received in &self.esthread.refer {
                                self.mapdatanextxz = vec![];
                                self.mapdatanextxz = received
                            }

                            self.donese +=1;
                        }
                            y = self.mapdatanextxz[(usex -3600) as usize][(usez - 3600) as usize];
                        }else if self.west{
                            if self.donesw ==0 {
                            for received in &self.swthread.refer {
                                self.mapdatanextxz = vec![];
                                self.mapdatanextxz = received
                            }

                            self.donesw +=1;
                        }
                            y = self.mapdatanextxz[(usex +3600) as usize][(usez - 3600) as usize];
                        }else{
                            y = self.mapdatanextz[usex as usize][(usez - 3600) as usize];
                        }
                    }else{
                    if self.north {
                        if self.east{
                            if self.donene ==0 {
                            for received in &self.nethread.refer {
                                self.mapdatanextxz = vec![];
                                self.mapdatanextxz = received
                            }

                            self.donene +=1;
                        }
                            y = self.mapdatanextxz[(usex - 3600) as usize][(usez + 3600) as usize];
                        }else if self.west{
                            if self.donenw ==0 {
                            for received in &self.wnthread.refer {
                                self.mapdatanextxz = vec![];
                                self.mapdatanextxz = received
                            }

                            self.donenw +=1;
                        }
                            y = self.mapdatanextxz[(usex +3600) as usize][(usez + 3600) as usize];
                        }else{
                            y = self.mapdatanextz[usex as usize][(usez + 3600) as usize];
                        }
                    }else{
                    if self.east{
                        y = self.mapdatanextx[(usex - 3600) as usize][usez as usize];
                    }else{
                    if self.west{
                        y = self.mapdatanextx[(usex + 3600) as usize][usez as usize];
                    }}}}
                        }else{
                    y = self.mapdata[usex as usize][usez as usize];
                }
                    self.north = false;
                    self.east = false;
                    self.south = false;
                    self.west = false;
                    //print!("{} ,,",y);
                    if y < self.water_level {//making sure the y values to be water values are the same for a smooth water line
                        y = self.water_level - 0.01;
                    }
                    let position = [x as f32, y, z as f32];
                    let color = self.add_terrain_colors(&cdata, &ta, 0.0, 1.0, y);
                    let texturecolor = self.add_terrain_colors(&tdata, &ta, 0.0, 1.0, y);
                    data.push(Vertex { position, color });
                    texturedata.push(Vertex { position, color: texturecolor });
                }
            }
        }else{
            for x in 0..(self.chunksize) as usize {
            for z in 0..(self.chunksize) as usize {
                //calculating new x and z values based on the chunk and the how much the user moves
                let usex : i32= (x as f32 + (self.offsets[0]) + (self.moves[0])/4.0)as i32;
                let usez : i32= (z as f32 + (self.offsets[1]) + (self.moves[1])/4.0)as i32;
                let mut y = 0.0;
                match usez as i32 {//retieving other srtm data from tiles depending on dirction making sure its only done once
                    m if m < -450 =>{
                        println!("chunk");
                            self.moves[1] += 3600.0;
                            self.minmapdata = vec![];
                            self.minmapdata = self.minmapdatanextz.clone();
                            self.lat += 1;
                            self.mindoneinitxz =2;
                            self.mindoneinitz =2;
                            self.mindoneinitx=2;
                            self.mindonezn = 0;
                            self.mindonene = 0;
                            self.mindonenw = 0;
                            self.north= true;
                    }
                    -450 ..=-1 => {
                        self.north = true;

                    }
                     0 ..=50 => {
                        if self.mindonezn==0 {
                            for received in &self.minnthread.refer {
                                self.minmapdatanextz = vec![];
                                self.minmapdatanextz = received
                            }

                            self.mindonezn += 1;
                        }
                    }
                    51 ..=850 => {
                        self.mindoneinitxz = 0;
                        if self.mindoneinitz ==2 {//new srtm tiles to be created
                                Threaded::transferwithret(&mut self.minsthread,self.lat-1,self.long,true);
                                Threaded::transferwithret(&mut self.minnthread,self.lat+1,self.long,true);

                            self.mindoneinitz +=1;
                        }
                        if self.mindoneinitxz ==2 {
                                Threaded::transferwithret(&mut self.minnethread,self.lat+1,self.long-1,true);
                                Threaded::transferwithret(&mut self.minwnthread,self.lat+1,self.long+1,true);
                                Threaded::transferwithret(&mut self.minesthread,self.lat-1,self.long-1,true);
                                Threaded::transferwithret(&mut self.minswthread,self.lat-1,self.long+1,true);

                            self.mindoneinitz +=1;
                        }
                    }
                    851 ..= 899=>{
                        if self.mindonezs == 0 {
                            for received in &self.minsthread.refer {
                                self.mapdatanextz = vec![];
                                self.mapdatanextz = received
                            }
                            self.donezs += 1;
                        }
                    }
                    900 ..= 1350=>{
                        self.south= true;
                    }
                    _ => {
                        println!("chunk");
                        self.moves[1] -=3600.0;
                        self.minmapdata = vec![];
                        self.minmapdata = self.minmapdatanextz.clone();
                        self.lat -= 1;
                        self.mindoneinitxz =2;
                        self.mindoneinitz =2;
                        self.mindoneinitx=2;
                        self.mindonezs=0;
                        self.mindonesw =0;
                        self.mindonese = 0;
                        self.south= true;

                }}
                match usex as i32 {
                    n if n < -450 =>{
                        println!("chunk");
                        self.moves[0] += 3600.0;
                        self.minmapdata = vec![];
                        self.minmapdata = self.minmapdatanextx.clone();
                        self.long +=1;
                        self.mindoneinitxz =2;
                        self.mindoneinitz =2;
                        self.mindoneinitx=2;
                        self.mindonexw=0;
                        self.mindonesw =0;
                        self.mindonenw = 0;
                        self.west = true
                    }
                    -450 ..=-1 => {
                        if self.mindonexw==0 {
                            for received in &self.minwthread.refer {
                                self.minmapdatanextx = vec![];
                                self.minmapdatanextx = received
                            }

                            self.mindonexw += 1;
                        }
                        self.west = true;
                    }
                     0 ..= 50 => {
                         self.west = false;
                        if self.mindonexw==0 {
                            for received in &self.minwthread.refer {
                                self.minmapdatanextx = vec![];
                                self.minmapdatanextx = received
                            }

                            self.mindonexw += 1;
                        }
                    }
                    51 ..=850 =>{
                        self.mindoneinitxz = 0;
                        if self.mindoneinitx ==2 {
                            Threaded::transferwithret(&mut self.minethread,self.lat,self.long-1,true);

                            Threaded::transferwithret(&mut self.minwthread,self.lat,self.long+1,true);

                            self.mindoneinitx +=1;
                        }
                        if self.mindoneinitxz ==2 {
                                Threaded::transferwithret(&mut self.minnethread,self.lat+1,self.long-1,true);
                                Threaded::transferwithret(&mut self.minwnthread,self.lat+1,self.long+1,true);
                                Threaded::transferwithret(&mut self.minesthread,self.lat-1,self.long-1,true);
                                Threaded::transferwithret(&mut self.swthread,self.lat-1,self.long+1,true);

                            self.mindoneinitz +=1;
                        }
                    }
                    851 ..= 899=>{
                        if self.mindonexe == 0 {
                            for received in &self.minethread.refer {
                                self.minmapdatanextx = vec![];
                                self.minmapdatanextx = received
                            }
                            self.mindonexe += 1;
                        }
                    }
                    900 ..= 1350=>{
                        self.east =true;
                    }
                    _ => {
                        println!("chunk");
                        self.moves[0] -=3600.0;
                        self.minmapdata = vec![];
                        self.minmapdata = self.minmapdatanextx.clone();
                        if self.long >1 {
                            self.long -= 1;
                        }
                        self.mindoneinitxz =2;
                        self.mindoneinitz =2;
                        self.mindoneinitx=2;
                        self.mindonexe=0;
                        self.mindonene = 0;
                        self.mindonese = 0;
                        self.east =true;

                }}
                //finding the correct map data to use based on the direction moving in
                if self.south || self.north || self.east || self.west {
                    if self.south {
                        if self.east{if self.mindonese ==0 {
                            for received in &self.minesthread.refer {
                                self.minmapdatanextxz = vec![];
                                self.minmapdatanextxz = received
                            }

                            self.mindonese +=1;
                        }
                            y = self.minmapdatanextxz[(usex -900) as usize][(usez - 900) as usize];
                        }else if self.west{
                            if self.mindonesw ==0 {
                            for received in &self.minswthread.refer {
                                self.minmapdatanextxz = vec![];
                                self.minmapdatanextxz = received
                            }

                            self.mindonesw +=1;
                        }
                            y = self.minmapdatanextxz[(usex +900) as usize][(usez - 900) as usize];
                        }else{
                            y = self.minmapdatanextz[usex as usize][(usez - 900) as usize];
                        }
                    }else{
                    if self.north {
                        if self.east{
                            if self.mindonene ==0 {
                            for received in &self.minnethread.refer {
                                self.minmapdatanextxz = vec![];
                                self.minmapdatanextxz = received
                            }

                            self.mindonene +=1;
                        }
                            y = self.minmapdatanextxz[(usex - 900) as usize][(usez + 900) as usize];
                        }else if self.west{
                            if self.mindonenw ==0 {
                            for received in &self.minwnthread.refer {
                                self.minmapdatanextxz = vec![];
                                self.minmapdatanextxz = received
                            }

                            self.mindonenw +=1;
                        }
                            y = self.minmapdatanextxz[(usex +900) as usize][(usez + 900) as usize];
                        }else{
                            y = self.minmapdatanextz[usex as usize][(usez + 900) as usize];
                        }
                    }else{
                    if self.east{
                        y = self.minmapdatanextx[(usex - 900) as usize][usez as usize];
                    }else{
                    if self.west{
                        y = self.minmapdatanextx[(usex + 900) as usize][usez as usize];
                    }}}}
                }else{
                    y = self.minmapdata[usex as usize][usez as usize];
                }
                self.north = false;
                self.east = false;
                self.south = false;
                self.west = false;
               if y < self.water_level {//making sure the y values to be water values are the same for a smooth water line
                    y = self.water_level - 0.01;
                }
                let position = [x as f32, y, z as f32];
                let color = self.add_terrain_colors(&cdata, &ta, 0.0, 1.0, y);
                let texturecolor = self.add_terrain_colors(&tdata, &ta, 0.0, 1.0, y);
                data.push(Vertex { position, color});
                texturedata.push(Vertex { position, color: texturecolor });
            }
        }
        }
    (data, texturedata, vertices_per_row)
    }
    pub fn create_collection_of_terrain_data(&mut self, x_chunks:u32, z_chunks:u32, translations:&Vec<[f32;2]>) -> (Vec<Vec<Vertex>>,Vec<Vec<Vertex>>, u32) {
        let mut data:Vec<Vec<Vertex>> = vec![];
        let mut texturedata:Vec<Vec<Vertex>> = vec![];
        let mut vertices_per_row = 0u32;
        //going through all the chunks and calculating the function to calculate the y values
        let mut k:u32 = 0;
        for _i in 0..x_chunks {
            //self.level_of_detail=5;
            for _j in 0..z_chunks {
                self.offsets = translations[k as usize];
                let dd = self.create_terrain_data();
                data.push(dd.0);
                texturedata.push(dd.1);
                vertices_per_row = dd.2;
                k += 1;
            }
        }
        (data, texturedata, vertices_per_row)
    }





}
