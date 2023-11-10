mod instances_app;

use crate::instances_app::ShadingApp;
use wgpu_bootstrap::runner::Runner;

fn main() {
    let mut runner = pollster::block_on(Runner::new());

    let app = ShadingApp::new(&mut runner.context);

    runner.start(app);
}
