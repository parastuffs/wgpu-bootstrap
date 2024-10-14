mod wireframe_app;

use crate::wireframe_app::WireframeApp;
use wgpu_bootstrap::runner::Runner;

fn main() {
    let mut runner = Runner::new(Box::new(|context| Box::new(WireframeApp::new(context))));
    pollster::block_on(runner.run());
}
