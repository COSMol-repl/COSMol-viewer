mod shader;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub mod utils;

use eframe::egui::{self, Color32, Stroke};


use serde::{Deserialize, Serialize};
use shader::Canvas;

pub use crate::utils::{Shape, Sphere};
use crate::{shader::CameraState, utils::ToMesh};

pub struct AppWrapper(pub Arc<Mutex<Option<App>>>);

impl eframe::App for AppWrapper {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Some(app) = &mut *self.0.lock().unwrap() {
            app.update(ctx, frame);
        }
    }
}

pub struct App {
    canvas: Canvas,
    gl: Option<Arc<eframe::glow::Context>>,
    pub ctx: egui::Context,
}

impl App {
    // #[cfg(not(target_arch = "wasm32"))]
    pub fn new(cc: &eframe::CreationContext<'_>, scene: SharedScene) -> Self {
        let gl = cc.gl.clone();
        let canvas = Canvas::new(gl.as_ref().unwrap().clone(), scene).unwrap();
        App {
            gl,
            canvas,
            ctx: cc.egui_ctx.clone(),
        }
    }

    // #[cfg(target_arch = "wasm32")]
    // pub fn new(cc: &eframe::CreationContext<'_>, scene: Scene) -> Self {
    //     let gl = cc.gl.clone();
    //     let triangle = Canvas::new(gl.as_ref().unwrap().clone(), &scene).unwrap();
    //     App {
    //         gl: gl,
    //         canvas: triangle,
    //         scene: scene,
    //     }
    // }

    pub fn update_scene(&mut self, scene: &SceneData) {
        self.canvas.update_scene(scene);
    }
}

impl eframe::App for App {
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
pub struct SceneData {
    pub background_color: [f32; 3],
    pub camera_state: CameraState,
    pub named_shapes: HashMap<String, Shape>,
    pub unnamed_shapes: Vec<Shape>,
    // pub dirty: AtomicBool,
}

impl SceneData {
    pub fn get_meshes(&self) -> Vec<utils::MeshData> {
        self.named_shapes
            .values()
            .chain(self.unnamed_shapes.iter())
            .map(|s| s.to_mesh())
            .collect()
    }

    pub fn new() -> Self {
        SceneData {
            background_color: [1.0, 1.0, 1.0],
            camera_state: CameraState::new(1.0),
            named_shapes: HashMap::new(),
            unnamed_shapes: Vec::new(),
            // dirty: AtomicBool::new(false),
        }
    }
}

#[derive(Clone)]
pub struct Scene {
    pub inner: Arc<Mutex<SceneData>>,
}

pub type SharedScene = Arc<Mutex<SceneData>>;

impl Scene {
    pub fn create_viewer() -> Self {
        Scene {
            inner: Arc::new(Mutex::new(SceneData {
                background_color: [1.0, 1.0, 1.0],
                named_shapes: HashMap::new(),
                unnamed_shapes: Vec::new(),
                camera_state: CameraState::new(1.0),
                // dirty: AtomicBool::new(false),
            })),
        }
    }

    pub fn add_shapes(&mut self, spec: Sphere, id: Option<impl Into<String>>) {
        let mut guard = self.inner.lock().unwrap();
        if let Some(id) = id {
            guard.named_shapes.insert(id.into(), Shape::Sphere(spec));
        } else {
            guard.unnamed_shapes.push(Shape::Sphere(spec));
        }
        // guard.dirty.store(true, Ordering::SeqCst);
    }

    pub fn delete_sphere(&mut self, id: &str) {
        let mut guard = self.inner.lock().unwrap();
        if guard.named_shapes.remove(id).is_none() {
            panic!("Sphere with ID '{}' not found", id);
        }
        // guard.dirty.store(true, Ordering::SeqCst);
    }

    pub fn add_sphere(&mut self, spec: Sphere, id: Option<&str>) {
        self.add_shapes(spec, id);
    }

    pub fn update_sphere(&mut self, id: &str, update_fn: impl FnOnce(&mut Sphere)) {
        let mut guard = self.inner.lock().unwrap();
        if let Some(shape) = guard.named_shapes.get_mut(id) {
            if let Shape::Sphere(sphere) = shape {
                // print!("------{}", serde_json::to_string(&sphere).unwrap());
                update_fn(sphere);
                // print!("------{}", serde_json::to_string(&sphere).unwrap());
            }
        } else {
            panic!("Sphere with ID '{}' not found", id);
        }
    }
}


