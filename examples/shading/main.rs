mod shading_app;

use std::sync::Arc;

use crate::shading_app::ShadingApp;
use wgpu_bootstrap::Runner;

fn main() {
    let mut runner = Runner::new(
        "Shading App",
        800,
        600,
        32,
        0,
        Box::new(|context| Arc::new(ShadingApp::new(context))),
    );
    runner.run();
}
