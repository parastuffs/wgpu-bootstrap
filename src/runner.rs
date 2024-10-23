use crate::{context::Context, egui_layer::EguiLayer};
use std::time::Instant;
use wgpu::TextureView;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

pub trait App {
    #[allow(unused_variables)]
    fn render(&mut self, context: &mut Context, view: &TextureView) {}

    #[allow(unused_variables)]
    fn render_gui(&mut self, context: &mut Context, egui_context: &egui::Context) {}

    #[allow(unused_variables)]
    fn update(&mut self, context: &mut Context, delta_time: f32) {}

    #[allow(unused_variables)]
    fn window_event(&mut self, context: &mut Context, event: &WindowEvent) -> bool {
        return false;
    }

    #[allow(unused_variables)]
    fn device_event(&mut self, context: &mut Context, event: &DeviceEvent) -> bool {
        return false;
    }

    #[allow(unused_variables)]
    fn resize(&mut self, context: &mut Context, new_size: winit::dpi::PhysicalSize<u32>) {}
}

pub struct Runner {
    context: Option<Context>,
    egui_layer: Option<EguiLayer>,
    app_creator: Option<Box<dyn FnOnce(&mut Context) -> Box<dyn App>>>,
    app: Option<Box<dyn App>>,
    start_time: Instant,
    last: Option<Instant>,
}

impl Runner {
    pub fn new(app_creator: Box<dyn FnOnce(&mut Context) -> Box<dyn App>>) -> Self {
        Self {
            context: None,
            egui_layer: None,
            app_creator: Some(app_creator),
            app: None,
            start_time: Instant::now(),
            last: None,
        }
    }

    pub async fn run(&mut self) {
        let event_loop = EventLoop::new().unwrap();
        let _ = event_loop.run_app(self);
    }
}

