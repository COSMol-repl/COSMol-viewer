mod shader;
#[cfg(not(target_arch = "wasm32"))]
use egui_winit::winit::{self, event_loop::EventLoop};
use std::sync::{Arc, Mutex};

#[cfg(not(target_arch = "wasm32"))]
use eframe::native::winit_integration::WinitApp;

pub mod parser;
pub mod utils;
pub use eframe::egui;

#[cfg(not(target_arch = "wasm32"))]
use eframe::{AppCreator, Error, NativeOptions, Renderer, UserEvent, native::run::WinitAppWrapper};

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
            let native_options = NativeOptions {
                viewport: ViewportBuilder::default().with_inner_size(Vec2::new(800.0, 500.0)),
                depth_buffer: 24,
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

#[cfg(not(target_arch = "wasm32"))]
pub fn run_native(
    app_name: &str,
    mut native_options: NativeOptions,
    app_creator: AppCreator<'_>,
) -> Result {
    if native_options.viewport.title.is_none() {
        native_options.viewport.title = Some(app_name.to_owned());
    }

    let renderer = native_options.renderer;

    {
        match renderer {
            Renderer::Glow => "glow",
        };
    }

    match renderer {
        Renderer::Glow => run_glow(app_name, native_options, app_creator),
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub type Result<T = (), E = Error> = std::result::Result<T, E>;

#[cfg(not(target_arch = "wasm32"))]
pub fn run_glow(
    app_name: &str,
    mut native_options: NativeOptions,
    app_creator: AppCreator<'_>,
) -> Result {
    #![allow(clippy::needless_return_with_question_mark)] // False positive

    use eframe::native::glow_integration::GlowWinitApp;

    let event_loop = create_event_loop(&mut native_options)?;
    let glow_eframe = GlowWinitApp::new(&event_loop, app_name, native_options, app_creator);
    run_and_exit(event_loop, glow_eframe)
}

#[cfg(not(target_arch = "wasm32"))]
fn create_event_loop(native_options: &mut NativeOptions) -> Result<EventLoop<UserEvent>> {
    let mut builder = winit::event_loop::EventLoop::with_user_event();
    #[cfg(all(unix, not(target_vendor = "apple")))]
    {
        #[cfg(feature = "wayland")]
        {
            use winit::platform::wayland::EventLoopBuilderExtWayland;
            builder.with_any_thread(true);
        }
        #[cfg(feature = "x11")]
        {
            use winit::platform::x11::EventLoopBuilderExtX11;
            builder.with_any_thread(true);
        }
    }
    #[cfg(target_family = "windows")]
    {
        use winit::platform::windows::EventLoopBuilderExtWindows;
        builder.with_any_thread(true);
    }

    if let Some(hook) = std::mem::take(&mut native_options.event_loop_builder) {
        hook(&mut builder);
    }

    Ok(builder.build()?)
}

#[cfg(not(target_arch = "wasm32"))]
fn run_and_exit(event_loop: EventLoop<UserEvent>, winit_app: impl WinitApp) -> Result {
    let mut app = WinitAppWrapper::new(winit_app, false);
    event_loop.run_app(&mut app)?;

    Ok(())
}
