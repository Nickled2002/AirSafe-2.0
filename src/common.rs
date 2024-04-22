use std:: {collections::VecDeque,iter, mem };
use cgmath::Matrix4;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    window::Window,
    event_loop::{ControlFlow, EventLoop},
};
use bytemuck::{cast_slice, Pod, Zeroable};
use wgpu::VertexBufferLayout;
use std::time::{Instant, Duration};



#[path="transforms.rs"]
mod transforms;//transforms:: references transforms.rs file
#[path="surface_data.rs"]
mod surface;//surface:: references surface.rs file

const X_CHUNKS_COUNT: u32 = 4;
const Z_CHUNKS_COUNT: u32 = 4;



#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CamPos{//initialise CamPos Struct
    pub x:f32,
    pub y:f32,
    pub z:f32
}
struct RenderPipeline<'a> {// render pipeline struct created incase a second render pipeline is required such as changing the view to a wireframe or the color
    pub shader: Option<&'a wgpu::ShaderModule>,
    pub vs_shader: Option<&'a wgpu::ShaderModule>,
    pub fs_shader: Option<&'a wgpu::ShaderModule>,
    pub vertex_buffer_layout: &'a [VertexBufferLayout<'a>],
    pub pipeline_layout: Option<&'a wgpu::PipelineLayout>,
    pub topology: wgpu::PrimitiveTopology,
    pub strip_index_format: Option<wgpu::IndexFormat>,
    pub is_depth_stencil: bool,
    pub vs_entry: String,
    pub fs_entry: String,
}impl Default for RenderPipeline<'_> {
    fn default() -> Self {
        Self {
            shader: None,
            vs_shader: None,
            fs_shader: None,
            vertex_buffer_layout: &[],
            pipeline_layout: None,
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            is_depth_stencil: true,
            vs_entry: String::from("vs_main"),
            fs_entry: String::from("fs_main"),
        }
    }
}impl RenderPipeline<'_> {
    pub fn new(&mut self, init: &WgpuInit) -> wgpu::RenderPipeline {//new render pipeline stuct initialisation
        if self.shader.is_some() {
            self.vs_shader = self.shader;
            self.fs_shader = self.shader;
        }

        let mut depth_stencil:Option<wgpu::DepthStencilState> = None;
        if self.is_depth_stencil {
            depth_stencil = Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            });
        }

        init.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&self.pipeline_layout.unwrap()),
            vertex: wgpu::VertexState {
                module: &self.vs_shader.as_ref().unwrap(),
                entry_point: &self.vs_entry,
                buffers: &self.vertex_buffer_layout,
            },
            fragment: Some(wgpu::FragmentState {
                module: &self.fs_shader.as_ref().unwrap(),
                entry_point: &self.fs_entry,
                targets: &[Some(init.config.format.into())],
            }),

            primitive: wgpu::PrimitiveState {
                topology: self.topology,
                strip_index_format: self.strip_index_format,
                ..Default::default()
            },
            depth_stencil,
            multisample: wgpu::MultisampleState{
                count: init.sample_count,
                ..Default::default()
            },
            multiview: None,
        })
    }
}


