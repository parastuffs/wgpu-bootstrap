use crate::context::Context;

pub struct Computation<'a> {
    encoder: wgpu::CommandEncoder,
    queue: &'a wgpu::Queue,
}


impl<'a> Computation<'a> {
    pub fn new(context: &'a Context) -> Self {
        let encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Compute Encoder"),
            });

        Self {
            encoder,
            queue: &context.queue,
        }
    }

    pub fn begin_compute_pass(&mut self) -> wgpu::ComputePass {
        self.encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Compute Pass"),
        })
    }

    pub fn submit(self) {
        //let command_buffer = self.encoder.take().expect("You can't run a computation more than once").finish();
        self.queue.submit(std::iter::once(self.encoder.finish()));
    }
}
