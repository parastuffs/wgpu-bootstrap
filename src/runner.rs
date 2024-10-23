use eframe::{
    egui::{self, InputState},
    egui_wgpu::{self, CallbackResources, CallbackTrait},
    wgpu, CreationContext,
};
use std::time::Instant;

pub trait App {
    fn render(&self, _render_pass: &mut wgpu::RenderPass<'_>) {}

    fn render_gui(&mut self, _ctx: &egui::Context, _device: &wgpu::Device, _queue: &wgpu::Queue) {}

    fn update(&mut self, _delta_time: f32, _device: &wgpu::Device, _queue: &wgpu::Queue) {}

    fn input(&mut self, _input: InputState, _device: &wgpu::Device, _queue: &wgpu::Queue) {}

    fn resize(
        &mut self,
        _new_width: i32,
        _new_height: i32,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
    }
}

pub struct Runner {
    app_creator: Option<Box<dyn FnOnce(&CreationContext<'_>) -> Box<dyn App + Send + Sync>>>,
}

impl Runner {
    pub fn new(
        app_creator: Box<dyn FnOnce(&CreationContext<'_>) -> Box<dyn App + Send + Sync>>,
    ) -> Self {
        Self {
            app_creator: Some(app_creator),
        }
    }

    pub async fn run(&mut self) {
        let native_options = eframe::NativeOptions::default();
        let _ = eframe::run_native(
            "My egui App",
            native_options,
            Box::new(|cc| {
                Ok(Box::new(EframeApp::new(
                    cc,
                    self.app_creator.take().unwrap(),
                )))
            }),
        );
    }
}

struct EframeApp {
    window_width: i32,
    window_height: i32,
    last: Option<Instant>,
}

impl EframeApp {
    fn new(
        cc: &eframe::CreationContext<'_>,
        app_creator: Box<dyn FnOnce(&CreationContext<'_>) -> Box<dyn App + Send + Sync>>,
    ) -> Self {
        let wgpu_render_state = cc.wgpu_render_state.as_ref().unwrap();

        wgpu_render_state
            .renderer
            .write()
            .callback_resources
            .insert(app_creator(cc));

        Self {
            window_width: cc.egui_ctx.screen_rect().width() as i32,
            window_height: cc.egui_ctx.screen_rect().height() as i32,
            last: None,
        }
    }
}

impl eframe::App for EframeApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let wgpu_render_state = frame.wgpu_render_state().unwrap();
        let device = wgpu_render_state.device.clone();
        let queue = wgpu_render_state.queue.clone();
        let mut writer = wgpu_render_state.renderer.write();
        let app: &mut Box<dyn App> = writer.callback_resources.get_mut().unwrap();
        let now = Instant::now();
        let delta_time = match self.last {
            Some(last) => now.duration_since(last).as_secs_f32(),
            None => 0.0,
        };
        self.last = Some(now);
        if ctx.screen_rect().width() as i32 != self.window_width
            || ctx.screen_rect().height() as i32 != self.window_height
        {
            self.window_width = ctx.screen_rect().width() as i32;
            self.window_height = ctx.screen_rect().height() as i32;
            app.resize(
                self.window_width,
                self.window_height,
                device.as_ref(),
                queue.as_ref(),
            );
        }
        let input = ctx.input(|i| i.clone());

        egui::CentralPanel::default().show(ctx, |ui| {
            // egui::Frame::canvas(ui.style()).show(ui, |ui| {
            let response = ui.allocate_response(
                egui::vec2(ctx.screen_rect().width(), ctx.screen_rect().height()),
                egui::Sense::click_and_drag(),
            );

            app.input(input, device.as_ref(), queue.as_ref());

            ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                response.rect,
                WgpuCallback { delta_time },
            ));
            // });
        });

        app.render_gui(ctx, device.as_ref(), queue.as_ref());
    }
}

struct WgpuCallback {
    delta_time: f32,
}

impl CallbackTrait for WgpuCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let resources: &mut Box<dyn App> = resources.get_mut().unwrap();
        resources.update(self.delta_time, device, queue);
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        resources: &CallbackResources,
    ) {
        let resources: &Box<dyn App> = resources.get().unwrap();
        resources.render(render_pass);
    }
}
