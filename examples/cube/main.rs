mod cube_app;

use crate::cube_app::CubeApp;
use wgpu_bootstrap::runner::Runner;

fn main() {
    let mut runner = Runner::new(Box::new(|context| Box::new(CubeApp::new(context))));
    pollster::block_on(runner.run());
}
