mod shader;
use std::sync::{Arc, Mutex};

pub mod parser;
pub mod utils;
pub use eframe::egui;

use eframe::egui::{Color32, Stroke};

use shader::Canvas;

pub use crate::utils::Shape;
pub mod shapes;
use crate::scene::Scene;

pub mod scene;

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
    pub fn new(cc: &eframe::CreationContext<'_>, scene: Scene) -> Self {
        let gl = cc.gl.clone();
        let canvas = Canvas::new(gl.as_ref().unwrap().clone(), scene).unwrap();
        App {
            gl,
            canvas,
            ctx: cc.egui_ctx.clone(),
        }
    }

    pub fn update_scene(&mut self, scene: Scene) {
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

#[cfg(not(target_arch = "wasm32"))]
pub struct NativeGuiViewer {
    pub app: Arc<Mutex<Option<App>>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl NativeGuiViewer {
    pub fn render(scene: &Scene) -> Self {
        use std::{
            sync::{Arc, Mutex},
            thread,
        };

        use eframe::{
            NativeOptions,
            egui::{Vec2, ViewportBuilder},
        };

        let app: Arc<Mutex<Option<App>>> = Arc::new(Mutex::new(None));
        let app_clone = Arc::clone(&app);

        let scene = Arc::new(Mutex::new(scene.clone()));

        thread::spawn(move || {
            use std::process;

            use eframe::{EventLoopBuilderHook, run_native};
            let event_loop_builder: Option<EventLoopBuilderHook> =
                Some(Box::new(|event_loop_builder| {
                    #[cfg(target_family = "windows")]
                    {
                        use egui_winit::winit::platform::windows::EventLoopBuilderExtWindows;
                        event_loop_builder.with_any_thread(true);
                        println!("Running on windows")
                    }
                    #[cfg(feature = "wayland")]
                    {
                        use egui_winit::winit::platform::wayland::EventLoopBuilderExtWayland;
                        event_loop_builder.with_any_thread(true);
                        println!("Running on wayland")
                    }
                    #[cfg(feature = "x11")]
                    {
                        use egui_winit::winit::platform::x11::EventLoopBuilderExtX11;
                        event_loop_builder.with_any_thread(true);
                        println!("Running on X11")
                    }
                }));

            let native_options = NativeOptions {
                viewport: ViewportBuilder::default().with_inner_size(Vec2::new(800.0, 500.0)),
                depth_buffer: 24,
                event_loop_builder,
                ..Default::default()
            };

            let _ = run_native(
                "cosmol_viewer",
                native_options,
                Box::new(move |cc| {
                    let mut guard = app.lock().unwrap();
                    *guard = Some(App::new(cc, scene.lock().unwrap().clone()));
                    Ok(Box::new(AppWrapper(app.clone())))
                }),
            );
            process::exit(0);
        });

        Self { app: app_clone }
    }

    pub fn update(&self, scene: &Scene) {
        let mut app_guard = self.app.lock().unwrap();
        if let Some(app) = &mut *app_guard {
            // println!("Received scene update");
            app.update_scene(scene.clone());
            app.ctx.request_repaint();
        }
    }
}
