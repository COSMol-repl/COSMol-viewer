use cosviewer_core::{self, MyEguiApp};
use eframe::egui::{Vec2, ViewportBuilder};

fn main() {
    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size(Vec2::new(800.0, 600.0)),
        depth_buffer: 24,
        ..Default::default()
    };
    let _ = eframe::run_native(
        "My egui App",
        native_options,
        Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))),
    );
}


