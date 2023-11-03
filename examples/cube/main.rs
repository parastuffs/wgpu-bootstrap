mod cube_app;

use crate::cube_app::CubeApp;
use wgpu_bootstrap::runner::Runner;

fn main() {
    let mut runner = pollster::block_on(Runner::new());

    let app = CubeApp::new(&mut runner.context);

    runner.start(app);
}
