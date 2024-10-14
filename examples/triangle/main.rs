mod triangle_app;

use crate::triangle_app::TriangleApp;
use wgpu_bootstrap::runner::Runner;

fn main() {
    let mut runner = Runner::new(Box::new(|context| Box::new(TriangleApp::new(context))));
    pollster::block_on(runner.run());
}
