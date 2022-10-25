use crate::{context::Context, texture::Texture};

pub struct Frame<'a> {
    output: wgpu::SurfaceTexture,
    view: wgpu::TextureView,
    encoder: wgpu::CommandEncoder,
    queue: &'a wgpu::Queue,
    depth_texture: &'a Texture,
}

impl<'a> Frame<'a> {
    pub fn new(context: &'a Context) -> Result<Self, wgpu::SurfaceError> {
        let output = context.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        Ok(Self {
            output,
            view,
            encoder,
            queue: &context.queue,
            depth_texture: &context.depth_texture,
        })
    }

    pub fn begin_render_pass(&mut self) -> wgpu::RenderPass {
        self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        })
    }

    pub fn present(self) {
        self.queue.submit(std::iter::once(self.encoder.finish()));
        self.output.present();
    }
}