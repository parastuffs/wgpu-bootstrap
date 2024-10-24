mod instances_app;

use std::sync::Arc;

use crate::instances_app::InstanceApp;
use wgpu_bootstrap::Runner;

fn main() {
    let mut runner = Runner::new(
        "Gui App",
        800,
        600,
        32,
        0,
        Box::new(|context| Arc::new(InstanceApp::new(context))),
    );
    runner.run();
}
