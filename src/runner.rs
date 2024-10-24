use eframe::{
    egui::{self, InputState},
    egui_wgpu::{self, depth_format_from_bits, CallbackResources, CallbackTrait},
    wgpu,
};
use std::{sync::Arc, time::Instant};

#[allow(dead_code)]
pub struct Context<'a> {
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    size: egui::Vec2,
    format: wgpu::TextureFormat,
    depth_stencil_format: Option<wgpu::TextureFormat>,
}

impl<'a> Context<'a> {
    pub fn device(&self) -> &wgpu::Device {
        self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        self.queue
    }

    pub fn size(&self) -> egui::Vec2 {
        self.size
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    pub fn depth_stencil_format(&self) -> wgpu::TextureFormat {
        self.depth_stencil_format.unwrap()
    }
}

pub trait App {
    fn render(&self, _render_pass: &mut wgpu::RenderPass<'_>) {}

    fn render_gui(&mut self, _egui_ctx: &egui::Context, _context: &Context) {}

    fn update(&mut self, _delta_time: f32, _context: &Context) {}

    fn input(&mut self, _input: InputState, _context: &Context) {}

    fn resize(&mut self, _new_width: u32, _new_height: u32, _context: &Context) {}
}

pub struct Runner {
    app_name: String,
    width: u32,
    height: u32,
    depth_buffer: u8,
    stencil_buffer: u8,
    app_creator: Option<Box<dyn FnOnce(&Context) -> Arc<dyn App + Send + Sync>>>,
}

impl Runner {
    pub fn new(
        app_name: &str,
        width: u32,
        height: u32,
        depth_buffer: u8,
        stencil_buffer: u8,
        app_creator: Box<dyn FnOnce(&Context) -> Arc<dyn App + Send + Sync>>,
    ) -> Self {
        env_logger::init();

        Self {
            app_name: String::from(app_name),
            width,
            height,
            app_creator: Some(app_creator),
            depth_buffer,
            stencil_buffer,
        }
    }

    pub fn run(&mut self) {
        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size((self.width as f32, self.height as f32)),
            depth_buffer: self.depth_buffer,
            stencil_buffer: self.stencil_buffer,
            ..Default::default()
        };

        let depth_stencil_format = depth_format_from_bits(self.depth_buffer, self.stencil_buffer);

        let _ = eframe::run_native(
            &self.app_name,
            native_options,
            Box::new(|cc| {
                Ok(Box::new(EframeApp::new(
                    cc,
                    self.width,
                    self.height,
                    depth_stencil_format,
                    self.app_creator.take().unwrap(),
                )))
            }),
        );
    }
}

struct EframeApp {
    window_width: u32,
    window_height: u32,
    depth_stencil_format: Option<wgpu::TextureFormat>,
    last: Option<Instant>,
    app: Arc<dyn App + Send + Sync>,
}

impl EframeApp {
    fn new(
        cc: &eframe::CreationContext<'_>,
        width: u32,
        height: u32,
        depth_stencil_format: Option<wgpu::TextureFormat>,
        app_creator: Box<dyn FnOnce(&Context) -> Arc<dyn App + Send + Sync>>,
    ) -> Self {
        let wgpu_render_state = cc.wgpu_render_state.as_ref().unwrap();
        let device = wgpu_render_state.device.as_ref();
        let queue = wgpu_render_state.queue.as_ref();
        let format = wgpu_render_state.target_format;

        let context = Context {
            device,
            queue,
            size: egui::vec2(width as f32, height as f32),
            format,
            depth_stencil_format,
        };

        // wgpu_render_state
        //     .renderer
        //     .write()
        //     .callback_resources
        //     .insert(app_creator(&context));

        Self {
            window_width: width,
            window_height: height,
            depth_stencil_format,
            last: None,
            app: app_creator(&context),
        }
    }
}

impl eframe::App for EframeApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let now = Instant::now();
        let delta_time = match self.last {
            Some(last) => now.duration_since(last).as_secs_f32(),
            None => 0.0,
        };
        self.last = Some(now);

        let wgpu_render_state = frame.wgpu_render_state().unwrap();
        let device = wgpu_render_state.device.clone();
        let queue = wgpu_render_state.queue.clone();
        let format = wgpu_render_state.target_format;
        // let app: &mut Box<dyn App> = writer.callback_resources.get_mut().unwrap();
        let mut context = Context {
            device: device.as_ref(),
            queue: queue.as_ref(),
            size: egui::vec2(self.window_width as f32, self.window_height as f32),
            format,
            depth_stencil_format: self.depth_stencil_format,
        };

        if ctx.screen_rect().width() as u32 != self.window_width
            || ctx.screen_rect().height() as u32 != self.window_height
        {
            self.window_width = ctx.screen_rect().width() as u32;
            self.window_height = ctx.screen_rect().height() as u32;
            context.size = egui::vec2(self.window_width as f32, self.window_height as f32);
            Arc::get_mut(&mut self.app).unwrap().resize(
                self.window_width,
                self.window_height,
                &context,
            );
        }
        let input = ctx.input(|i| i.clone());

        if !ctx.wants_pointer_input() && !ctx.wants_keyboard_input() {
            Arc::get_mut(&mut self.app).unwrap().input(input, &context);
        }

        Arc::get_mut(&mut self.app)
            .unwrap()
            .update(delta_time, &context);

        Arc::get_mut(&mut self.app)
            .unwrap()
            .render_gui(ctx, &context);

        egui::CentralPanel::default().show(ctx, |ui| {
            // egui::Frame::canvas(ui.style()).show(ui, |ui| {
            let response = ui.allocate_space(egui::vec2(
                ctx.screen_rect().width(),
                ctx.screen_rect().height(),
            ));

            ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                response.1,
                WgpuCallback {
                    app: self.app.clone(),
                },
            ));
            // });
        });
        ctx.request_repaint();
    }
}

struct WgpuCallback {
    app: Arc<dyn App + Send + Sync>,
}

impl CallbackTrait for WgpuCallback {
    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        _resources: &CallbackResources,
    ) {
        // let app: &Box<dyn App> = resources.get().unwrap();
        self.app.render(render_pass);
    }
}
