mod shader;
use std::sync::Arc;

pub mod utils;

use eframe::{egui::{self, Color32, Stroke, Vec2, ViewportBuilder}};

#[cfg(not(target_arch = "wasm32"))]
use eframe::{NativeOptions};

use serde::{Deserialize, Serialize};
use shader::Canvas;

use crate::{shader::{Camera, CameraState}, utils::ToMesh};
pub use crate::utils::{Shape, Sphere};

pub struct EguiRender {
    canvas: Canvas,
    gl: Option<Arc<eframe::glow::Context>>,
    #[cfg(target_arch = "wasm32")]
    scene: Scene,
}

impl EguiRender {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(cc: &eframe::CreationContext<'_>, scene: &Scene) -> Self {
        let gl = cc.gl.clone();
        let triangle = Canvas::new(gl.as_ref().unwrap().clone(), scene).unwrap();
        EguiRender {
            gl: gl,
            canvas: triangle,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new(cc: &eframe::CreationContext<'_>, scene: Scene) -> Self {
        let gl = cc.gl.clone();
        let triangle = Canvas::new(gl.as_ref().unwrap().clone(), &scene).unwrap();
        EguiRender {
            gl: gl,
            canvas: triangle,
            scene: scene,
        }
    }
}

impl eframe::App for EguiRender {
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

                self.canvas.custom_painting(ui);
            });
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Scene {
    background_color: [f32; 3],
    camera_state: CameraState,
    shapes: Vec<Shape>,
}

impl Scene {
    pub fn create_viewer() -> Self {
        Scene {
            background_color: [1.0, 1.0, 1.0],
            shapes: Vec::new(),
            camera_state: CameraState::new(1.0),
        }
    }

    pub fn add_spheres(&mut self, spec: Sphere) {
        self.shapes.push(Shape::Sphere(spec));
    }
    
    pub fn get_meshes(&self) -> Vec<utils::MeshData> {
        self.shapes.iter().map(|s| s.to_mesh()).collect()
    }
}

pub struct cosmol_viewer;

impl cosmol_viewer {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn render(scene: &Scene) {
        let native_options = NativeOptions {
            viewport: ViewportBuilder::default().with_inner_size(Vec2::new(400.0, 250.0)),
            depth_buffer: 24,
            ..Default::default()
        };

        let _ = eframe::run_native(
            "cosmol_viewer",
            native_options,
            Box::new(|cc| Ok(Box::new(EguiRender::new(cc, scene)))),
        );
    }

    pub fn update(scene: &Scene) {
        unimplemented!("cosmol_viewer::update is not implemented for this target");
    }
}
