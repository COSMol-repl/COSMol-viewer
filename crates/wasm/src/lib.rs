use base64::Engine;
use cosmol_viewer_core::scene::{Animation, Scene};
use std::io::Write;

#[cfg(feature = "wasm")]
use {
    cosmol_viewer_core::{App, utils::Logger},
    std::sync::{Arc, Mutex},
    wasm_bindgen::{JsValue, prelude::wasm_bindgen},
    web_sys::HtmlCanvasElement,
};

#[cfg(feature = "js_bridge")]
use {
    pyo3::{PyErr, PyResult, Python, ffi::c_str},
    serde::Serialize,
    std::ffi::CStr,
};

#[cfg(not(target_arch = "wasm32"))]
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(feature = "js_bridge")]
fn print_to_notebook(msg: &CStr, py: Python) {
    let _ = py.run(msg, None, None);
}

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
    }} else {{
        console.log("Cosmol viewer already set up, version:", version);
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
    pub fn initiate_viewer(py: Python, scene: &Scene, width: f32, height: f32) -> PyResult<Self> {
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

        let escaped = serde_json::to_string::<String>(
            &compress_data(&scene)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?,
        )
        .unwrap();

        let combined_js = format!(
            r#"
(function() {{
    const version = "{VERSION}";
    const ns = "cosmol_viewer_" + version;

    import(window[ns + "_blob_url"]).then(async (mod) => {{
        await mod.default(window[ns + "_wasm_bytes"]);

        const canvas = document.getElementById('{id}');
        const gl = canvas.getContext('webgl2', {{ antialias: true }});
        if (!gl) {{
            console.error("WebGL2 not supported or failed to initialize");
            return;
        }}
        const app = new mod.WebHandle();
        const scene_compressed = {SCENE};
        console.log(scene_compressed);
        await app.start_with_scene(canvas, scene_compressed);

        window[ns + "_instances"] = window[ns + "_instances"] || {{}};
        window[ns + "_instances"]["{id}"] = app;
        console.log("Cosmol viewer instance {id} (v{VERSION}) started");
    }});
}})();
    "#,
            VERSION = VERSION,
            id = unique_id,
            SCENE = escaped
        );

        print_to_notebook(c_str!("Scene compressed: {scene_compressed}"), py);
        let ipython = py.import("IPython.display")?;
        let display = ipython.getattr("display")?;

        let html = ipython
            .getattr("HTML")
            .unwrap()
            .call1((html_code,))
            .unwrap();
        display.call1((html,))?;

        let js = ipython
            .getattr("Javascript")
            .unwrap()
            .call1((combined_js,))
            .unwrap();
        display.call1((js,))?;

        Ok(Self { id: unique_id })
    }

    pub fn initiate_viewer_and_play(
        py: Python,
        animation: Animation,
        width: f32,
        height: f32,
    ) -> PyResult<Self> {
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

        let escaped = serde_json::to_string::<String>(
            &compress_data(&animation)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?,
        )
        .unwrap();

        let combined_js = format!(
            r#"
(function() {{
    const version = "{VERSION}";
    const ns = "cosmol_viewer_" + version;

    import(window[ns + "_blob_url"]).then(async (mod) => {{
        await mod.default(window[ns + "_wasm_bytes"]);

        const canvas = document.getElementById('{id}');
        const gl = canvas.getContext('webgl2', {{ antialias: true }});
        if (!gl) {{
            console.error("WebGL2 not supported or failed to initialize");
            return;
        }}
        const app = new mod.WebHandle();
        const animation_compressed = {ANIMATION};
        await app.initiate_viewer_and_play(canvas, animation_compressed);

        window[ns + "_instances"] = window[ns + "_instances"] || {{}};
        window[ns + "_instances"]["{id}"] = app;
        console.log("Cosmol viewer instance {id} (v{VERSION}) started");
    }});
}})();
    "#,
            VERSION = VERSION,
            id = unique_id,
            ANIMATION = escaped
        );
        let ipython = py.import("IPython.display")?;
        let display = ipython.getattr("display")?;

        let html = ipython
            .getattr("HTML")
            .unwrap()
            .call1((html_code,))
            .unwrap();
        display.call1((html,))?;

        let js = ipython
            .getattr("Javascript")
            .unwrap()
            .call1((combined_js,))
            .unwrap();
        display.call1((js,))?;

        Ok(Self { id: unique_id })
    }

    pub fn call<T: Serialize>(&self, py: Python, name: &str, input: T) -> PyResult<()> {
        use pyo3::types::PyAnyMethods;

        let escaped = serde_json::to_string::<String>(
            &compress_data(&input)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?,
        )
        .unwrap();
        let combined_js = format!(
            r#"
(async function() {{
    const ns = "cosmol_viewer_" + "{VERSION}";
    const instances = window[ns + "_instances"] || {{}};
    const app = instances["{id}"];
    if (app) {{
        try {{
            const result = await app.{name}({escaped});
            // console.log("Call `{name}` on instance {id} (v{VERSION}) result:", result);
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

        let ipython = py.import("IPython.display")?;
        let display = ipython.getattr("display")?;

        let js = ipython
            .getattr("Javascript")
            .unwrap()
            .call1((combined_js,))
            .unwrap();
        display.call1((js,))?;
        Ok(())
    }

    pub fn update(&self, py: Python, scene: &Scene) -> PyResult<()> {
        self.call(py, "update_scene", scene)
    }

    pub fn take_screenshot(&self, py: Python) -> PyResult<()> {
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
#[derive(Clone, Copy)]
pub struct WasmLogger;

#[cfg(feature = "wasm")]
impl Logger for WasmLogger {
    fn log(&self, message: impl std::fmt::Display) {
        web_sys::console::log_1(&JsValue::from_str(&message.to_string()));
    }

    fn warn(&self, message: impl std::fmt::Display) {
        web_sys::console::warn_1(&JsValue::from_str(&message.to_string()));
    }

    fn error(&self, message: impl std::fmt::Display) {
        let msg = message.to_string();

        // Send to console
        web_sys::console::error_1(&JsValue::from_str(&msg));

        // Show browser alert
        if let Some(window) = web_sys::window() {
            window.alert_with_message(&msg).ok();
        }
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WebHandle {
    #[cfg(target_arch = "wasm32")]
    runner: WebRunner,
    app: Arc<Mutex<Option<App<WasmLogger>>>>,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WebHandle {
    #[wasm_bindgen(constructor)]
    #[expect(clippy::new_without_default)]
    pub fn new() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            eframe::WebLogger::init(log::LevelFilter::Debug).ok();
            Self {
                runner: WebRunner::new(),
                app: Arc::new(Mutex::new(None)),
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self {
                app: Arc::new(Mutex::new(None)),
            }
        }
    }

    #[wasm_bindgen]
    pub async fn start_with_scene(
        &mut self,
        _canvas: HtmlCanvasElement,
        scene_json: String,
    ) -> Result<(), JsValue> {
        let _scene: Scene =
            decompress_data(&scene_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

        println!("{:?}", _scene);
        web_sys::console::log_1(&JsValue::from_str(&scene_json.to_string()));
        web_sys::console::log_1(&JsValue::from_str(format!("{:?}", _scene).as_str()));

        #[cfg(target_arch = "wasm32")]
        {
            let app = Arc::clone(&self.app);

            let _ = self
                .runner
                .start(
                    _canvas,
                    eframe::WebOptions {
                        // multisampling: 4, // Enable 4x MSAA
                        ..Default::default()
                    },
                    Box::new(move |cc| {
                        use cosmol_viewer_core::AppWrapper;

                        let mut guard = app.lock().unwrap();
                        *guard = Some(App::new(cc, _scene, WasmLogger));
                        Ok(Box::new(AppWrapper(app.clone())))
                    }),
                )
                .await;
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub async fn update_scene(&mut self, scene_json: String) -> Result<(), JsValue> {
        let scene: Scene =
            decompress_data(&scene_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

        let mut app_guard = self.app.lock().unwrap();
        if let Some(app) = &mut *app_guard {
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
        _canvas: HtmlCanvasElement,
        _animation_compressed: String,
    ) -> Result<(), JsValue> {
        #[cfg(target_arch = "wasm32")]
        {
            let payload = _animation_compressed.to_string();
            let kb = payload.as_bytes().len() as f64 / 1024.0;
            web_sys::console::log_1(&format!("Transmission size: {kb:.2} KB").into());

            let animation: Animation = decompress_data(&_animation_compressed)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;

            let app = Arc::clone(&self.app);
            let _ = self
                .runner
                .start(
                    _canvas,
                    eframe::WebOptions::default(),
                    Box::new(move |cc| {
                        use cosmol_viewer_core::AppWrapper;

                        let mut guard = app.lock().unwrap();
                        *guard = Some(App::new_play(cc, animation, WasmLogger));
                        Ok(Box::new(AppWrapper(app.clone())))
                    }),
                )
                .await;
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub async fn take_screenshot(&self) -> Option<String> {
        Some("The returned value is omitted!".to_string())
    }
}

use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use std::io::Read;
/// Compress data into a base64-encoded string.
pub fn compress_data<T: serde::Serialize>(value: &T) -> Result<String, postcard::Error> {
    let bytes = postcard::to_allocvec(value)?;
    let mut encoder = GzEncoder::new(Vec::with_capacity(bytes.len() / 2), Compression::fast());
    encoder.write_all(&bytes).unwrap();
    let compressed = encoder.finish().unwrap();
    Ok(base64::engine::general_purpose::STANDARD.encode(compressed))
}

#[derive(thiserror::Error, Debug)]
pub enum DecompressError {
    #[error("base64 decode failed: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("gzip decode failed: {0}")]
    Gzip(#[from] std::io::Error),

    #[error("postcard decode failed: {0}")]
    Postcard(#[from] postcard::Error),
}

/// Decompress data from a base64-encoded string.
pub fn decompress_data<T: for<'de> serde::Deserialize<'de>>(s: &str) -> Result<T, DecompressError> {
    let compressed = base64::engine::general_purpose::STANDARD.decode(s)?;
    let mut decoder = GzDecoder::new(&*compressed);
    let mut bytes = Vec::with_capacity(compressed.len());
    decoder.read_to_end(&mut bytes)?;
    Ok(postcard::from_bytes(&bytes)?)
}
