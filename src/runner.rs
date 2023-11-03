use crate::context::Context;
//use crate::myapp::MyApp;
// use crate::app::App;
use egui::FontDefinitions;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use std::time::Instant;
use wgpu::TextureView;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
};

pub trait App {
    #[allow(unused_variables)]
    fn render(&mut self, context: &mut Context, view: &TextureView) {}

    #[allow(unused_variables)]
    fn render_gui(&mut self, context: &egui::Context) {}

    #[allow(unused_variables)]
    fn update(&mut self, context: &mut Context, delta_time: f32) {}

    #[allow(unused_variables)]
    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }
}

pub struct Runner {
    event_loop: EventLoop<()>,
    pub context: Context,
}

impl Runner {
    pub async fn new() -> Self {
        env_logger::init();
        let event_loop = EventLoop::new();
        let context = Context::new(&event_loop).await;

        Self {
            event_loop,
            context,
        }
    }

    pub fn start(mut self, mut app: impl App + 'static) {
        let mut platform = Platform::new(PlatformDescriptor {
            physical_width: self.context.size().width,
            physical_height: self.context.size().height,
            scale_factor: self.context.window().scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });

        // We use the egui_wgpu_backend crate as the render backend.
        let mut egui_rpass =
            RenderPass::new(self.context.device(), self.context.config().format, 1);

        let start_time = Instant::now();
        let mut last: Option<Instant> = None;
        self.event_loop.run(move |event, _, control_flow| {
            control_flow.set_poll();

            platform.handle_event(&event);

            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.context.window().id() => {
                    if !app.input(event) {
                        match event {
                            WindowEvent::CloseRequested => {
                                println!("The close button was pressed; stopping");
                                control_flow.set_exit();
                            }
                            WindowEvent::Resized(physical_size) => {
                                // Hack for MacOS 14 Bug
                                if physical_size.width < u32::MAX {
                                    self.context.resize(*physical_size);
                                }
                            }
                            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                println!("Scale Factor Changed");
                                self.context.resize(**new_inner_size);
                            }
                            _ => {}
                        }
                    }
                }
                Event::MainEventsCleared => {
                    self.context.window().request_redraw();
                }
                Event::RedrawRequested(_) => {
                    let now = Instant::now();
                    let delta_time = match last {
                        Some(last) => now.duration_since(last).as_secs_f32(),
                        None => 0.0,
                    };
                    last = Some(now);
                    platform.update_time(now.duration_since(start_time).as_secs_f64());

                    app.update(&mut self.context, delta_time);

                    let output = self.context.surface().get_current_texture();

                    let frame = match output {
                        Ok(texture) => texture,
                        // Reconfigure the surface if lost
                        Err(wgpu::SurfaceError::Lost) => {
                            self.context.resize(*self.context.size());
                            return;
                        }
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            eprintln!("Out Of Memory");
                            control_flow.set_exit();
                            return;
                        }
                        // All other errors (Outdated, Timeout) should be resolved by the next frame
                        Err(e) => {
                            eprintln!("{:?}", e);
                            return;
                        }
                    };

                    let view = frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());

                    app.render(&mut self.context, &view);

                    // platform.update_time(start_time.elapsed().as_secs_f64());

                    // Begin to draw the UI frame.
                    platform.begin_frame();

                    app.render_gui(&platform.context());

                    let full_output = platform.end_frame(Some(self.context.window()));
                    let paint_jobs = platform.context().tessellate(full_output.shapes);

                    let mut encoder = self.context.device().create_command_encoder(
                        &wgpu::CommandEncoderDescriptor {
                            label: Some("egui encoder"),
                        },
                    );

                    // Upload all resources for the GPU.
                    let screen_descriptor = ScreenDescriptor {
                        physical_width: self.context.size().width,
                        physical_height: self.context.size().height,
                        scale_factor: self.context.window().scale_factor() as f32,
                    };
                    let tdelta: egui::TexturesDelta = full_output.textures_delta;
                    egui_rpass
                        .add_textures(self.context.device(), self.context.queue(), &tdelta)
                        .expect("add texture ok");
                    egui_rpass.update_buffers(
                        self.context.device(),
                        self.context.queue(),
                        &paint_jobs,
                        &screen_descriptor,
                    );

                    egui_rpass
                        .execute(&mut encoder, &view, &paint_jobs, &screen_descriptor, None)
                        .unwrap();

                    self.context
                        .queue()
                        .submit(std::iter::once(encoder.finish()));
                    frame.present();

                    egui_rpass
                        .remove_textures(tdelta)
                        .expect("remove texture ok");
                }
                _ => {}
            }
        });
    }
}
