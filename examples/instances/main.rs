mod instances_app;

use crate::instances_app::InstanceApp;
use wgpu_bootstrap::runner::Runner;

fn main() {
    let mut runner = Runner::new(Box::new(|context| Box::new(InstanceApp::new(context))));
    pollster::block_on(runner.run());
}