struct WgpuInit {//struct WgpuInit required variables for inialisation of window with WGPU
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub sample_count: u32,
}impl WgpuInit {
    //implementation of funtions of WgpuInit struct
    pub async fn new(window: &Window, sample_count:u32, limits:Option<wgpu::Limits>) -> Self {
        //new WgpuInitStruct with params
        let limits_device = limits.unwrap_or(wgpu::Limits::default());

        let size = window.inner_size();
        let instance = wgpu::Instance::default();

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    //features: wgpu::Features::empty(),
                    features:wgpu::Features::default() | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                    //limits: wgpu::Limits::default(),
                    limits: limits_device
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let surface_caps = surface.get_capabilities(&adapter);
        let format = surface_caps.formats[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode:surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        Self {//initialisation of self based on the values determined above
            surface,
            device,
            queue,
            config,
            size,
            sample_count,
        }
    }
}



#[derive(Debug)]
struct FpsCounter {//FpsCounter srtuct initialisation
    last_second_frames: VecDeque<Instant>,
    last_print_time: Instant,
}
impl Default for FpsCounter {//point to new when FpsCounter::Default() is called
    fn default() -> Self {
        Self::new()
    }
}

impl FpsCounter {
    //Creates a new FpsCounter.
    pub fn new() -> Self {
        Self {
            last_second_frames: VecDeque::with_capacity(128),
            last_print_time: Instant::now(),
        }
    }

    //Updates the fps counter and print fps.
    pub fn print_fps(&mut self, interval:u64) {
        let now = Instant::now();
        let a_second_ago = now - Duration::from_secs(1);

        while self.last_second_frames.front().map_or(false, |t| *t < a_second_ago) {
            self.last_second_frames.pop_front();
        }

        self.last_second_frames.push_back(now);

        //Check if the interval seconds have passed since the last print time
        if now - self.last_print_time >= Duration::from_secs(interval) {
            let fps = self.last_second_frames.len();
            println!("FPS: {}", fps);
            self.last_print_time = now;
        }
    }
}

struct State {
    //struct State variables all required variables to render a window
    init: WgpuInit,//sturct WgpuInit
    texture_pipeline: wgpu::RenderPipeline,
    pipeline: wgpu::RenderPipeline,//render pipeline
    vertex_buffer: Vec<wgpu::Buffer>,//vector of vertex buffers its a vector due to the use of chunks
    vertex_texture_buffer: Vec<wgpu::Buffer>,
    index_buffer: wgpu::Buffer,//index buffer
    tex_index_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    uniform_texture_bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    //view and projection matrix
    view_mat: Matrix4<f32>,
    project_mat: Matrix4<f32>,
    depth_texture_view: wgpu::TextureView,//depth texture
    index_length: u32,
    texindex_length: u32,
    plot_type: u32,
    camera: CamPos,//campos struct for positioning of camera
    camlook: CamPos,//campos struct for looking direction of camera
    translations: Vec<[f32; 2]>,
    terrain: surface::Terrain,//terrain struct initialised from surface_data.rs file
    update_buffers: bool,//update the buffers
    //update_buffers_view: bool, Not used anymore was used to update the view buffer without having to rerender and find the y values of the terrain thought to be more efficient wasnt
    fps_counter: FpsCounter,
}
impl State {
    async fn new(
        window: &Window,

    ) -> Self {
        let init = WgpuInit::new(&window,1, None).await;

        let shader = init.device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));//attach shader module written in wgsl
        //model matrix not needed to be calculated here anymore
        /*let model_mat = transforms::create_transforms(
            [-0.65 * width as f32, 5.0, -0.5 * height as f32],
            [0.0, 0.0, 0.0],
            [1.0, 100.0, 1.0],
        );*/
        let camera = CamPos{//values added to struct for position of camera
            x:0.0,
            y:100.0,
            z:0.0,
        };
        let camlook = CamPos{//values added to struct for looking direction of camera
            x:0.0,
            y:100.0,
            z:-30.0,
        };
        let mut terrain = surface::Terrain::default();//calling default function for terrrain struct
        let mut translations: Vec<[f32; 2]> = vec![];//empty vectors made mut so it can be filled later
        let mut model_mat: Vec<[f32; 16]> = vec![];
        let chunk_size1 = (terrain.chunksize - 1) as f32;
        for i in 0..X_CHUNKS_COUNT {//going through chunks to create the model matrix and the translation vector that will be used later for the positioning of the chunks
            for j in 0..Z_CHUNKS_COUNT {
                let xt = -0.5 * X_CHUNKS_COUNT as f32 * chunk_size1 + i as f32 * chunk_size1;
                let zt = -0.5 * Z_CHUNKS_COUNT as f32 * chunk_size1 + j as f32 * chunk_size1;
                let translation = [xt, 10.0, zt];
                let m = transforms::create_transforms(translation, [0.0, 0.0, 0.0], [1.0, 150.0, 1.0]);
                model_mat.push(*(m.as_ref()));
                translations.push([xt, zt]);
            }
        }
        //Model Matrix Storage Buffer initialised
        let model_storage_buffer = init.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Model Matrix Storage Buffer"),
                    contents: cast_slice(&model_mat),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                });
        //CamPos struct values added to the camera postition and look direction respectively
        let camera_position = (camera.x, camera.y, camera.z).into();
        let look_direction = (camlook.x, camlook.y, camlook.z).into();
        let up_direction = cgmath::Vector3::unit_y();
        //Calculation of view matrix projection matrix and viewprojection matrix from transforms.rs file
        let (view_mat, project_mat, vp_mat) = transforms::create_view_projection(
            camera_position,
            look_direction,
            up_direction,
            init.config.width as f32 / init.config.height as f32,
        );

        //let mvp_mat = vp_mat * model_mat; not needed to be calculated anymore here

        //Initiqalisation of vertex uniform buffers
        let vertex_uniform_buffer = init.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Uniform Buffer"),
                    contents: cast_slice(vp_mat.as_ref() as &[f32; 16]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });


        //And uniform bind group for vertex shader
        let (vertex_bind_group_layout, vertex_bind_group) = create_bind_group_storage(
            &init.device,
            vec![wgpu::ShaderStages::VERTEX, wgpu::ShaderStages::VERTEX],
            vec![
                wgpu::BufferBindingType::Uniform,
                wgpu::BufferBindingType::Storage { read_only: true },
            ],
            &[
                vertex_uniform_buffer.as_entire_binding(),
                model_storage_buffer.as_entire_binding(),
            ],
        );
        let (vertex_texture_bind_group_layout, vertex_texture_bind_group) = create_bind_group_storage(
            &init.device,
            vec![wgpu::ShaderStages::VERTEX, wgpu::ShaderStages::VERTEX],
            vec![
                wgpu::BufferBindingType::Uniform,
                wgpu::BufferBindingType::Storage { read_only: true },
            ],
            &[
                vertex_uniform_buffer.as_entire_binding(),
                model_storage_buffer.as_entire_binding(),
            ],
        );
        //And vertex buffer layout
        let vertex_buffer_layout = VertexBufferLayout {
            array_stride: mem::size_of::<surface::Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3], // position and color added to location 0 and 1 respectively (for shader)
        };
        //Configuring Layout of render pipeline
        let pipeline_layout = init.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&vertex_bind_group_layout],
                push_constant_ranges: &[],
            });
        //Initialised pipeline based on the above layout
        let mut ppl = RenderPipeline {
            shader: Some(&shader),
            pipeline_layout: Some(&pipeline_layout),
            vertex_buffer_layout: &[vertex_buffer_layout],
            ..Default::default()
        };
        let pipeline = ppl.new(&init);
        let vertex_texture_buffer_layout = VertexBufferLayout {
            array_stride: mem::size_of::<surface::Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3], // pos, col
        };

        let pipeline_texture_layout =
            init.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Texture Pipeline Layout"),
                    bind_group_layouts: &[&vertex_texture_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let mut pplt = RenderPipeline {
            topology: wgpu::PrimitiveTopology::LineList,
            shader: Some(&shader),
            pipeline_layout: Some(&pipeline_texture_layout),
            vertex_buffer_layout: &[vertex_texture_buffer_layout],
            ..Default::default()
        };
        let pipeline_texture = pplt.new(&init);


        let depth_texture_view = create_depth_view(&init);//Creattion o depth texture view no need for multi sample texture view
        let vertex_data = terrain.create_collection_of_terrain_data(//Calling create.... func from surface_data.rs file with those params
            X_CHUNKS_COUNT,
            Z_CHUNKS_COUNT,
            &translations,
        );
        let index_data = terrain.create_indices(vertex_data.2, vertex_data.2);//Calculation of indices
        let mut vertex_buffer: Vec<wgpu::Buffer> = vec![]; //Mutable vector of vertex buffers created filled below
        let mut vertex_texture_buffer: Vec<wgpu::Buffer> = vec![];
        let mut k: usize = 0;
        for _i in 0..X_CHUNKS_COUNT {//Iterate through all the chunks
            for _j in 0..Z_CHUNKS_COUNT {
                let vb = init.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {//retrieve data from vertex data from terrain struct
                        label: Some("Vertex Buffer"),
                        contents: cast_slice(&vertex_data.0[k]),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    });
                let vtb = init
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Vertex Texture Buffer"),
                        contents: cast_slice(&vertex_data.1[k]),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    });
                vertex_buffer.push(vb);//pushed into vertex vector
                vertex_texture_buffer.push(vtb);
                k += 1;
            }
        }
        //index buffer initialised and index data casted to it
        let index_buffer = init.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: cast_slice(&index_data.0),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            });
        let tex_index_buffer = init.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: cast_slice(&index_data.1),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            });


        Self {
            //Variables initialised above added to State struct
            init,
            pipeline,
            texture_pipeline: pipeline_texture,
            vertex_buffer,
            vertex_texture_buffer: vertex_texture_buffer,
            index_buffer,
            tex_index_buffer: tex_index_buffer,
            uniform_bind_group: vertex_bind_group,
            uniform_texture_bind_group: vertex_texture_bind_group,
            uniform_buffer:vertex_uniform_buffer,
            view_mat,
            project_mat,
            depth_texture_view,
            index_length: index_data.0.len() as u32,
            texindex_length: index_data.1.len() as u32,
            camera,
            camlook,
            translations,
            terrain,
            plot_type: 0,
            update_buffers: false,
            //update_buffers_view: false,
            fps_counter: FpsCounter::default(),
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {//Resizing window function
        if new_size.width > 0 && new_size.height > 0 {
            self.init.size = new_size;
            self.init.config.width = new_size.width;
            self.init.config.height = new_size.height;
            self.init.surface.configure(&self.init.device, &self.init.config);
            //Determines the new sizes recalculates the matrixes needed for viewing and updates the buffer
            self.project_mat = transforms::create_projection(new_size.width as f32 / new_size.height as f32, true);
            let vp_mat = self.project_mat * self.view_mat;
            self.init.queue.write_buffer(&self.uniform_buffer, 0, cast_slice(vp_mat.as_ref() as &[f32; 16]), );
            self.depth_texture_view = create_depth_view(&self.init);
        }
    }

    pub fn plane_move(&mut self, moves: char) {//Plane moving function called from input
        match moves {
            'e' => {//North south west and east depending on direction also moves camera to face the moving direction
                    self.terrain.moves[0] += 2.0;
                    self.update_buffers = true;
            },
            'w' => {

                    self.terrain.moves[0] -= 2.0;
                    self.update_buffers = true;

            },
            's' => {
                    self.terrain.moves[1] += 2.0;
                    self.update_buffers = true;

                },
            'n' =>{
                    self.terrain.moves[1] -= 2.0;
                    self.update_buffers = true;

            },
            'u' => {//Change y level completely
                self.camera.y = self.camera.y+1.0;
                self.camlook.y = self.camlook.y+1.0;
                self.update_buffers = true;

            },
            'd' => {
                self.camera.y = self.camera.y-1.0;
                self.camlook.y = self.camlook.y-1.0;
                self.update_buffers = true;

            },//or just the direction the camera is looking at
            'q' => self.camlook.y = self.camlook.y+1.0,
            'z' => self.camlook.y = self.camlook.y-1.0,
            'c' => self.update_buffers = true,
            _ => {}
        }
        let look_direction = (self.camlook.x,self.camlook.y,self.camlook.z).into();
        let up_direction = cgmath::Vector3::unit_y();

        let camera_position = (self.camera.x, self.camera.y, self.camera.z).into();
        //Looking direction and camera position recalculated
        let (view_mat, project_mat, _vp_mat) = transforms::create_view_projection(
            camera_position,
            look_direction,
            up_direction,
            self.init.config.width as f32 / self.init.config.height as f32,
        );//Update the view and projection matries _vp_mat unused thats why it starts with _

        self.view_mat=view_mat;
        self.project_mat=project_mat;
    }
    fn input(&mut self, event: &WindowEvent) -> bool {
        //Match key inputs to the appropriate effects
        match event {
            WindowEvent::KeyboardInput {
                input:
                KeyboardInput {
                    virtual_keycode: Some(keycode),
                    state: ElementState::Pressed,
                    ..
                },
                ..
            } => match keycode {
                VirtualKeyCode::W => {//Move north with w
                    self.plane_move('n');
                    true
                }
                VirtualKeyCode::S => {//Move south with s
                    self.plane_move('s');
                    true
                }
                VirtualKeyCode::Space => {
                    self.plot_type = (self.plot_type + 1) % 2;
                    self.update_buffers = true;
                    true
                }
                VirtualKeyCode::A => {//Move west with a
                    self.plane_move('w');
                    true
                }
                VirtualKeyCode::D => {//Move east with d
                    self.plane_move('e');
                    true
                }
                VirtualKeyCode::PageUp => {//Plane goes up
                    self.plane_move('u');
                    true
                }
                VirtualKeyCode::PageDown => {//Plane goes down
                    self.plane_move('d');
                    true
                }VirtualKeyCode::Right => {//Plane goes down
                    if self.camlook.x < 120.0 {
                        self.camlook.x += 3.0;
                    }
                    self.plane_move('c');
                    true
                }
                VirtualKeyCode::Left => {//Plane goes down
                    if self.camlook.x > -120.0 {
                        self.camlook.x -= 3.0;
                    }
                    self.plane_move('c');
                    true
                }
                VirtualKeyCode::Down => {//Plane goes down
                    if self.camlook.z == 30.0{if self.camlook.x<0.0 {
                        if self.camlook.x < -10.0 { self.camlook.x += 3.0; }
                        self.camlook.x += 1.0;
                    }
                    if self.camlook.x>0.0 {
                        if self.camlook.x > 10.0 { self.camlook.x -= 3.0; }
                        self.camlook.x -= 1.0;
                    }}
                    if self.camlook.z == -30.0{
                    if self.camlook.x<=0.0 {
                    if self.camlook.x > -120.0 {
                        self.camlook.x -= 3.0;
                    }
                    if self.camlook.x < -119.0 {
                        self.camlook.z = 30.0;
                        self.camlook.x = -120.0;
                    }}if self.camlook.x>0.0 {
                        if self.camlook.x < 120.0 {
                            self.camlook.x += 3.0;
                        }
                        if self.camlook.x > 119.0 {
                            self.camlook.z = 30.0;
                            self.camlook.x = 120.0;
                        }
                    }}
                    self.plane_move('c');
                    true
                }
                VirtualKeyCode::Up => {//Plane goes down
                    if self.camlook.z == 30.0{
                        if self.camlook.x<=0.0 {
                            if self.camlook.x > -120.0 {
                                self.camlook.x -= 3.0;
                            }
                        if self.camlook.x < -119.0{
                            self.camlook.z = -30.0;
                            self.camlook.x =-120.0;
                        }}if self.camlook.x>0.0 {
                            if self.camlook.x < 120.0 {
                                self.camlook.x += 3.0;
                        }
                        if self.camlook.x > 119.0 {
                            self.camlook.z = -30.0;
                            self.camlook.x = 120.0;
                        }
                        }}
                        if self.camlook.z == -30.0{
                        if self.camlook.x<0.0{
                        if self.camlook.x< -10.0{self.camlook.x += 3.0;}
                        self.camlook.x += 1.0;
                        }
                        if self.camlook.x>0.0{
                        if self.camlook.x> 10.0{self.camlook.x -= 3.0;}
                            self.camlook.x -= 1.0;
                        }
                        }
                    self.plane_move('c');
                    true
                }
                VirtualKeyCode::Q => {//Look up
                    self.plane_move('q');
                    self.update_buffers = true;
                    true
                }
                VirtualKeyCode::E => {//Look down
                    self.plane_move('z');
                    self.update_buffers = true;
                    true
                }
                VirtualKeyCode::R => {//Decrease level of detail increase performance
                    if self.terrain.level_of_detail <7 {
                        self.terrain.level_of_detail += 1;
                    }
                    self.update_buffers = true;
                    true
                }
                VirtualKeyCode::F => {//Increase level of detail decrease performance
                    if self.terrain.level_of_detail >0 {
                        self.terrain.level_of_detail -= 1;
                    }
                    self.update_buffers = true;
                    true
                }
                VirtualKeyCode::LControl => {//Increase level of detail decrease performance
                    if self.terrain.minimised == true {
                        self.terrain.minimised = false;
                    }else{
                        self.terrain.minimised = true;
                    }
                    self.update_buffers = true;
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    fn update(&mut self) {
        // update buffers:
        if self.update_buffers {
            //Recalculate vertex data
            let vertex_data = self.terrain.create_collection_of_terrain_data(
            X_CHUNKS_COUNT,
            Z_CHUNKS_COUNT,
            &self.translations,
        );
            let mut k = 0usize;
            //Iterate through all chunks and add vertex data to the buffer
            for _i in 0..X_CHUNKS_COUNT {
                for _j in 0..Z_CHUNKS_COUNT {
                    self.init.queue.write_buffer(
                        &self.vertex_buffer[k],
                        0,
                        cast_slice(&vertex_data.0[k]),
                    );
                    self.init.queue.write_buffer(
                        &self.vertex_texture_buffer[k],
                        0,
                        cast_slice(&vertex_data.1[k]),
                    );
                    k += 1;
                }
            }//re calculate view projection matrix
            let vp_mat = self.project_mat * self.view_mat;
            self.init.queue.write_buffer(&self.uniform_buffer, 0, cast_slice(vp_mat.as_ref() as &[f32; 16]), );
            //update index data and write to buffer
            let index_data = self.terrain.create_indices(vertex_data.2, vertex_data.2);
            self.init.queue.write_buffer(&self.index_buffer, 0, cast_slice(&index_data.0));
            self.init.queue.write_buffer(&self.tex_index_buffer, 0, cast_slice(&index_data.1));
            self.index_length = index_data.0.len() as u32;
            self.texindex_length = index_data.1.len() as u32;
            self.update_buffers = false;
        }
    }

     fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        //Render pass renders all data from the buffers
        let output = self.init.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.init.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder"), });
        {
            let color_attach = create_color_attachment(&view);
            let depth_attachment = create_depth_stencil_attachment(&self.depth_texture_view);

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(color_attach)],
                depth_stencil_attachment: Some(depth_attachment),
            });

            let plot_type = if self.plot_type == 0 {
                "shape"
            }else{
                "both"
            };
             if plot_type == "shape" || plot_type == "both" {
                 render_pass.set_pipeline(&self.pipeline);
                 render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                 let mut k: u32 = 0;
                 for _i in 0..X_CHUNKS_COUNT {
                     for _j in 0..Z_CHUNKS_COUNT {
                         render_pass.set_vertex_buffer(0, self.vertex_buffer[k as usize].slice(..));
                         render_pass.set_index_buffer(
                             self.index_buffer.slice(..),
                             wgpu::IndexFormat::Uint32,
                         );
                         render_pass.draw_indexed(0..self.index_length, 0, k..k + 1);
                         k += 1;
                     }
                 }
             }
            if plot_type == "both" {
                render_pass.set_pipeline(&self.texture_pipeline);
                render_pass.set_bind_group(0, &self.uniform_texture_bind_group, &[]);

                let mut k: u32 = 0;
                for _i in 0..X_CHUNKS_COUNT {
                    for _j in 0..Z_CHUNKS_COUNT {
                        render_pass
                            .set_vertex_buffer(0, self.vertex_texture_buffer[k as usize].slice(..));
                        render_pass.set_index_buffer(
                            self.tex_index_buffer.slice(..),
                            wgpu::IndexFormat::Uint32,
                        );
                        render_pass.draw_indexed(0..self.texindex_length, 0, k..k + 1);
                        k += 1;
                    }
                }
            }
        }
        self.fps_counter.print_fps(5);
        self.init.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}pub fn create_bind_group_layout_storage(device: &wgpu::Device, shader_stages: Vec<wgpu::ShaderStages>, binding_types: Vec<wgpu::BufferBindingType>) -> wgpu::BindGroupLayout {
    //function to create bind group layout returns wgpu object BindGroupLayout
    let mut entries = vec![];

    for i in 0..shader_stages.len() {
        entries.push(wgpu::BindGroupLayoutEntry {
            binding: i as u32,
            visibility: shader_stages[i],
            ty: wgpu::BindingType::Buffer {
                ty: binding_types[i],
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });
    }

    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
        entries: &entries,
        label: Some("Uniform Bind Group Layout"),
    })
}

