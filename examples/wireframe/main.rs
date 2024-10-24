mod wireframe_app;

use std::sync::Arc;

use crate::wireframe_app::WireframeApp;
use wgpu_bootstrap::{egui, Runner};

fn main() {
    let mut runner = Runner::new(
        "Wireframe App",
        800,
        600,
        egui::Color32::from_rgb(245, 245, 245),
        32,
        0,
        Box::new(|context| Arc::new(WireframeApp::new(context))),
    );
    runner.run();
}
