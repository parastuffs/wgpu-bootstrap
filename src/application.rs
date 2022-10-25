
use winit::event::WindowEvent;
use crate::context::Context;

pub trait Application {
    #[allow(unused_variables)]
    fn render(&self, context: &Context) -> Result<(), wgpu::SurfaceError> { Ok(()) }
    
    #[allow(unused_variables)]
    fn update(&mut self, context: &Context) {}
    
    #[allow(unused_variables)]
    fn input(&mut self, event: &WindowEvent, context: &Context) -> bool { false }
}