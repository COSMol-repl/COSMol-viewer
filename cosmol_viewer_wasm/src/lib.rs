use cosmol_viewer_core::{self, EguiRender, Scene};
#[cfg(target_arch = "wasm32")]
use eframe::WebRunner;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WebHandle {
    runner: WebRunner,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WebHandle {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        eframe::WebLogger::init(log::LevelFilter::Debug).ok();
        Self {
            runner: WebRunner::new(),
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

        self.runner
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|cc| Ok(Box::new(EguiRender::new(cc, scene)))),
            )
            .await
    }

    #[wasm_bindgen]
    pub fn destroy(&self) {
        self.runner.destroy();
    }
}
