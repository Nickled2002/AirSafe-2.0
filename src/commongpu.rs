use bytemuck::{cast_slice};
use cgmath::Matrix4;
use std:: iter;
use srtm::Tile;
use wgpu::{util::DeviceExt, VertexBufferLayout};

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use crate::common::CamPos;


#[path="transforms.rs"]
mod transforms;
#[path="common.rs"]
mod common;
pub struct Params{
    resolution: f32,
    octaves: f32,
    persistence: f32,
    lacunarity: f32,
    offsetX: f32,
    offsetZ: f32,
    scale: f32,
    waterLevel: f32,
    mapdata: Vec<Vec<f32>>,
}
struct State {
    init: common::IWgpuInit,
    pipeline: wgpu::RenderPipeline,
    uniform_bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    model_mat: Matrix4<f32>,
    view_mat: Matrix4<f32>,
    project_mat: Matrix4<f32>,
    depth_texture_view: wgpu::TextureView,
    camera: CamPos,
    camlook: CamPos,
    update_buffers: bool,
    aspect_ratio: f32,
    update_buffers_view: bool,
    fps_counter: common::FpsCounter,
    map:Vec<Vec<f32>>,


    cs_pipelines: Vec<wgpu::ComputePipeline>,
    cs_vertex_buffer: wgpu::Buffer,
    cs_index_buffer: wgpu::Buffer,
    cs_uniform_buffers: Vec<wgpu::Buffer>,
    cs_bind_groups: Vec<wgpu::BindGroup>,




    resolution: u32,
    water_level: f32,
    movez: f32,
    movex: f32,
    scale: f32,
    triangles_count: u32,


}

