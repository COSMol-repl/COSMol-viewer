mod triangle;
use std::sync::Arc;

use eframe::egui::{self, Color32, Stroke};
use triangle::Triangle;

pub struct CosViewerCore{
    state: Option<String>,
}

impl CosViewerCore {
    pub fn new() -> Self {
        CosViewerCore {
            state: None,
        }
    }

    pub fn add(a: i32, b: i32) -> i32 {
        a + b
    }

    pub fn set_state(&mut self, state: String) {
        self.state = Some(state);
    }

    pub fn get_state(&self) -> Option<&String> {
        self.state.as_ref()
    }
}

pub struct MyEguiApp {
    triangle: Triangle,
    gl: Option<Arc<eframe::glow::Context>>,
}

impl MyEguiApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let gl = cc.gl.clone();
        let triangle = Triangle::new(gl.as_ref().unwrap().clone()).unwrap();
        MyEguiApp {
            gl: gl,
            triangle: triangle,
        }
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);
        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .fill(Color32::from_rgb(48, 48, 48))
                    .inner_margin(0.0)
                    .outer_margin(0.0)
                    .stroke(Stroke::new(0.0, Color32::from_rgb(30, 200, 30))),
            )
            .show(ctx, |ui| {
                ui.set_width(ui.available_width());
                ui.set_height(ui.available_height());

                self.triangle.custom_painting(ui);
            });
    }
}