mod triangle_app;

use std::sync::Arc;

use crate::triangle_app::TriangleApp;
use wgpu_bootstrap::{egui, Runner};

fn main() {
    let mut runner = Runner::new(
        "Triangle App",
        800,
        600,
        egui::Color32::from_rgb(245, 245, 245),
        0,
        0,
        Box::new(|context| Arc::new(TriangleApp::new(context))),
    );
    runner.run();
}