impl State {
    async fn new(window: &Window, sample_count: u32, resolution: u32) -> Self {
        let init = common::IWgpuInit::new(&window, sample_count, None).await;

        let resol = common::round_to_multiple(resolution, 8);
        let vertices_count = resol * resol;
        let triangles_count = 6 * (resol - 1) * (resol - 1);
        println!("resolution = {}", resol);

        let shader = init
            .device
            .create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let cs_noise = include_str!("noise.wgsl");
        let cs_terrain = include_str!("terrain_comp.wgsl");
        let cs_combine = [cs_noise, cs_terrain].join("\n");
        let cs_comp = init
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Compute Shader"),
                source: wgpu::ShaderSource::Wgsl(cs_combine.into()),
            });

        let cs_indices = init
            .device
            .create_shader_module(wgpu::include_wgsl!("indices_comp.wgsl"));

        // uniform data
        let model_mat = transforms::create_transforms(
            [-0.65 * resol as f32, 5.0, -0.5 * resol as f32],
            [0.0, 0.0, 0.0],
            [1.0, 20.0, 1.0],
        );
        let camera = CamPos{
            x:0.0,
            y:20.0,
            z:0.0,
        };
        let camlook = CamPos{
            x:0.0,
            y:0.0,
            z:-30.0,
        };

        let camera_position = (camera.x, camera.y, camera.z).into();
        let look_direction = (camlook.x, camlook.y, camlook.z).into();
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
        let (vert_bind_group_layout, vert_bind_group) = common::create_bind_group(
            &init.device,
            vec![wgpu::ShaderStages::VERTEX],
            &[vert_uniform_buffer.as_entire_binding()],
        );

        let vertex_buffer_layout = VertexBufferLayout {
            array_stride: 32,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4], //pos, col,
        };

        let pipeline_layout = init
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&vert_bind_group_layout],
                push_constant_ranges: &[],
            });

        let mut ppl = common::IRenderPipeline {
            shader: Some(&shader),
            pipeline_layout: Some(&pipeline_layout),
            vertex_buffer_layout: &[vertex_buffer_layout],
            ..Default::default()
        };
        let pipeline = ppl.new(&init);

        let depth_texture_view = common::create_depth_view(&init);

        // create compute pipeline for indices
        let cs_index_buffer = init.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index Buffer"),
            size: 4 * triangles_count as u64,
            usage: wgpu::BufferUsages::INDEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });



        let cs_index_uniform_buffer = init.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index Uniform Buffer"),
            size: 4,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        init.queue
            .write_buffer(&cs_index_uniform_buffer, 0, cast_slice(&[resol]));

        let (cs_index_bind_group_layout, cs_index_bind_group) = common::create_bind_group_storage(
            &init.device,
            vec![
                wgpu::ShaderStages::COMPUTE,
                wgpu::ShaderStages::COMPUTE,
            ],
            vec![
                wgpu::BufferBindingType::Storage { read_only: false },
                wgpu::BufferBindingType::Uniform,
            ],
            &[
                cs_index_buffer.as_entire_binding(),
                cs_index_uniform_buffer.as_entire_binding(),
            ],
        );

        let cs_index_pipeline_layout =
            init.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Compute Index Pipeline Layout"),
                    bind_group_layouts: &[&cs_index_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let cs_index_pipeline =
            init.device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("Compute Index Pipeline"),
                    layout: Some(&cs_index_pipeline_layout),
                    module: &cs_indices,
                    entry_point: "cs_main",
                });

        // create compute pipeline for terrain
        let cs_vertex_buffer = init.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: 32 * vertices_count as u64,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mapdata = State::find_world_map(54, 4);

        let params = [resol as f32, 5.0, 0.5, 2.0, 0.0, 0.0, 50.0, 0.2];
        let cs_vertex_uniform_buffer =
            init.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Uniform Buffer"),
                    contents: bytemuck::cast_slice(&params),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let (cs_vertex_bind_group_layout, cs_vertex_bind_group) = common::create_bind_group_storage(
            &init.device,
            vec![
                wgpu::ShaderStages::COMPUTE,
                wgpu::ShaderStages::COMPUTE,
            ],
            vec![
                wgpu::BufferBindingType::Storage { read_only: false },
                wgpu::BufferBindingType::Uniform,
            ],
            &[
                cs_vertex_buffer.as_entire_binding(),
                cs_vertex_uniform_buffer.as_entire_binding(),
            ],
        );

        let cs_pipeline_layout =
            init.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Compute Pipeline Layout"),
                    bind_group_layouts: &[&cs_vertex_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let cs_pipeline = init
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Compute Pipeline"),
                layout: Some(&cs_pipeline_layout),
                module: &cs_comp,
                entry_point: "cs_main",
            });

        Self {
            init,
            pipeline: pipeline,
            uniform_bind_group: vert_bind_group,
            uniform_buffer:vert_uniform_buffer,

            cs_pipelines: vec![cs_index_pipeline,cs_pipeline],
            cs_vertex_buffer:cs_vertex_buffer,
            cs_index_buffer:cs_index_buffer,
            cs_uniform_buffers:vec![cs_index_uniform_buffer,cs_vertex_uniform_buffer],
            cs_bind_groups:vec![cs_index_bind_group, cs_vertex_bind_group],

            model_mat,
            view_mat,
            project_mat,
            depth_texture_view,
            map:vec![],
            //terrain,
            camera,
            camlook,
            update_buffers: false,
            update_buffers_view: false,
            aspect_ratio: 20.0,
            resolution: resol,
            water_level: 0.2,
            movez: 0.0,
            movex: 0.0,
            scale: 50.0,
            triangles_count,

            fps_counter: common::FpsCounter::default(),

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

            self.depth_texture_view = common::create_depth_view(&self.init);
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
                VirtualKeyCode::D => {
                    self.movex += 1.0;
                    self.update_buffers = true;

                    true
                }
                VirtualKeyCode::A => {
                    self.movex -= 1.0;
                    self.update_buffers = true;
                    true
                }
                VirtualKeyCode::S => {
                    self.movez -= 1.0;
                    self.update_buffers = true;
                    true
                }
                VirtualKeyCode::W => {
                    self.movez += 1.0;
                    self.update_buffers = true;
                    true
                }
                VirtualKeyCode::Z => {
                    self.scale += 1.0;
                    self.update_buffers = true;
                    println!("scale = {}", self.scale);
                    true
                }
                VirtualKeyCode::X => {
                    self.scale -= 1.0;
                    if self.scale < 1.0 {
                        self.scale = 1.0;
                    }
                    self.update_buffers = true;
                    println!("scale = {}", self.scale);
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
                    -0.65 * self.resolution as f32,
                    5.0,
                    -0.5 * self.resolution as f32,
                ],
                [0.0, 0.0, 0.0],
                [1.0, self.aspect_ratio, 1.0],
            );
            let mvp_mat = self.project_mat * self.view_mat * self.model_mat;
            self.init.queue.write_buffer(
                &self.uniform_buffer,
                0,
                cast_slice(mvp_mat.as_ref() as &[f32; 16]),
            );
            let params = [
                self.resolution as f32,
                5.0,
                0.5,
                2.0,
                self.movex,
                self.movez,
                self.scale,
                self.water_level,
            ];
            self.init
                .queue
                .write_buffer(&self.cs_uniform_buffers[1], 0, cast_slice(&params));
            self.update_buffers = false;
        }
        if self.update_buffers_view {
            let mvp_mat = self.project_mat * self.view_mat * self.model_mat;
            self.init.queue.write_buffer(
                &self.uniform_buffer,
                0,
                cast_slice(mvp_mat.as_ref() as &[f32; 16]),
            );
            self.update_buffers_view = false;
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

        // compute pass for indices
        {
            let mut cs_index_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Index Pass"),
            });
            cs_index_pass.set_pipeline(&self.cs_pipelines[0]);
            cs_index_pass.set_bind_group(0, &self.cs_bind_groups[0], &[]);
            cs_index_pass.dispatch_workgroups(self.resolution / 8, self.resolution / 8, 1);
        }

        // compute pass for vertices
        {
            let mut cs_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
            });
            cs_pass.set_pipeline(&self.cs_pipelines[1]);
            cs_pass.set_bind_group(0, &self.cs_bind_groups[1], &[]);
            cs_pass.dispatch_workgroups(self.resolution / 8, self.resolution / 8, 1);
        }

        // render pass
        {
            let color_attach = common::create_color_attachment(&view);
            let depth_attachment = common::create_depth_stencil_attachment(&self.depth_texture_view);

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(color_attach)],
                depth_stencil_attachment: Some(depth_attachment),
            });

                render_pass.set_pipeline(&self.pipeline);
                render_pass.set_vertex_buffer(0, self.cs_vertex_buffer.slice(..));
                render_pass.set_index_buffer(self.cs_index_buffer.slice(..), wgpu::IndexFormat::Uint32, );
                render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                render_pass.draw_indexed(0..self.triangles_count, 0, 0..1);
        }
        self.fps_counter.print_fps(5);
        self.init.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
    fn find_world_map(lat:u32,long:u32) -> Vec<Vec<f32>>{
        let mut map: Vec<Vec<f32>> = vec![];
        let mut height_min = f32::MAX;
        let mut height_max = f32::MIN;
        let worldmap: Tile = Tile::from_file("src/Scotlandhgt/N".to_owned() + &*lat.to_string() +"W00"+ &*long.to_string() +".hgt").unwrap();

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

        map
    }
}


pub fn run() {
    let mut sample_count = 1 as u32;
    let mut resolution = 1024u32;
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        sample_count = args[1].parse::<u32>().unwrap();
    }
    if args.len() > 2 {
        resolution = args[2].parse::<u32>().unwrap();
    }

    env_logger::init();
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .build(&event_loop)
        .unwrap();
    window.set_title(&*format!("Honours"));

    let mut state = pollster::block_on(State::new(&window, sample_count, resolution, ));

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
