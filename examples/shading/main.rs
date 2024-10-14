mod shading_app;

use crate::shading_app::ShadingApp;
use wgpu_bootstrap::runner::Runner;

fn main() {
    let mut runner = Runner::new(Box::new(|context| Box::new(ShadingApp::new(context))));
    pollster::block_on(runner.run());
}
