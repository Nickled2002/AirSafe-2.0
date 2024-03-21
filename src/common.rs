#![allow(dead_code)]
use std:: {iter, mem };
use std::f32::consts::PI;
use cgmath::Matrix4;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    window::Window,
    event_loop::{ControlFlow, EventLoop},
};
use bytemuck::cast_slice;
use wgpu::VertexBufferLayout;
use std::time::{Instant, Duration};
use std::collections::VecDeque;
use rand::{SeedableRng, rngs::StdRng,distributions::{Distribution, Uniform}};


#[path="transforms.rs"]
mod transforms;
#[path="surface_data.rs"]
mod surface;
#[path="texture_data.rs"]
mod texture;


fn create_compute_texture_bind_group_layout(
    device: &wgpu::Device
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
        label: Some("Compute Texture Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
        ]
    })
}

pub fn create_compute_texture_bind_group(
    device: &wgpu::Device,
    texture_view: &wgpu::TextureView
) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
    let layout = create_compute_texture_bind_group_layout(device);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor{
        label: Some("Compute Texture Bind Group"),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            }
        ]
    });
    (layout, bind_group)
}

fn create_texture_bind_group_layout(
    device: &wgpu::Device,
    img_files:Vec<&str>
) -> wgpu::BindGroupLayout {
    let mut entries:Vec<wgpu::BindGroupLayoutEntry> = vec![];
    for i in 0..img_files.len() {
        entries.push( wgpu::BindGroupLayoutEntry {
            binding: (2*i) as u32,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            count: None,
        });
        entries.push(wgpu::BindGroupLayoutEntry {
            binding: (2*i+1) as u32,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        })
    }

    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &entries,
        label: Some("texture_bind_group_layout"),
    })
}

pub fn create_texture_bind_group(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    img_files:Vec<&str>,
    u_mode:wgpu::AddressMode,
    v_mode:wgpu::AddressMode
) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
    let mut img_textures:Vec<texture::ITexture> = vec![];
    let mut entries:Vec<wgpu::BindGroupEntry<'_>> = vec![];
    for i in 0..img_files.len() {
        img_textures.push(texture::ITexture::create_texture_data(device, queue, img_files[i], u_mode, v_mode).unwrap());
    }
    for i in 0..img_files.len() {
        entries.push( wgpu::BindGroupEntry {
            binding: (2*i) as u32,
            resource: wgpu::BindingResource::TextureView(&img_textures[i].view),
        });
        entries.push( wgpu::BindGroupEntry {
            binding: (2*i + 1) as u32,
            resource: wgpu::BindingResource::Sampler(&img_textures[i].sampler),
        })
    }

    let layout = create_texture_bind_group_layout(device, img_files);

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor{
        layout: &layout,
        label: Some("texture_bind_group"),
        entries: &entries
    });
    (layout, bind_group)
}

pub fn create_texture_store_bind_group(
    device: &wgpu::Device,
    store_texture: &texture::ITexture
) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
    let layout = create_texture_bind_group_layout(device, vec!["None"]);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor{
        layout: &layout,
        label: Some("texture_bind_group"),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&store_texture.view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&store_texture.sampler),
            },
        ]
    });
    (layout, bind_group)
}

pub fn create_shadow_texture_view(init: &IWgpuInit, width:u32, height:u32) -> wgpu::TextureView {
    let shadow_depth_texture = init.device.create_texture(&wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: init.sample_count,
        dimension: wgpu::TextureDimension::D2,
        format:wgpu::TextureFormat::Depth24Plus,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        label: None,
        view_formats: &[],
    });

    shadow_depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
}

