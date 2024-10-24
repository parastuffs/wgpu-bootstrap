mod wireframe_app;

use std::sync::Arc;

use crate::wireframe_app::WireframeApp;
use wgpu_bootstrap::Runner;

fn main() {
    let mut runner = Runner::new(
        "Wireframe App",
        800,
        600,
        32,
        0,
        Box::new(|context| Arc::new(WireframeApp::new(context))),
    );
    runner.run();
}
