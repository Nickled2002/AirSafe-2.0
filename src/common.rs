#![allow(dead_code)]
use std:: {iter, mem };
use cgmath::{ Matrix, Matrix4, SquareMatrix };
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    window::Window,
    event_loop::{ControlFlow, EventLoop},
};
use bytemuck:: {Pod, Zeroable, cast_slice};

#[path="transforms.rs"]
mod transforms;
#[path="surface_data.rs"]
mod surface;

const ANIMATION_SPEED:f32 = 0.0;//changes if and how fast it rotates
const IS_PERSPECTIVE:bool = true;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Light {
    specular_color : [f32; 4],
    ambient_intensity: f32,
    diffuse_intensity :f32,
    specular_intensity: f32,
    specular_shininess: f32,
    is_two_side: i32,
}

pub fn light(sc:[f32;3], ambient: f32, diffuse: f32, specular: f32, shininess: f32, two_side: i32) -> Light {
    Light {
        specular_color: [sc[0], sc[1], sc[2], 1.0],
        ambient_intensity: ambient,
        diffuse_intensity: diffuse,
        specular_intensity: specular,
        specular_shininess: shininess,
        is_two_side: two_side,
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 4],
    pub normal: [f32; 4],
    pub color: [f32; 4]
}
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CamPos{
    pub x:f32,
    pub y:f32,
    pub z:f32
}



pub fn vertex(p:[f32;3], n:[f32; 3], c:[f32; 3]) -> Vertex {
    Vertex {
        position: [p[0], p[1], p[2], 1.0],
        normal: [n[0], n[1], n[2], 1.0],
        color: [c[0], c[1], c[2], 1.0],
    }
}

