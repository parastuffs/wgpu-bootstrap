
use winit::event::WindowEvent;
use crate::context::Context;

pub trait Application {
    fn render(&self, context: &Context) -> Result<(), wgpu::SurfaceError> { Ok(()) }
    fn update(&mut self, context: &Context) {}
    fn input(&mut self, event: &WindowEvent, context: &Context) -> bool { false }
}