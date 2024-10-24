mod gui_app;

use std::sync::Arc;

use crate::gui_app::GuiApp;
use wgpu_bootstrap::Runner;

fn main() {
    let mut runner = Runner::new(
        "Gui App",
        800,
        600,
        32,
        0,
        Box::new(|context| Arc::new(GuiApp::new(context))),
    );
    runner.run();
}
