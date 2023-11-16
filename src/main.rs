use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use std::borrow::Cow;
use std::future::Future;
use winit::event::VirtualKeyCode::N;

pub async fn run(event_loop: EventLoop<()>, window: Window){
    let size = window.inner_size();
    let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);
    let surface = unsafe { instance.create_surface(&window)};
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false
        })
        .await
        .expect("Failed to find an apropriate adapter");

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None,
        )
        .await
        .expect("Failed to create device");

    let format = surface.get_preferred_format(&adapter).unwrap();
    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    };
    surface.configure(&device, &config);

    let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor{
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });
    //Placegolder we dont use gpu buffer
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });
    //Describes the actions the GPU will perform when acting on a set of data
    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState{
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState{
            module: &shader,
            entry_point: "fs_main",
            targets: &[format.into()],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    event_loop.run(move |event,_, control_flow| {
        let _ =(&instance, &adapter, &shader, &pipeline_layout);
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                //Recreate surface with the new size
                config.width = size.width;
                config.height = size.height;
                surface.configure(&device, &config);
            }
            Event::RedrawRequested(_) => {
                let frame = surface.get_current_texture().unwrap();
                let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {label: None});
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
                        label: None,
                        color_attachments: &[wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations{
                                load: wgpu::LoadOp::Clear(wgpu::Color {r: 0.05, g:0.062, b:0.08, a:1.0}),
                                store: true,
                            },
                        }],
                        depth_stencil_attachment: None, //set render for 3d stuff
                    });
                    rpass.set_pipeline(&render_pipeline);
                    rpass.draw(0..3, 0..1);
                }

                queue.submit(Some(encoder.finish()));
                frame.present();
            }
            Event::WindowEvent {
                event:WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });

}
fn main() {
   //========================Check backend EP1====================
   //let instances = wgpu::Instance::new(wgpu::Backends::all());
   //for adapter in instances.enumerate_adapters(wgpu::Backends::all()){
   //     println!("{:?}",adapter.get_info())
   // }
    //====================Window Generation EP2=================
    //let event_loop: EventLoop<()> = EventLoop::new();
    //let window: Window = Window::new(&event_loop).unwrap();
    //window.set_title("my_window");
    //env_logger::init();

    //event_loop.run(move |event: Event<()>, _,control_flow: &mut ControlFlow|{
    //   *control_flow = ControlFlow::Wait;
    //    match event {
    //        Event::WindowEvent {
    //            event: WindowEvent::CloseRequested,
    //            ..
    //       } => *control_flow = ControlFlow::Exit,
    //        _=>{}
    //    }
    //});

    let event_loop = EventLoop::new();
    let window =winit::window::Window::new(&event_loop).unwrap();
    window.set_title("wgpu03");
    env_logger::init();
    pollster::block_on(run(event_loop,window));

}
