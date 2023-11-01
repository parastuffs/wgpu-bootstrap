mod triangle_app;

use crate::triangle_app::TriangleApp;
use wgpu_bootstrap::runner::Runner;

fn main() {
    let mut runner = pollster::block_on(Runner::new());

    let app = TriangleApp::new(&mut runner.context);

    runner.start(app);
}
