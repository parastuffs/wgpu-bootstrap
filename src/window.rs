use crate::{application::Application, context::Context};
use std::time::Instant;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window as WinitWindow, WindowBuilder},
};

pub struct Window {
    event_loop: EventLoop<()>,
    window: WinitWindow,
    context: Context,
}

impl Window {
    pub fn new() -> Self {
        env_logger::init();

        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();
        let context = pollster::block_on(Context::new(&window));

        Self {
            event_loop,
            window,
            context,
        }
    }

    pub fn run<T>(self, mut application: T)
    where
        T: Application + 'static,
    {
        let mut now = Instant::now();
        let window = self.window;
        let mut context = self.context;
        self.event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => {
                    if !application.input(&event, &context) {
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
                                println!("RESIZE {}", physical_size.width);
                                if physical_size.width == u32::MAX
                                    || physical_size.height == u32::MAX
                                {
                                    // HACK to fix a bug on Macos 14
                                    return;
                                }
                                context.resize(*physical_size);
                                application.resize(&context);
                            }
                            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                // new_inner_size is &mut so w have to dereference it twice
                                context.resize(**new_inner_size);
                                application.resize(&context);
                            }
                            _ => {}
                        }
                    }
                }
                Event::RedrawRequested(window_id) if window_id == window.id() => {
                    let new_now = Instant::now();
                    let delta_time = new_now.duration_since(now).as_micros();
                    now = new_now;
                    application.update(&context, delta_time as f32 / 1000000.0);
                    match application.render(&context) {
                        Ok(_) => {}
                        // Reconfigure the surface if it's lost or outdated
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            context.resize(context.size)
                        }
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        // We're ignoring timeouts
                        Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                    }
                }
                Event::MainEventsCleared => {
                    // RedrawRequested will only trigger once, unless we manually
                    // request it.
                    window.request_redraw();
                }
                _ => {}
            }
        });
    }

    pub fn get_context(&self) -> &Context {
        return &self.context;
    }
}