pub fn create_vertices(f: &dyn Fn(f32, f32, f32) ->[f32;3], colormap_name: &str, xmin:f32, xmax:f32, zmin:f32, zmax:f32,
                       nx:usize, nz:usize, scale:f32, aspect:f32, ) -> Vec<Vertex> {
    let (pts, yrange) = surface::simple_surface_points(f, xmin, xmax, zmin, zmax, nx, nz, scale, aspect);//normalised points
    let pos = surface::simple_surface_positions(&pts, nx, nz);//use it to get position
    let normal = surface::simple_surface_normals(&pts, nx, nz);//normal
    let color = surface::simple_surface_colors(&pts, nx, nz, yrange, colormap_name);//and color
    let mut data:Vec<Vertex> = Vec::with_capacity(pos.len());
    for i in 0..pos.len() {
        data.push(vertex(pos[i], normal[i], color[i]));//data to vertexes from above
    }
    data.to_vec()
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![0=>Float32x4, 1=>Float32x4, 2=>Float32x4];
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

pub struct State {
    pub init: transforms::InitWgpu,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    uniform_bind_group:wgpu::BindGroup,
    vertex_uniform_buffer: wgpu::Buffer,
    view_mat: Matrix4<f32>,
    project_mat: Matrix4<f32>,
    num_vertices: u32,
    camera: CamPos,
}

impl State {//use vertex data to specify light data and vector data for any kind of 3d surface
    pub async fn new(window: &Window, vertex_data: &Vec<Vertex>, light_data: Light) -> Self {
        let init =  transforms::InitWgpu::init_wgpu(window).await;

        let shader = init.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            //source: wgpu::ShaderSource::Wgsl(include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/examples/ch06/line3d.wgsl")).into()),
        });
        let camera = CamPos{
            x:0.0,
            y:0.0,
            z:0.0,
        };

        let camera_position = (camera.x, camera.y, camera.z).into();
        //let look_direction = (1.0,0.0,0.75).into();
        let look_dir = cgmath::Vector3{
            x: (camera.x+1.0),
            y: (camera.y),
            z: (camera.z+1.0),
        };
        let up_direction = cgmath::Vector3::unit_y();

        let (view_mat, project_mat, _view_project_mat ) =
            transforms::create_view_projection(camera_position, look_dir, up_direction,
                                               init.config.width as f32 / init.config.height as f32, IS_PERSPECTIVE);
        // create vertex uniform buffer
        // model_mat and view_projection_mat will be stored in vertex_uniform_buffer inside the update function
        let vertex_uniform_buffer = init.device.create_buffer(&wgpu::BufferDescriptor{
            label: Some("Vertex Uniform Buffer"),
            size: 192,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // create fragment uniform buffer. here we set eye_position = camera_position and light_position = eye_position
        let fragment_uniform_buffer = init.device.create_buffer(&wgpu::BufferDescriptor{
            label: Some("Fragment Uniform Buffer"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // store light and eye positions
        let light_position:&[f32; 3] = camera_position.as_ref();
        let eye_position:&[f32; 3] = camera_position.as_ref();
        init.queue.write_buffer(&fragment_uniform_buffer, 0, cast_slice(light_position));
        init.queue.write_buffer(&fragment_uniform_buffer, 16,cast_slice(eye_position));

        // create light uniform buffer
        let light_uniform_buffer = init.device.create_buffer(&wgpu::BufferDescriptor{
            label: Some("Light Uniform Buffer"),
            size: 48,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // store light parameters
        init.queue.write_buffer(&light_uniform_buffer, 0, cast_slice(&[light_data]));

        let uniform_bind_group_layout = init.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("Uniform Bind Group Layout"),
        });

        let uniform_bind_group = init.device.create_bind_group(&wgpu::BindGroupDescriptor{
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: vertex_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: fragment_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: light_uniform_buffer.as_entire_binding(),
                },
            ],
            label: Some("Uniform Bind Group"),
        });

        let pipeline_layout = init.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = init.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: init.config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState{
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            //depth_stencil: None,
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let vertex_buffer = init.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: cast_slice(vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let num_vertices = vertex_data.len() as u32;

        Self {
            init,
            pipeline,
            vertex_buffer,
            uniform_bind_group,
            vertex_uniform_buffer,
            view_mat,
            project_mat,
            num_vertices,
            camera,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.init.instance.poll_all(true);
            self.init.size = new_size;
            self.init.config.width = new_size.width;
            self.init.config.height = new_size.height;
            self.init.surface.configure(&self.init.device, &self.init.config);
            self.project_mat = transforms::create_projection(new_size.width as f32 / new_size.height as f32, IS_PERSPECTIVE);
        }
    }

    pub fn plane_move(&mut self, moves: char) {
        match moves {
            'w' => self.camera.x =self.camera.x+0.1,
            's' => self.camera.x =self.camera.x-0.1,
            'a' => self.camera.z = self.camera.z -0.1,
            'd' => self.camera.z = self.camera.z +0.1,
            'q' => self.camera.y=self.camera.y+0.1,
            'e' => self.camera.y=self.camera.y-0.1,
            _ => {}
        }
        let look_direction = (1.0,0.0,0.75).into();
        let up_direction = cgmath::Vector3::unit_y();

        let camera_position = (self.camera.x, self.camera.y, self.camera.z).into();
        let (view_mat,   project_mat, _view_project_mat) =
        transforms::create_view_projection(camera_position, look_direction, up_direction, self.init.config.width as f32 / self.init.config.height as f32, IS_PERSPECTIVE);
        self.view_mat=view_mat;
        self.project_mat=project_mat;
    }

    #[allow(unused_variables)]
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    //render here
    pub fn update(&mut self, dt: std::time::Duration) {
        // update uniform buffer
        let dt = ANIMATION_SPEED * dt.as_secs_f32();
        let model_mat = transforms::create_transforms([0.0,0.0,0.0], [dt.sin(), dt.cos(), 0.0], [2.0, 2.0, 2.0]);
        let view_project_mat = self.project_mat * self.view_mat;

        let normal_mat = model_mat.invert().unwrap().transpose();

        let model_ref:&[f32; 16] = model_mat.as_ref();
        let view_projection_ref:&[f32; 16] = view_project_mat.as_ref();
        let normal_ref:&[f32; 16] = normal_mat.as_ref();

        self.init.queue.write_buffer(&self.vertex_uniform_buffer, 0, cast_slice(model_ref));
        self.init.queue.write_buffer(&self.vertex_uniform_buffer, 64, cast_slice(view_projection_ref));
        self.init.queue.write_buffer(&self.vertex_uniform_buffer, 128, cast_slice(normal_ref));
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        //let output = self.init.surface.get_current_frame()?.output;
        let output = self.init.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let depth_texture = self.init.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.init.config.width,
                height: self.init.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format:wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .init.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.2,
                            g: 0.247,
                            b: 0.314,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                //depth_stencil_attachment: None,
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: false,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.draw(0..self.num_vertices, 0..1);
        }

        self.init.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

pub fn run(vertex_data: &Vec<Vertex>, light_data: Light, colormap_name: &str, title: &str) {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title(&*format!("Honours{}: {}", title, colormap_name));
    let mut state = pollster::block_on(State::new(&window, &vertex_data, light_data));
    let render_start_time = std::time::Instant::now();
//window and event loop was in main
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::KeyboardInput {
                            input:
                            KeyboardInput{
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::W),
                                ..
                            },
                            ..
                        } => {
                            state.plane_move('w');
                        },
                        WindowEvent::KeyboardInput {
                            input:
                            KeyboardInput{
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::S),
                                ..
                            },
                            ..
                        } => {
                            state.plane_move('s');
                        },WindowEvent::KeyboardInput {
                            input:
                            KeyboardInput{
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::A),
                                ..
                            },
                            ..
                        } => {
                            state.plane_move('a');
                        },WindowEvent::KeyboardInput {
                            input:
                            KeyboardInput{
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::D),
                                ..
                            },
                            ..
                        } => {
                            state.plane_move('d');
                        },WindowEvent::KeyboardInput {
                            input:
                            KeyboardInput{
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Q),
                                ..
                            },
                            ..
                        } => {
                            state.plane_move('q');
                        },WindowEvent::KeyboardInput {
                            input:
                            KeyboardInput{
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::E),
                                ..
                            },
                            ..
                        } => {
                            state.plane_move('e');
                        },
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                //virtual_keycode: Some(VirtualKeyCode::W),
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
                let now = std::time::Instant::now();
                let dt = now - render_start_time;
                state.update(dt);

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
        }
    });
}