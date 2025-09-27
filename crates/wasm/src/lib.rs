use cosmol_viewer_core::App;
use cosmol_viewer_core::scene::Scene;
#[cfg(feature = "js_bridge")]
use serde::Serialize;
use std::sync::Arc;
use std::sync::Mutex;

const VERSION: &str = env!("CARGO_PKG_VERSION"); // crate 当前版本号

#[cfg(feature = "wasm")]
use web_sys::HtmlCanvasElement;

#[cfg(feature = "wasm")]
use wasm_bindgen::JsValue;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(feature = "js_bridge")]
use pyo3::Python;
#[cfg(feature = "js_bridge")]
pub fn setup_wasm_if_needed(py: Python) {
    use base64::Engine;
    use pyo3::types::PyAnyMethods;

    const JS_CODE: &str = include_str!("../../wasm/pkg/cosmol_viewer_wasm.js");
    const WASM_BYTES: &[u8] = include_bytes!("../../wasm/pkg/cosmol_viewer_wasm_bg.wasm");

    let js_base64 = base64::engine::general_purpose::STANDARD.encode(JS_CODE);
    let wasm_base64 = base64::engine::general_purpose::STANDARD.encode(WASM_BYTES);

    let combined_js = format!(
        r#"
(function() {{
    const version = "{VERSION}";
    const ns = "cosmol_viewer_" + version;

    if (!window[ns + "_ready"]) {{
        // 1. setup JS module
        const jsCode = atob("{js_base64}");
        const jsBlob = new Blob([jsCode], {{ type: 'application/javascript' }});
        window[ns + "_blob_url"] = URL.createObjectURL(jsBlob);

        // 2. preload WASM
        const wasmBytes = Uint8Array.from(atob("{wasm_base64}"), c => c.charCodeAt(0));
        window[ns + "_wasm_bytes"] = wasmBytes;

        window[ns + "_ready"] = true;
        console.log("Cosmol viewer setup done, version:", version);
    }}
}})();
        "#,
        VERSION = VERSION,
        js_base64 = js_base64,
        wasm_base64 = wasm_base64
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
    pub fn initiate_viewer(py: Python, scene: &Scene, width: f32, height: f32) -> Self {
        use pyo3::types::PyAnyMethods;
        use uuid::Uuid;

        let unique_id = format!("cosmol_viewer_{}", Uuid::new_v4());

        let html_code = format!(
            r#"
<canvas id="{id}" width="{width}" height="{height}" style="width:{width}px; height:{height}px;"></canvas>
            "#,
            id = unique_id,
            width = width,
            height = height
        );

        let scene_json = serde_json::to_string(scene).unwrap();
        let escaped = serde_json::to_string(&scene_json).unwrap();

        let combined_js = format!(
            r#"
(function() {{
    const version = "{VERSION}";
    const ns = "cosmol_viewer_" + version;

    import(window[ns + "_blob_url"]).then(async (mod) => {{
        await mod.default(window[ns + "_wasm_bytes"]);

        const canvas = document.getElementById('{id}');
        const app = new mod.WebHandle();
        const sceneJson = {SCENE_JSON};
        await app.start_with_scene(canvas, sceneJson);

        window[ns + "_instances"] = window[ns + "_instances"] || {{}};
        window[ns + "_instances"]["{id}"] = app;
        console.log("Cosmol viewer instance {id} (v{VERSION}) started");
    }});
}})();
    "#,
            VERSION = VERSION,
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

    pub fn initiate_viewer_and_play(
        py: Python,
        frames: Vec<Scene>,
        interval: u64,
        loops: i64,
        width: f32,
        height: f32,
    ) -> Self {
        use pyo3::types::PyAnyMethods;
        use uuid::Uuid;

        let unique_id = format!("cosmol_viewer_{}", Uuid::new_v4());

        let html_code = format!(
            r#"
<canvas id="{id}" width="{width}" height="{height}" style="width:{width}px; height:{height}px;"></canvas>
            "#,
            id = unique_id,
            width = width,
            height = height
        );

        let frames_json = serde_json::to_string(&Frames {
            frames,
            interval,
            loops,
        })
        .unwrap();
        let escaped = serde_json::to_string(&frames_json).unwrap();

        let combined_js = format!(
            r#"
(function() {{
    const version = "{VERSION}";
    const ns = "cosmol_viewer_" + version;

    import(window[ns + "_blob_url"]).then(async (mod) => {{
        await mod.default(window[ns + "_wasm_bytes"]);

        const canvas = document.getElementById('{id}');
        const app = new mod.WebHandle();
        const framesJson = {FRAMES_JSON};
        await app.initiate_viewer_and_play(canvas, framesJson);

        window[ns + "_instances"] = window[ns + "_instances"] || {{}};
        window[ns + "_instances"]["{id}"] = app;
        console.log("Cosmol viewer instance {id} (v{VERSION}) started");
    }});
}})();
    "#,
            VERSION = VERSION,
            id = unique_id,
            FRAMES_JSON = escaped
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
    const ns = "cosmol_viewer_" + "{VERSION}";
    const instances = window[ns + "_instances"] || {{}};
    const app = instances["{id}"];
    if (app) {{
        try {{
            const result = await app.{name}({escaped});
            console.log("Call `{name}` on instance {id} (v{VERSION}) result:", result);
        }} catch (err) {{
            console.error("Error calling `{name}` on instance {id} (v{VERSION}):", err);
        }}
    }} else {{
        console.error("No app found for ID {id} in namespace", ns);
    }}
}})();
        "#,
            VERSION = VERSION,
            id = self.id,
            name = name,
            escaped = escaped
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

#[derive(serde::Serialize, serde::Deserialize)]
struct Frames {
    frames: Vec<Scene>,
    interval: u64,
    loops: i64, // -1 = infinite
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
    #[expect(clippy::new_without_default)]
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
                eframe::WebOptions {
                    // multisampling: 4, // Enable 4x MSAA
                    ..Default::default()
                },
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
    pub async fn initiate_viewer_and_play(
        &mut self,
        canvas: HtmlCanvasElement,
        frames_json: String,
    ) -> Result<(), JsValue> {
        use std::{thread, time::Duration};
        let frames: Frames = serde_json::from_str(&frames_json)
            .map_err(|e| JsValue::from_str(&format!("Frames parse error: {}", e)))?;

        let app = Arc::clone(&self.app);

        let scene = frames.frames[0].clone();

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

        let timeout_ms = 30000;
        let mut waited = 0;
        loop {
            if self.app.lock().unwrap().is_some() {
                break;
            }
            if waited > timeout_ms {
                panic!("Fail to initialize App");
            }
            gloo_timers::future::sleep(Duration::from_millis(10)).await;
            // Delay::new(Duration::from_secs(1)).await.unwrap();
            waited += 10;
        }

        let mut count = 0;
        loop {
            if frames.loops >= 0 && count >= frames.loops {
                break;
            }
            count += 1;
            for frame in &frames.frames {
                {
                    let mut guard = self.app.lock().unwrap();
                    if let Some(app) = &mut *guard {
                        app.update_scene(frame.clone());
                        app.ctx.request_repaint();
                    }
                }
                gloo_timers::future::sleep(Duration::from_millis(frames.interval)).await;
            }
        }

        Ok(())
    }

    #[wasm_bindgen]
    pub async fn take_screenshot(&self) -> Option<String> {
        Some("The returned value is omitted!".to_string())
    }
}
