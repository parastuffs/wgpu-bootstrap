use crate::{context::Context, runner::App};
use egui::FontDefinitions;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use wgpu::TextureView;
use winit::event::WindowEvent;

pub struct EguiLayer {
    platform: Platform,
    render_pass: RenderPass,
}

impl EguiLayer {
    pub fn new(context: &mut Context) -> Self {
        let platform = Platform::new(PlatformDescriptor {
            physical_width: context.size().width,
            physical_height: context.size().height,
            scale_factor: context.window().scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });

        let render_pass = RenderPass::new(context.device(), context.config().format, 1);

        Self {
            platform,
            render_pass,
        }
    }

    pub fn update_time(&mut self, elapsed_seconds: f64) {
        self.platform.update_time(elapsed_seconds);
    }

    pub fn platform(&mut self) -> &mut Platform {
        &mut self.platform
    }

    pub fn render_pass(&mut self) -> &mut RenderPass {
        &mut self.render_pass
    }

    pub fn window_event(&mut self, event: &WindowEvent) -> bool {
        self.platform.handle_event(&event);
        if self.platform.captures_event(&event) {
            true
        } else {
            false
        }
    }

    pub fn render(&mut self, context: &mut Context, view: &TextureView, app: &mut Box<dyn App>) {
        let platform = self.platform();

        // Begin to draw the UI frame.
        platform.begin_frame();

        app.render_gui(context, &platform.context());

        let full_output = platform.end_frame(Some(context.window()));
        let paint_jobs = platform
            .context()
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        let mut encoder =
            context
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("egui encoder"),
                });

        // Upload all resources for the GPU.
        let screen_descriptor = ScreenDescriptor {
            physical_width: context.size().width,
            physical_height: context.size().height,
            scale_factor: context.window().scale_factor() as f32,
        };

        let tdelta: egui::TexturesDelta = full_output.textures_delta;

        self.render_pass()
            .add_textures(context.device(), context.queue(), &tdelta)
            .expect("add texture ok");
        self.render_pass().update_buffers(
            context.device(),
            context.queue(),
            &paint_jobs,
            &screen_descriptor,
        );

        self.render_pass()
            .execute(&mut encoder, &view, &paint_jobs, &screen_descriptor, None)
            .unwrap();

        context.queue().submit(std::iter::once(encoder.finish()));
    }
}