impl ApplicationHandler for Runner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes().with_title("Game of life"))
            .unwrap();
        self.context = Some(Context::new(window));
        self.egui_layer = Some(EguiLayer::new(self.context.as_mut().unwrap()));
        let app_creator = self.app_creator.take();
        self.app = Some((app_creator.unwrap())(self.context.as_mut().unwrap()));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let context = self.context.as_mut().unwrap();
        let window = context.window();
        let app = self.app.as_mut().unwrap();
        let egui_layer = self.egui_layer.as_mut().unwrap();

        if window.id() == window_id {
            if !egui_layer.window_event(&event) {
                if !app.window_event(context, &event) {
                    match event {
                        WindowEvent::CloseRequested => {
                            event_loop.exit();
                        }
                        WindowEvent::Resized(physical_size) => {
                            context.resize(physical_size);
                            app.resize(context, physical_size);
                        }
                        WindowEvent::RedrawRequested => {
                            let now = Instant::now();
                            let delta_time = match self.last {
                                Some(last) => now.duration_since(last).as_secs_f32(),
                                None => 0.0,
                            };
                            self.last = Some(now);

                            egui_layer
                                .update_time(now.duration_since(self.start_time).as_secs_f64());

                            app.update(context, delta_time);

                            let output = context.surface().get_current_texture();

                            let frame = match output {
                                Ok(texture) => texture,
                                // Reconfigure the surface if lost
                                Err(wgpu::SurfaceError::Lost) => {
                                    let size = *context.size();
                                    context.resize(size);
                                    return;
                                }
                                // The system is out of memory, we should probably quit
                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    eprintln!("Out Of Memory");
                                    event_loop.exit();
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

                            app.render(context, &view);

                            egui_layer.render(context, &view, app);

                            frame.present();

                            // egui_layer
                            //     .render_pass()
                            //     .remove_textures(tdelta)
                            //     .expect("remove texture ok");

                            egui_layer.cleanup();
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        let context = self.context.as_mut().unwrap();
        let app = self.app.as_mut().unwrap();

        app.device_event(context, &event);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let window = self.context.as_ref().unwrap().window();
        window.request_redraw();
    }
}

// impl ApplicationHandler for Runner {
//     fn resumed(&mut self, event_loop: &ActiveEventLoop) {
//         self.window = Some(
//             event_loop
//                 .create_window(Window::default_attributes())
//                 .unwrap(),
//         );
//     }
//
//     fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
//         match event {
//             WindowEvent::CloseRequested => {
//                 println!("The close button was pressed; stopping");
//                 event_loop.exit();
//             }
//             WindowEvent::RedrawRequested => {
//                 // Redraw the application.
//                 //
//                 // It's preferable for applications that do not render continuously to render in
//                 // this event rather than in AboutToWait, since rendering in here allows
//                 // the program to gracefully handle redraws requested by the OS.
//
//                 // Draw.
//
//                 // Queue a RedrawRequested event.
//                 //
//                 // You only need to call this if you've determined that you need to redraw in
//                 // applications which do not always need to. Applications that redraw continuously
//                 // can render here instead.
//                 self.window.as_ref().unwrap().request_redraw();
//             }
//             _ => (),
//         }
//     }
// }

// pub async fn start_old(mut app: impl App + 'static) {
//     env_logger::init();
//     let event_loop = EventLoop::new().unwrap();
//     let window = WindowBuilder::new().build(&event_loop).unwrap();
//     let context = Context::new(&window).await;
//
//     let mut platform = Platform::new(PlatformDescriptor {
//         physical_width: context.size().width,
//         physical_height: context.size().height,
//         scale_factor: context.window().scale_factor(),
//         font_definitions: FontDefinitions::default(),
//         style: Default::default(),
//     });
//
//     // We use the egui_wgpu_backend crate as the render backend.
//     let mut egui_rpass = RenderPass::new(context.device(), context.config().format, 1);
//
//     let start_time = Instant::now();
//     let mut last: Option<Instant> = None;
//     event_loop.run(move |event, control_flow| {
//         control_flow.set_poll();
//
//         match event {
//             Event::WindowEvent {
//                 ref event,
//                 window_id,
//             } if window_id == window.id() => {
//                 match event {
//                     WindowEvent::CloseRequested => {
//                         println!("The close button was pressed; stopping");
//                         control_flow.exit();
//                     }
//                     WindowEvent::Resized(physical_size) => {
//                         // Hack for MacOS 14 Bug
//                         if physical_size.width < u32::MAX {
//                             context.resize(*physical_size);
//                             app.resize(&mut context, *physical_size);
//                         }
//                     }
//                     WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
//                         println!("Scale Factor Changed");
//                         context.resize(**new_inner_size);
//                         app.resize(&mut context, **new_inner_size);
//                     }
//                     _ => {}
//                 }
//             }
//             Event::AboutToWait => {
//                 let now = Instant::now();
//                 let delta_time = match last {
//                     Some(last) => now.duration_since(last).as_secs_f32(),
//                     None => 0.0,
//                 };
//                 last = Some(now);
//                 platform.update_time(now.duration_since(start_time).as_secs_f64());
//
//                 app.update(&mut context, delta_time);
//                 context.window().request_redraw();
//             }
//             Event::WindowEvent {
//                 event: WindowEvent::RedrawRequested,
//                 ..
//             } => {
//                 let output = context.surface().get_current_texture();
//
//                 let frame = match output {
//                     Ok(texture) => texture,
//                     // Reconfigure the surface if lost
//                     Err(wgpu::SurfaceError::Lost) => {
//                         context.resize(*context.size());
//                         return;
//                     }
//                     // The system is out of memory, we should probably quit
//                     Err(wgpu::SurfaceError::OutOfMemory) => {
//                         eprintln!("Out Of Memory");
//                         control_flow.exit();
//                         return;
//                     }
//                     // All other errors (Outdated, Timeout) should be resolved by the next frame
//                     Err(e) => {
//                         eprintln!("{:?}", e);
//                         return;
//                     }
//                 };
//
//                 let view = frame
//                     .texture
//                     .create_view(&wgpu::TextureViewDescriptor::default());
//
//                 app.render(&mut context, &view);
//
//                 // platform.update_time(start_time.elapsed().as_secs_f64());
//
//                 // Begin to draw the UI frame.
//                 platform.begin_frame();
//
//                 app.render_gui(&mut context, &platform.context());
//
//                 let full_output = platform.end_frame(Some(context.window()));
//                 let paint_jobs = platform
//                     .context()
//                     .tessellate(full_output.shapes, full_output.pixels_per_point);
//
//                 let mut encoder =
//                     context
//                         .device()
//                         .create_command_encoder(&wgpu::CommandEncoderDescriptor {
//                             label: Some("egui encoder"),
//                         });
//
//                 // Upload all resources for the GPU.
//                 let screen_descriptor = ScreenDescriptor {
//                     physical_width: context.size().width,
//                     physical_height: context.size().height,
//                     scale_factor: context.window().scale_factor() as f32,
//                 };
//                 let tdelta: egui::TexturesDelta = full_output.textures_delta;
//                 egui_rpass
//                     .add_textures(context.device(), context.queue(), &tdelta)
//                     .expect("add texture ok");
//                 egui_rpass.update_buffers(
//                     context.device(),
//                     context.queue(),
//                     &paint_jobs,
//                     &screen_descriptor,
//                 );
//
//                 egui_rpass
//                     .execute(&mut encoder, &view, &paint_jobs, &screen_descriptor, None)
//                     .unwrap();
//
//                 context.queue().submit(std::iter::once(encoder.finish()));
//                 frame.present();
//
//                 egui_rpass
//                     .remove_textures(tdelta)
//                     .expect("remove texture ok");
//             }
//             _ => {}
//         }
//
//         platform.handle_event(&event);
//
//         if !platform.captures_event(&event) {
//             app.input(&mut context, &event);
//         }
//     });
// }

// pub struct Runner<'a> {
//     event_loop: EventLoop<()>,
//     pub context: Context<'a>,
// }
//
// impl<'a> Runner<'a> {
//     pub async fn new() -> Self {
//         env_logger::init();
//         let event_loop = EventLoop::new().unwrap();
//         let window = WindowBuilder::new().build(&event_loop).unwrap();
//         let context = Context::new(window).await;
//
//         Self {
//             event_loop,
//             context,
//         }
//     }
//
//     pub fn start(mut self, mut app: impl App + 'static) {
//         let mut platform = Platform::new(PlatformDescriptor {
//             physical_width: self.context.size().width,
//             physical_height: self.context.size().height,
//             scale_factor: self.context.window().scale_factor(),
//             font_definitions: FontDefinitions::default(),
//             style: Default::default(),
//         });
//
//         // We use the egui_wgpu_backend crate as the render backend.
//         let mut egui_rpass =
//             RenderPass::new(self.context.device(), self.context.config().format, 1);
//
//         let start_time = Instant::now();
//         let mut last: Option<Instant> = None;
//         self.event_loop.run(move |event, _, control_flow| {
//             control_flow.set_poll();
//
//             match event {
//                 Event::WindowEvent {
//                     ref event,
//                     window_id,
//                 } if window_id == self.context.window().id() => {
//                     match event {
//                         WindowEvent::CloseRequested => {
//                             println!("The close button was pressed; stopping");
//                             control_flow.set_exit();
//                         }
//                         WindowEvent::Resized(physical_size) => {
//                             // Hack for MacOS 14 Bug
//                             if physical_size.width < u32::MAX {
//                                 self.context.resize(*physical_size);
//                                 app.resize(&mut self.context, *physical_size);
//                             }
//                         }
//                         WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
//                             println!("Scale Factor Changed");
//                             self.context.resize(**new_inner_size);
//                             app.resize(&mut self.context, **new_inner_size);
//                         }
//                         _ => {}
//                     }
//                 }
//                 Event::MainEventsCleared => {
//                     let now = Instant::now();
//                     let delta_time = match last {
//                         Some(last) => now.duration_since(last).as_secs_f32(),
//                         None => 0.0,
//                     };
//                     last = Some(now);
//                     platform.update_time(now.duration_since(start_time).as_secs_f64());
//
//                     app.update(&mut self.context, delta_time);
//                     self.context.window().request_redraw();
//                 }
//                 Event::RedrawRequested(_) => {
//                     let output = self.context.surface().get_current_texture();
//
//                     let frame = match output {
//                         Ok(texture) => texture,
//                         // Reconfigure the surface if lost
//                         Err(wgpu::SurfaceError::Lost) => {
//                             self.context.resize(*self.context.size());
//                             return;
//                         }
//                         // The system is out of memory, we should probably quit
//                         Err(wgpu::SurfaceError::OutOfMemory) => {
//                             eprintln!("Out Of Memory");
//                             control_flow.set_exit();
//                             return;
//                         }
//                         // All other errors (Outdated, Timeout) should be resolved by the next frame
//                         Err(e) => {
//                             eprintln!("{:?}", e);
//                             return;
//                         }
//                     };
//
//                     let view = frame
//                         .texture
//                         .create_view(&wgpu::TextureViewDescriptor::default());
//
//                     app.render(&mut self.context, &view);
//
//                     // platform.update_time(start_time.elapsed().as_secs_f64());
//
//                     // Begin to draw the UI frame.
//                     platform.begin_frame();
//
//                     app.render_gui(&mut self.context, &platform.context());
//
//                     let full_output = platform.end_frame(Some(self.context.window()));
//                     let paint_jobs = platform.context().tessellate(full_output.shapes);
//
//                     let mut encoder = self.context.device().create_command_encoder(
//                         &wgpu::CommandEncoderDescriptor {
//                             label: Some("egui encoder"),
//                         },
//                     );
//
//                     // Upload all resources for the GPU.
//                     let screen_descriptor = ScreenDescriptor {
//                         physical_width: self.context.size().width,
//                         physical_height: self.context.size().height,
//                         scale_factor: self.context.window().scale_factor() as f32,
//                     };
//                     let tdelta: egui::TexturesDelta = full_output.textures_delta;
//                     egui_rpass
//                         .add_textures(self.context.device(), self.context.queue(), &tdelta)
//                         .expect("add texture ok");
//                     egui_rpass.update_buffers(
//                         self.context.device(),
//                         self.context.queue(),
//                         &paint_jobs,
//                         &screen_descriptor,
//                     );
//
//                     egui_rpass
//                         .execute(&mut encoder, &view, &paint_jobs, &screen_descriptor, None)
//                         .unwrap();
//
//                     self.context
//                         .queue()
//                         .submit(std::iter::once(encoder.finish()));
//                     frame.present();
//
//                     egui_rpass
//                         .remove_textures(tdelta)
//                         .expect("remove texture ok");
//                 }
//                 _ => {}
//             }
//
//             platform.handle_event(&event);
//
//             if !platform.captures_event(&event) {
//                 app.input(&mut self.context, &event);
//             }
//         });
//     }
// }
