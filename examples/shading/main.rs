mod shading_app;

use std::sync::Arc;

use crate::shading_app::ShadingApp;
use wgpu_bootstrap::{egui, Runner};

fn main() {
    let mut runner = Runner::new(
        "Shading App",
        800,
        600,
        egui::Color32::from_rgb(245, 245, 245),
        32,
        0,
        Box::new(|context| Arc::new(ShadingApp::new(context))),
    );
    runner.run();
}