pub fn create_bind_group_layout_storage(
    device: &wgpu::Device,
    shader_stages: Vec<wgpu::ShaderStages>,
    binding_types: Vec<wgpu::BufferBindingType>
) -> wgpu::BindGroupLayout {
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

pub fn create_bind_group_storage(
    device: &wgpu::Device,
    shader_stages: Vec<wgpu::ShaderStages>,
    binding_types: Vec<wgpu::BufferBindingType>,
    resources: &[wgpu::BindingResource<'_>]
) -> ( wgpu::BindGroupLayout, wgpu::BindGroup) {
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

pub fn create_bind_group_layout(
    device: &wgpu::Device,
    shader_stages: Vec<wgpu::ShaderStages>
) -> wgpu::BindGroupLayout {
    let mut entries = vec![];

    for i in 0..shader_stages.len() {
        entries.push(wgpu::BindGroupLayoutEntry {
            binding: i as u32,
            visibility: shader_stages[i],
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
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

pub fn create_bind_group(
    device: &wgpu::Device,
    shader_stages: Vec<wgpu::ShaderStages>,
    resources: &[wgpu::BindingResource<'_>]
) -> ( wgpu::BindGroupLayout, wgpu::BindGroup) {
    let entries: Vec<_> = resources.iter().enumerate().map(|(i, resource)| {
        wgpu::BindGroupEntry {
            binding: i as u32,
            resource: resource.clone(),
        }
    }).collect();

    let layout = create_bind_group_layout(device, shader_stages);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &layout,
        entries: &entries,
        label: Some("Uniform Bind Group"),
    });

    (layout, bind_group)
}


pub fn create_color_attachment<'a>(texture_view: &'a wgpu::TextureView) -> wgpu::RenderPassColorAttachment<'a> {
    wgpu::RenderPassColorAttachment {
        view: texture_view,
        resolve_target: None,
        ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            store: true,
        },
    }
}




pub fn create_depth_view(init: &IWgpuInit) -> wgpu::TextureView {
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

pub struct IRenderPipeline<'a> {
    pub shader: Option<&'a wgpu::ShaderModule>,
    pub vs_shader: Option<&'a wgpu::ShaderModule>,
    pub fs_shader: Option<&'a wgpu::ShaderModule>,
    pub vertex_buffer_layout: &'a [wgpu::VertexBufferLayout<'a>],
    pub pipeline_layout: Option<&'a wgpu::PipelineLayout>,
    pub topology: wgpu::PrimitiveTopology,
    pub strip_index_format: Option<wgpu::IndexFormat>,
    pub cull_mode: Option<wgpu::Face>,
    pub is_depth_stencil: bool,
    pub vs_entry: String,
    pub fs_entry: String,
}

impl Default for IRenderPipeline<'_> {
    fn default() -> Self {
        Self {
            shader: None,
            vs_shader: None,
            fs_shader: None,
            vertex_buffer_layout: &[],
            pipeline_layout: None,
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            cull_mode: None,
            is_depth_stencil: true,
            vs_entry: String::from("vs_main"),
            fs_entry: String::from("fs_main"),
        }
    }
}

impl IRenderPipeline<'_> {
    pub fn new(&mut self, init: &IWgpuInit) -> wgpu::RenderPipeline {
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


pub struct IWgpuInit {
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub sample_count: u32,
}

impl IWgpuInit {
    pub async fn new(window: &Window, sample_count:u32, limits:Option<wgpu::Limits>) -> Self {
        let limits_device = limits.unwrap_or(wgpu::Limits::default());

        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        /*let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12,
            //dx12_shader_compiler: Default::default(),
            dx12_shader_compiler: {
                wgpu::Dx12Compiler::Dxc {
                    dxil_path: Some(PathBuf::from(r"assets/dxil.dll")),
                    dxc_path: Some(PathBuf::from(r"assets/dxcompiler.dll")),
                }
            },
        });*/

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

        Self {
            instance,
            surface,
            adapter,
            device,
            queue,
            config,
            size,
            sample_count,
        }
    }
}

// region: utility
/*pub struct FpsCounter {
    fps: u32,
    frame_count: u32,
    last_frame_time: Instant,
    fps_timer: Instant,
    min_frame_time: Duration,
    t0: Instant,
}

impl FpsCounter {
    pub fn new() -> Self {
        Self {
            frame_count: 0,
            fps: 0,
            last_frame_time: Instant::now(),
            fps_timer: Instant::now(),
            t0: Instant::now(),
            min_frame_time: Duration::from_secs(0),
        }
    }

    pub fn calculate_fps(&mut self) -> (u32, f32) {
        self.frame_count += 1;

        let now = Instant::now();
        let frame_time = now.duration_since(self.last_frame_time);
        self.last_frame_time = now;

        // Update the minimum frame time
        if frame_time < self.min_frame_time || self.min_frame_time == Duration::from_secs(0) {
            self.min_frame_time = frame_time;
        }

        let elapsed_time = self.fps_timer.elapsed();
        if elapsed_time >= Duration::from_secs(1) {
            self.fps = self.frame_count;
            self.frame_count = 0;
            self.fps_timer = Instant::now();

            // Reset the minimum frame time at the start of each FPS calculation interval
            self.min_frame_time = Duration::from_secs(0);
        }
        (self.fps, self.min_frame_time.as_secs_f32()*1000.0)
    }

    pub fn print_fps(&mut self, interval:u64) {
        let elapsed_time = self.t0.elapsed();
        let (fps, render_time) = self.calculate_fps();
        if elapsed_time >= Duration::from_secs(interval) && render_time > 0.0 {
            println!("FPS = {}, Rendering time = {}", fps, render_time);
            self.t0 = Instant::now();
        }
    }
}*/



#[derive(Debug)]
pub struct FpsCounter {
    last_second_frames: VecDeque<Instant>,
    last_print_time: Instant,
}

impl Default for FpsCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl FpsCounter {
    // Creates a new FpsCounter.
    pub fn new() -> Self {
        Self {
            last_second_frames: VecDeque::with_capacity(128),
            last_print_time: Instant::now(),
        }
    }

    // updates the fps counter and print fps.
    pub fn print_fps(&mut self, interval:u64) {
        let now = Instant::now();
        let a_second_ago = now - Duration::from_secs(1);

        while self.last_second_frames.front().map_or(false, |t| *t < a_second_ago) {
            self.last_second_frames.pop_front();
        }

        self.last_second_frames.push_back(now);

        // Check if the interval seconds have passed since the last print time
        if now - self.last_print_time >= Duration::from_secs(interval) {
            let fps = self.last_second_frames.len();
            println!("FPS: {}", fps);
            self.last_print_time = now;
        }
    }
}


pub fn round_to_multiple(any_number: u32, rounded_number: u32) -> u32 {
    num::integer::div_ceil(any_number, rounded_number) * rounded_number
}

pub fn seed_random_number(seed:u64) -> f32 {
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);
    let distribution = Uniform::new(0.0, 1.0);
    distribution.sample(&mut rng) as f32
}

// endregion: utility

struct State {
    init: IWgpuInit,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    model_mat: Matrix4<f32>,
    view_mat: Matrix4<f32>,
    project_mat: Matrix4<f32>,
    depth_texture_view: wgpu::TextureView,
    indices_lens: u32,
    plot_type: u32,

    terrain: surface::ITerrain,
    update_buffers: bool,
    aspect_ratio: f32,
    fps_counter: FpsCounter,
}

impl State {
    async fn new(
        window: &Window,
        width: u32,
        height: u32,
        colormap_name: &str,
    ) -> Self {
        let init = IWgpuInit::new(&window,1, None).await;

        let shader = init
            .device
            .create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        // uniform data
        let model_mat = transforms::create_transforms(
            [-0.65 * width as f32, 5.0, -0.5 * height as f32],
            [0.0, PI / 15.0, 0.0],
            [1.0, 10.0, 1.0],
        );

        let camera_position = (40.0, 50.0, 60.0).into();
        let look_direction = (0.0, 0.0, 0.0).into();
        let up_direction = cgmath::Vector3::unit_y();

        let (view_mat, project_mat, vp_mat) = transforms::create_view_projection(
            camera_position,
            look_direction,
            up_direction,
            init.config.width as f32 / init.config.height as f32,
        );

        let mvp_mat = vp_mat * model_mat;

        // create vertex uniform buffers
        let vert_uniform_buffer =
            init.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Uniform Buffer"),
                    contents: cast_slice(mvp_mat.as_ref() as &[f32; 16]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        // uniform bind group for vertex shader
        let (vert_bind_group_layout, vert_bind_group) = create_bind_group(
            &init.device,
            vec![wgpu::ShaderStages::VERTEX],
            &[vert_uniform_buffer.as_entire_binding()],
        );

        let vertex_buffer_layout = VertexBufferLayout {
            array_stride: mem::size_of::<surface::Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3], // pos, col
        };

        let pipeline_layout = init
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&vert_bind_group_layout],
                push_constant_ranges: &[],
            });

        let mut ppl = IRenderPipeline {
            shader: Some(&shader),
            pipeline_layout: Some(&pipeline_layout),
            vertex_buffer_layout: &[vertex_buffer_layout],
            ..Default::default()
        };
        let pipeline = ppl.new(&init);


        let depth_texture_view = create_depth_view(&init);

        let mut terrain = surface::ITerrain {
            scale: 50.0,
            colormap_name: colormap_name.to_string(),
            width,
            height,
            ..Default::default()
        };
        let vertex_data = terrain.create_terrain_data();
        let index_data = terrain.create_indices(width, height);

        let vertex_buffer = init
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: cast_slice(&vertex_data),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        let index_buffer = init
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&index_data),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            });


        Self {
            init,
            pipeline,
            vertex_buffer,
            index_buffer,
            uniform_bind_group:vert_bind_group,
            uniform_buffer:vert_uniform_buffer,
            model_mat,
            view_mat,
            project_mat,
            depth_texture_view,
            indices_lens: index_data.len() as u32,
            plot_type: 1,

            terrain,
            update_buffers: false,
            aspect_ratio: 10.0,
            fps_counter: FpsCounter::default(),
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.init.size = new_size;
            self.init.config.width = new_size.width;
            self.init.config.height = new_size.height;
            self.init
                .surface
                .configure(&self.init.device, &self.init.config);

            self.project_mat =
                transforms::create_projection(new_size.width as f32 / new_size.height as f32, true);
            let mvp_mat = self.project_mat * self.view_mat * self.model_mat;
            self.init.queue.write_buffer(
                &self.uniform_buffer,
                0,
                cast_slice(mvp_mat.as_ref() as &[f32; 16]),
            );

            self.depth_texture_view = create_depth_view(&self.init);
        }
    }

    #[allow(unused_variables)]
    fn input(&mut self, event: &WindowEvent) -> bool {
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
                VirtualKeyCode::Space => {
                    self.plot_type = (self.plot_type + 1) % 3;
                    true
                }
                VirtualKeyCode::Q => {
                    self.terrain.scale += 1.0;
                    self.update_buffers = true;
                    println!("scale = {}", self.terrain.scale);
                    true
                }
                VirtualKeyCode::A => {
                    self.terrain.scale -= 1.0;
                    if self.terrain.scale < 1.0 {
                        self.terrain.scale = 1.0;
                    }
                    self.update_buffers = true;
                    println!("scale = {}", self.terrain.scale);
                    true
                }
                VirtualKeyCode::W => {
                    self.terrain.octaves += 1;
                    self.update_buffers = true;
                    println!("octaves = {}", self.terrain.octaves);
                    true
                }
                VirtualKeyCode::S => {
                    self.terrain.octaves -= 1;
                    if self.terrain.octaves < 1 {
                        self.terrain.octaves = 1;
                    }
                    self.update_buffers = true;
                    println!("octaves = {}", self.terrain.octaves);
                    true
                }
                VirtualKeyCode::E => {
                    self.terrain.offsets[0] += 1.0;
                    self.update_buffers = true;
                    println!("offset_x = {}", self.terrain.offsets[0]);
                    true
                }
                VirtualKeyCode::D => {
                    self.terrain.offsets[0] -= 1.0;
                    self.update_buffers = true;
                    println!("offset_x = {}", self.terrain.offsets[0]);
                    true
                }
                VirtualKeyCode::R => {
                    self.terrain.offsets[1] += 1.0;
                    self.update_buffers = true;
                    println!("offset_z = {}", self.terrain.offsets[1]);
                    true
                }
                VirtualKeyCode::F => {
                    self.terrain.offsets[1] -= 1.0;
                    self.update_buffers = true;
                    println!("offset_z = {}", self.terrain.offsets[1]);
                    true
                }
                VirtualKeyCode::T => {
                    self.aspect_ratio += 1.0;
                    self.update_buffers = true;
                    println!("aspect_ratio = {}", self.aspect_ratio);
                    true
                }
                VirtualKeyCode::G => {
                    self.aspect_ratio -= 1.0;
                    self.update_buffers = true;
                    println!("aspect_ratio = {}", self.aspect_ratio);
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
            self.model_mat = transforms::create_transforms(
                [
                    -0.65 * self.terrain.width as f32,
                    5.0,
                    -0.5 * self.terrain.height as f32,
                ],
                [0.0, PI / 15.0, 0.0],
                [1.0, self.aspect_ratio, 1.0],
            );
            let mvp_mat = self.project_mat * self.view_mat * self.model_mat;
            self.init.queue.write_buffer(
                &self.uniform_buffer,
                0,
                cast_slice(mvp_mat.as_ref() as &[f32; 16]),
            );

            let vertex_data = self.terrain.create_terrain_data();
            self.init
                .queue
                .write_buffer(&self.vertex_buffer, 0, cast_slice(&vertex_data));
            self.update_buffers = false;
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        //let output = self.init.surface.get_current_frame()?.output;
        let output = self.init.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.init
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            let color_attach = create_color_attachment(&view);
            let depth_attachment = create_depth_stencil_attachment(&self.depth_texture_view);

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(color_attach)],
                depth_stencil_attachment: Some(depth_attachment),
            });


            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.draw_indexed(0..self.indices_lens, 0, 0..1);

        }

        self.fps_counter.print_fps(5);
        self.init.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
