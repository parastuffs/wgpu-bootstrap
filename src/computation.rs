use crate::context::Context;

pub struct Computation<'a> {
    encoder: Option<wgpu::CommandEncoder>,
    queue: &'a wgpu::Queue,
    device: &'a wgpu::Device,
}


impl<'a> Computation<'a> {
    pub fn new(context: &'a Context) -> Self {
        let encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Compute Encoder"),
            });

        Self {
            encoder: Some(encoder),
            queue: &context.queue,
            device: &context.device,
        }
    }

    pub fn begin_compute_pass(&mut self) -> wgpu::ComputePass {
        let encoder = self.encoder.as_mut().expect("You can't begin a compute pass after it was run");
        encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Compute Pass"),
        })
    }

    pub fn run(&mut self) {
        let command_buffer = self.encoder.take().expect("You can't run a computation more than once").finish();
        self.queue.submit(std::iter::once(command_buffer));
    }

    pub fn wait(&self) {
       self.device.poll(wgpu::Maintain::Wait);
    }
}