pub fn create_bind_group_storage(device: &wgpu::Device, shader_stages: Vec<wgpu::ShaderStages>, binding_types: Vec<wgpu::BufferBindingType>, resources: &[wgpu::BindingResource<'_>]) -> ( wgpu::BindGroupLayout, wgpu::BindGroup) {
    let entries: Vec<_> = resources.iter().enumerate().map(|(i, resource)| {
        wgpu::BindGroupEntry {
            binding: i as u32,
            resource: resource.clone(),
        }
    }).collect();

    let layout = create_bind_group_layout_storage(device, shader_stages, binding_types);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &layout,
        entries: &entries,
        label: Some("Uniform Bind Group"),
    });

    (layout, bind_group)
}


pub fn create_color_attachment<'a>(texture_view: &'a wgpu::TextureView) -> wgpu::RenderPassColorAttachment<'a> {
    let mut blue = wgpu::Color::BLUE;
    //light blue
    blue.r =0.68;
    blue.g =0.85;
    blue.b =0.9;
    blue.a =1.0;
    wgpu::RenderPassColorAttachment {
        view: texture_view,
        resolve_target: None,
        ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(blue),
            store: true,
        },
    }
}

fn create_depth_view(init: &WgpuInit) -> wgpu::TextureView {
    let depth_texture = init.device.create_texture(&wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width: init.config.width,
            height: init.config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: init.sample_count,
        dimension: wgpu::TextureDimension::D2,
        format:wgpu::TextureFormat::Depth24Plus,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: None,
        view_formats: &[],
    });

    depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
}

pub fn create_depth_stencil_attachment<'a>(depth_view: &'a wgpu::TextureView) -> wgpu::RenderPassDepthStencilAttachment<'a> {
    wgpu::RenderPassDepthStencilAttachment {
        view: depth_view,
        depth_ops: Some(wgpu::Operations {
            load: wgpu::LoadOp::Clear(1.0),
            store: false,
        }),
        stencil_ops: None,
    }
}

//Application starts
pub fn run() {
    env_logger::init();
    //initialise the environment logger
    let event_loop = EventLoop::new();
    //initialise the window that will be used to display the render
    let window = winit::window::WindowBuilder::new()
        .build(&event_loop)
        .unwrap();
    window.set_title(&*format!("Honours"));
    // initialise the struct "state" thread blocked until state is initialised
    let mut state = pollster::block_on(State::new(
        &window,
    ));

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }

                    _ => {}
                }
            }
        }
        Event::RedrawRequested(_) => {
            state.update();

            match state.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => state.resize(state.init.size),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => {}
    });
}