/*
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CamPos{
    pub x:f32,
    pub y:f32,
    pub z:f32
}

    pub fn plane_move(&mut self, moves: char) {
        match moves {
            'w' => {self.camera.x = self.camera.x+0.1;
                if (self.camera.x>2.0){
                    self.camera.x =0.0;
                }
            },
            's' => self.camera.x = self.camera.x-0.1,
            'a' => self.camera.z = self.camera.z -0.1,
            'd' => self.camera.z = self.camera.z +0.1,
            'q' => self.camera.y = self.camera.y+0.1,
            'e' => self.camera.y = self.camera.y-0.1,
            'r' => self.camlook.y = self.camlook.y+0.1,
            'f' => self.camlook.y = self.camlook.y-0.1,
            'z' => self.camlook.x = self.camlook.x +0.1,
            'x' => self.camlook.x = self.camlook.x -0.1,
            //'c' => self.camlook.z = self.camlook.z+0.1,
            //'v' => self.camlook.z = self.camlook.z-0.1,
            _ => {}
        }
        let look_direction = (self.camlook.x,self.camlook.y,self.camlook.z).into();
        let up_direction = cgmath::Vector3::unit_y();

        let camera_position = (self.camera.x, self.camera.y, self.camera.z).into();
        let (view_mat,   project_mat, _view_project_mat) =
        transforms::create_view_projection(camera_position, look_direction, up_direction, self.init.config.width as f32 / self.init.config.height as f32, IS_PERSPECTIVE);
        self.view_mat=view_mat;
        self.project_mat=project_mat;
    }
*/

pub fn run( width: u32, height: u32, colormap_name: &str, ) {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .build(&event_loop)
        .unwrap();
    window.set_title(&*format!("Honours"));

    let mut state = pollster::block_on(State::new(
        &window,
        width,
        height,
        colormap_name,
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