mod shader;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

pub mod parser;
pub mod utils;
pub use eframe::egui;

use eframe::egui::{Color32, Stroke, UserData, ViewportCommand};

use shader::Canvas;

pub use crate::utils::Shape;
pub mod shapes;
use crate::scene::Scene;

pub mod scene;
use image::{ImageBuffer, Rgba};

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
    screenshot_requested: bool,
    screenshot_result: Option<(Arc<egui::ColorImage>, egui::TextureHandle)>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, scene: Scene) -> Self {
        let gl = cc.gl.clone();
        let canvas = Canvas::new(gl.as_ref().unwrap().clone(), scene).unwrap();
        App {
            gl,
            canvas,
            ctx: cc.egui_ctx.clone(),
            screenshot_requested: false,
            screenshot_result: None,
        }
    }

    pub fn update_scene(&mut self, scene: Scene) {
        self.canvas.update_scene(scene);
    }

    pub fn take_screenshot(&mut self) {
        self.screenshot_requested = true;
    }

    pub fn poll_screenshot(&mut self) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
        if let Some((arc_image, _handle)) = self.screenshot_result.take() {
            let image = arc_image.as_ref();
            let width = image.size[0] as u32;
            let height = image.size[1] as u32;
            let raw_rgba = color_image_to_rgba_bytes(image);

            let buffer: ImageBuffer<Rgba<u8>, _> =
                ImageBuffer::from_raw(width, height, raw_rgba).expect("Invalid dimensions or data");

            Some(buffer)
        } else {
            None
        }
    }
}

fn color_image_to_rgba_bytes(image: &egui::ColorImage) -> Vec<u8> {
    image.pixels.iter().flat_map(|c| c.to_array()).collect()
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
                if self.screenshot_requested {
                    ui.ctx()
                        .send_viewport_cmd(ViewportCommand::Screenshot(UserData::default()));
                    self.screenshot_requested = false; // only request once
                }

                let image = ui.ctx().input(|i| {
                    i.events
                        .iter()
                        .filter_map(|e| {
                            if let egui::Event::Screenshot { image, .. } = e {
                                Some(image.clone())
                            } else {
                                None
                            }
                        })
                        .next_back()
                });

                if let Some(image) = image {
                    self.screenshot_result = Some((
                        image.clone(),
                        ui.ctx()
                            .load_texture("screenshot_demo", image, Default::default()),
                    ));
                }
            });
    }
}

pub struct NativeGuiViewer {
    pub app: Arc<Mutex<Option<App>>>,
}

impl NativeGuiViewer {
    pub fn render(scene: &Scene) -> Self {
        use std::{
            sync::{Arc, Mutex},
            thread,
        };

        #[cfg(not(target_arch = "wasm32"))]
        use eframe::{
            NativeOptions,
            egui::{Vec2, ViewportBuilder},
        };

        let viewport_size = scene.viewport.unwrap_or([800, 500]);

        let app: Arc<Mutex<Option<App>>> = Arc::new(Mutex::new(None));
        let app_clone = Arc::clone(&app);

        let scene = Arc::new(Mutex::new(scene.clone()));
        #[cfg(not(target_arch = "wasm32"))]
        thread::spawn(move || {
            use std::process;

            use eframe::{EventLoopBuilderHook, run_native};
            let event_loop_builder: Option<EventLoopBuilderHook> =
                Some(Box::new(|event_loop_builder| {
                    #[cfg(target_family = "windows")]
                    {
                        use egui_winit::winit::platform::windows::EventLoopBuilderExtWindows;
                        event_loop_builder.with_any_thread(true);
                    }
                    #[cfg(feature = "wayland")]
                    {
                        use egui_winit::winit::platform::wayland::EventLoopBuilderExtWayland;
                        event_loop_builder.with_any_thread(true);
                    }
                    #[cfg(feature = "x11")]
                    {
                        use egui_winit::winit::platform::x11::EventLoopBuilderExtX11;
                        event_loop_builder.with_any_thread(true);
                    }
                }));

            let native_options = NativeOptions {
                viewport: ViewportBuilder::default()
                    .with_inner_size(Vec2::new(viewport_size[0] as f32, viewport_size[1] as f32)),
                depth_buffer: 24,
                event_loop_builder,
                ..Default::default()
            };

            let _ = run_native(
                "cosmol_viewer",
                native_options,
                Box::new(move |cc| {
                    let mut guard = app_clone.lock().unwrap();
                    *guard = Some(App::new(cc, scene.lock().unwrap().clone()));
                    Ok(Box::new(AppWrapper(app_clone.clone())))
                }),
            );
            process::exit(0);
        });

        // 等待 App 初始化完成
        let timeout_ms = 3000;
        let mut waited = 0;

        loop {
            if app.lock().unwrap().is_some() {
                break;
            }
            if waited > timeout_ms {
                panic!("Timeout waiting for App to initialize");
            }
            thread::sleep(Duration::from_millis(10));
            waited += 10;
        }

        Self { app }
    }

    pub fn update(&self, scene: &Scene) {
        let mut app_guard = self.app.lock().unwrap();
        if let Some(app) = &mut *app_guard {
            app.update_scene(scene.clone());
            app.ctx.request_repaint();
        } else {
            panic!("App not initialized")
        }
    }

    pub fn take_screenshot(&self) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        loop {
            let mut app_guard = self.app.lock().unwrap();
            if let Some(app) = &mut *app_guard {
                println!("Taking screenshot");
                app.take_screenshot();
                app.ctx.request_repaint();
                break;
            }
            drop(app_guard);
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
        loop {
            let mut app_guard = self.app.lock().unwrap();
            if let Some(app) = &mut *app_guard {
                if let Some(image) = app.poll_screenshot() {
                    return image;
                }
            }
            drop(app_guard);
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}
