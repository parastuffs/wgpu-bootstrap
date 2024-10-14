mod gui_app;

use crate::gui_app::GuiApp;
use wgpu_bootstrap::runner::Runner;

fn main() {
    let mut runner = Runner::new(Box::new(|context| Box::new(GuiApp::new(context))));
    pollster::block_on(runner.run());
}
