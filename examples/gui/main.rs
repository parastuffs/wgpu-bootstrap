mod gui_app;

use crate::gui_app::GuiApp;
use wgpu_bootstrap::runner::Runner;

fn main() {
    let mut runner = pollster::block_on(Runner::new());

    let app = GuiApp::new(&mut runner.context);

    runner.start(app);
}
