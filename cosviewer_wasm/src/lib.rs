use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use eframe::WebRunner;
use web_sys::HtmlCanvasElement;
use cosviewer_core::CosViewerCore;
use cosviewer_core::{self, MyEguiApp};

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
    pub async fn start(&self, canvas: HtmlCanvasElement) -> Result<(), JsValue> {
        self.runner
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))),
            )
            .await
    }

    #[wasm_bindgen]
    pub fn destroy(&self) {
        self.runner.destroy();
    }
}
