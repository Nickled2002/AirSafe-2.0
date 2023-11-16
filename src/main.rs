use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
fn main() {
   //========================Check backend====================
   //let instances = wgpu::Instance::new(wgpu::Backends::all());
   //for adapter in instances.enumerate_adapters(wgpu::Backends::all()){
   //     println!("{:?}",adapter.get_info())
   // }
    let event_loop: EventLoop<()> = EventLoop::new();
    let window: Window = Window::new(&event_loop).unwrap();
    window.set_title("my_window");
    env_logger::init();

    event_loop.run(move |event: Event<()>, _,control_flow: &mut ControlFlow|{
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                 ..
            } => *control_flow = ControlFlow::Exit,
            _=>{}
        }
    });
}
