use cosmol_viewer_core::App;
use cosmol_viewer_core::scene::Scene;
#[cfg(feature = "js_bridge")]
use serde::Serialize;
use std::sync::Arc;
use std::sync::Mutex;

#[cfg(feature = "wasm")]
use web_sys::HtmlCanvasElement;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;
#[cfg(feature = "wasm")]
use wasm_bindgen::JsValue;

#[cfg(feature = "js_bridge")]
use pyo3::Python;
#[cfg(feature = "js_bridge")]
pub fn setup_wasm_if_needed(py: Python) {
    use base64::Engine;
    use pyo3::types::PyAnyMethods;

    const JS_CODE: &str = include_str!("../../wasm/pkg/cosmol_viewer_wasm.js");

    let js_base64 = base64::engine::general_purpose::STANDARD.encode(JS_CODE);

    let combined_js = format!(
        r#"
(function() {{
    if (!window.cosmol_viewer_blob_url) {{
        const jsCode = atob("{js_base64}");
        const blob = new Blob([jsCode], {{ type: 'application/javascript' }});
        window.cosmol_viewer_blob_url = URL.createObjectURL(blob);
    }}
}})();
    "#
    );

    let ipython = py.import("IPython.display").unwrap();
    let display = ipython.getattr("display").unwrap();

    let js = ipython
        .getattr("Javascript")
        .unwrap()
        .call1((combined_js,))
        .unwrap();
    display.call1((js,)).unwrap();
}

#[cfg(feature = "js_bridge")]
pub struct WasmViewer {
    pub id: String,
}
#[cfg(feature = "js_bridge")]
impl WasmViewer {
    pub fn initate_viewer(py: Python, scene: &Scene) -> Self {
        use base64::Engine;
        use pyo3::types::PyAnyMethods;
        use uuid::Uuid;

        let unique_id = format!("cosmol_viewer_{}", Uuid::new_v4());
        const WASM_BYTES: &[u8] = include_bytes!("../../wasm/pkg/cosmol_viewer_wasm_bg.wasm");
        let wasm_base64 = base64::engine::general_purpose::STANDARD.encode(WASM_BYTES);

        let viewport_size = scene.viewport.unwrap_or([800, 500]);

        let html_code = format!(
            r#"
<canvas id="{id}" width="{width}" height="{height}" style="width:{width}px; height:{height}px;"></canvas>
            "#,
            id = unique_id,
            width = viewport_size[0],
            height = viewport_size[1]
        );

        let scene_json = serde_json::to_string(scene).unwrap();
        let escaped = serde_json::to_string(&scene_json).unwrap();

        let combined_js = format!(
            r#"
(function() {{
    const wasmBase64 = "{wasm_base64}";
    import(window.cosmol_viewer_blob_url).then(async (mod) => {{
        const wasmBytes = Uint8Array.from(atob(wasmBase64), c => c.charCodeAt(0));
        await mod.default(wasmBytes);

        const canvas = document.getElementById('{id}');
        const app = new mod.WebHandle();
        const sceneJson = {SCENE_JSON};
        console.log("Starting cosmol_viewer with scene:", sceneJson);
        await app.start_with_scene(canvas, sceneJson);

        window.cosmol_viewer_instances = window.cosmol_viewer_instances || {{}};
        window.cosmol_viewer_instances["{id}"] = app;
    }});
}})();
            "#,
            wasm_base64 = wasm_base64,
            id = unique_id,
            SCENE_JSON = escaped
        );
        let ipython = py.import("IPython.display").unwrap();
        let display = ipython.getattr("display").unwrap();

        let html = ipython
            .getattr("HTML")
            .unwrap()
            .call1((html_code,))
            .unwrap();
        display.call1((html,)).unwrap();

        let js = ipython
            .getattr("Javascript")
            .unwrap()
            .call1((combined_js,))
            .unwrap();
        display.call1((js,)).unwrap();

        Self { id: unique_id }
    }

    pub fn call<T: Serialize>(&self, py: Python, name: &str, input: T) -> () {
        use pyo3::types::PyAnyMethods;

        let input_json = serde_json::to_string(&input).unwrap();
        let escaped = serde_json::to_string(&input_json).unwrap();
        let combined_js = format!(
            r#"
(async function() {{
    console.log(window.cosmol_viewer_instances)
    const instances = window.cosmol_viewer_instances || {{}};
    const app = instances["{id}"];
    if (app) {{
        const result = await app.{name}({escaped});
        // window.cosmol_viewer_result.set("cosmol_result", result);
    }} else {{
        console.error("No app found for ID {id}");
    }}
}})();
            "#,
            id = self.id,
        );

        let ipython = py.import("IPython.display").unwrap();
        let display = ipython.getattr("display").unwrap();

        let js = ipython
            .getattr("Javascript")
            .unwrap()
            .call1((combined_js,))
            .unwrap();
        let _ = display.call1((js,));
    }

    pub fn update(&self, py: Python, scene: &Scene) {
        self.call(py, "update_scene", scene);
    }

    pub fn take_screenshot(&self, py: Python) {
        self.call(py, "take_screenshot", None::<u8>)
    }
}

pub trait JsBridge {
    fn update(scene: &Scene) -> ();
}

#[cfg(feature = "wasm")]
#[cfg(target_arch = "wasm32")]
use eframe::WebRunner;

#[cfg(feature = "wasm")]
#[cfg(not(target_arch = "wasm32"))]
struct WebRunner;

#[cfg(feature = "wasm")]
#[cfg(not(target_arch = "wasm32"))]
impl WebRunner {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WebHandle {
    runner: WebRunner,
    app: Arc<Mutex<Option<App>>>,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WebHandle {
    
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        #[cfg(target_arch = "wasm32")]
        eframe::WebLogger::init(log::LevelFilter::Debug).ok();
        Self {
            runner: WebRunner::new(),
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

        let app = Arc::clone(&self.app);

        #[cfg(target_arch = "wasm32")]
        let _ = self
            .runner
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(move |cc| {
                    use cosmol_viewer_core::AppWrapper;

                    let mut guard = app.lock().unwrap();
                    *guard = Some(App::new(cc, scene));
                    Ok(Box::new(AppWrapper(app.clone())))
                }),
            )
            .await;
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

    #[wasm_bindgen]
    pub async fn take_screenshot(&self) -> Option<String> {
        Some("javavavavavavav".to_string())
    }
}
