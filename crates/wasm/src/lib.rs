use cosmol_viewer_core::App;
use cosmol_viewer_core::AppWrapper;
use cosmol_viewer_core::scene::Scene;
use std::sync::Arc;
use std::sync::Mutex;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlCanvasElement;

#[cfg(target_arch = "wasm32")]
use eframe::WebRunner;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WebHandle {
    runner: WebRunner,
    scene: Arc<Mutex<Scene>>,
    app: Arc<Mutex<Option<App>>>,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WebHandle {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        eframe::WebLogger::init(log::LevelFilter::Debug).ok();
        Self {
            runner: WebRunner::new(),
            scene: Arc::new(Mutex::new(Scene::new())),
            app: Arc::new(Mutex::new(None)),
        }
    }

    #[wasm_bindgen]
    pub async fn start_with_scene(
        &mut self,
        canvas: HtmlCanvasElement,
        scene_json: String,
    ) -> Result<(), JsValue> {
        let scene: Scene = serde_json::from_str(&scene_json)
            .map_err(|e| JsValue::from_str(&format!("Scene parse error: {}", e)))?;

        // let scene = Arc::clone(&self.scene);
        let app = Arc::clone(&self.app);

        let _ = self
            .runner
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(move |cc| {
                    let mut guard = app.lock().unwrap();
                    *guard = Some(App::new(
                        cc,
                        // Scene::new(),
                        scene,
                    ));
                    Ok(Box::new(AppWrapper(app.clone())))
                }),
            )
            .await;
        // });

        Ok(())
    }

    #[wasm_bindgen]
    pub async fn update_scene(&mut self, scene_json: String) -> Result<(), JsValue> {
        let scene: Scene = serde_json::from_str(&scene_json)
            .map_err(|e| JsValue::from_str(&format!("Scene parse error: {}", e)))?;

        let mut app_guard = self.app.lock().unwrap();
        if let Some(app) = &mut *app_guard {
            println!("Received scene update");
            app.update_scene(scene);
            app.ctx.request_repaint();
        } else {
            println!("scene update received but app is not initialized");
        }

        Ok(())
    }
}
