mod wireframe_app;

use crate::wireframe_app::WireframeApp;
use wgpu_bootstrap::runner::Runner;

fn main() {
    let mut runner = pollster::block_on(Runner::new());

    let app = WireframeApp::new(&mut runner.context);

    runner.start(app);
}
