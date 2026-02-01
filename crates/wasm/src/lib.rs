use base64::Engine;
use cosmol_viewer_core::BUILD_ID;
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
    pyo3::prelude::PyModule,
    pyo3::{PyErr, PyResult, Python, ffi::c_str},
    serde::Serialize,
    serde::de::DeserializeOwned,
    std::ffi::CStr,
};

#[cfg(feature = "js_bridge")]
fn print_to_notebook(msg: &CStr, py: Python) {
    let _ = py.run(msg, None, None);
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
    const version = "{BUILD_ID}";
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
        console.log("Cosmol viewer instance {id} (v{BUILD_ID}) started");
    }});
}})();
    "#,
            BUILD_ID = BUILD_ID,
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
    const version = "{BUILD_ID}";
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
        console.log("Cosmol viewer instance {id} (v{BUILD_ID}) started");
    }});
}})();
    "#,
            BUILD_ID = BUILD_ID,
            id = unique_id,
            ANIMATION = escaped
        );
        let ipython = py.import("IPython.display")?;
        let display = ipython.getattr("display")?;

        let html = ipython.getattr("HTML")?.call1((html_code,))?;
        display.call1((html,))?;

        let js = ipython.getattr("Javascript")?.call1((combined_js,))?;
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
    const ns = "cosmol_viewer_" + "{BUILD_ID}";
    const instances = window[ns + "_instances"] || {{}};
    const app = instances["{id}"];
    if (app) {{
        try {{
            const result = await app.{name}({escaped});
            // console.log("Call `{name}` on instance {id} (v{BUILD_ID}) result:", result);
        }} catch (err) {{
            console.error("Error calling `{name}` on instance {id} (v{BUILD_ID}):", err);
        }}
    }} else {{
        console.error("No app found for ID {id} in namespace", ns);
    }}
}})();
        "#,
            BUILD_ID = BUILD_ID,
            id = self.id,
            name = name,
            escaped = escaped
        );

        let ipython = py.import("IPython.display")?;
        let display = ipython.getattr("display")?;

        let js = ipython.getattr("Javascript")?.call1((combined_js,))?;
        display.call1((js,))?;
        Ok(())
    }

    pub fn call_colab_with_return<T: Serialize, R: DeserializeOwned>(
        &self,
        py: Python,
        name: &str,
        input: T,
    ) -> PyResult<R> {
        use pyo3::exceptions::PyRuntimeError;
        use pyo3::types::PyAnyMethods;

        let escaped = serde_json::to_string(
            &compress_data(&input)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?,
        )
        .unwrap();

        let combined_js = format!(
            r#"
    (async function() {{
        const ns = "cosmol_viewer_{BUILD_ID}";
        const instances = window[ns + "_instances"] || {{}};
        const app = instances["{id}"];
        if (app) {{
            try {{
                const result = await app.{name}({escaped});
                return result;
            }} catch (err) {{
                console.error("Error calling `{name}`:", err);
                throw err;
            }}
        }} else {{
            throw new Error("No app found for ID {id}");
        }}
    }})();
            "#,
            BUILD_ID = BUILD_ID,
            id = self.id,
            name = name,
            escaped = escaped
        );

        let colab_output = py.import("google.colab")?;
        let output = colab_output.getattr("output")?;

        let py_result = output.call1((combined_js,))?;

        let res: &str = py_result.extract()?;

        let result: R = decompress_data(&res).map_err(|e| {
            PyErr::new::<PyRuntimeError, _>(format!(
                "Failed to decompress data: {}, data: {}",
                e, res
            ))
        })?;

        Ok(result)
    }

    /// 基于 ipywidgets 实现的 Jupyter 同步回传函数
    pub fn call_jupyter_with_return<T: Serialize, R: DeserializeOwned>(
        &self,
        py: Python<'_>,
        name: &str,
        input: T,
    ) -> PyResult<R> {
        use pyo3::Bound;
        use pyo3::exceptions::{PyModuleNotFoundError, PyRuntimeError};
        use pyo3::ffi::c_str;
        use pyo3::types::{PyAnyMethods, PyModule, PyTuple};
        use std::time::{Duration, Instant};

        // 1. 序列化并压缩输入数据
        let compressed_input =
            compress_data(&input).map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;
        let escaped_input = serde_json::to_string(&compressed_input).unwrap();

        py.import("ipywidgets").map_err(|e| {
            PyErr::new::<PyModuleNotFoundError, _>(
                format!("{}, this feature requires ipywidgets...", e)
                    .replace("ModuleNotFoundError: ", ""),
            )
        })?;

        // 2. 增强版 Python 桥接模块：增加异步处理逻辑
        let bridge_module = PyModule::from_code(
            py,
            c_str!(
                "
import ipywidgets as widgets
from traitlets import Any
import asyncio
import inspect

class CosmolBridge(widgets.DOMWidget):
    response = Any(None).tag(sync=True)

def pump(kernel):
    '''统一处理同步和异步的内核迭代'''
    res = kernel.do_one_iteration()
    if inspect.isawaitable(res):
        try:
            loop = asyncio.get_event_loop()
            if loop.is_running():
                # 如果 loop 已经在运行（Jupyter 正常状态），我们需要 nest_asyncio
                # 或者尝试通过 run_until_complete 的变体来驱动。
                # 在同步阻塞的情况下，最简单的是让 loop 运行直到该协程完成
                import nest_asyncio
                nest_asyncio.apply()
                loop.run_until_complete(res)
            else:
                loop.run_until_complete(res)
        except Exception as e:
            # 最后的保底措施，如果环境极其特殊
            pass

def create_bridge():
    b = CosmolBridge()
    mid = getattr(b, 'model_id', None)
    if mid is None and hasattr(b, 'comm'):
        mid = b.comm.comm_id
    return b, mid
                "
            ),
            c_str!("bridge.py"),
            c_str!("bridge"),
        )?;

        // 创建实例
        let create_fn = bridge_module.getattr("create_bridge")?;
        let out = create_fn.call0()?;
        let out_tuple: Bound<'_, PyTuple> = out.cast_into()?;

        let bridge = out_tuple.get_item(0)?;
        let comm_id: String = out_tuple.get_item(1)?.extract()?;

        // 3. 构建 JS (逻辑保持不变)
        let combined_js = format!(
            r#"
            (async function() {{
                const ns = "cosmol_viewer_{BUILD_ID}";
                const app = (window[ns + "_instances"] || {{}})["{id}"];
                try {{
                    if (!app) throw new Error("App {id} not found");
                    const result = await app.{name}({escaped_input});
                    if (window.require) {{
                        window.require(["@jupyter-widgets/base"], (widgets) => {{
                            widgets.unpack_models({{ model_id: "{comm_id}" }}).then((model) => {{
                                model.set("response", result);
                                model.save_changes();
                            }});
                        }});
                    }}
                }} catch (err) {{ console.error("Jupyter Bridge Error:", err); }}
            }})();
            "#,
            BUILD_ID = BUILD_ID,
            id = self.id,
            name = name,
            escaped_input = escaped_input,
            comm_id = comm_id
        );

        // 4. 显示 JS
        let display_mod = py.import("IPython.display")?;
        let js_obj = display_mod.getattr("Javascript")?.call1((combined_js,))?;
        display_mod.call_method1("display", (js_obj,))?;

        // 5. 核心：调用 Python 端的 pump 函数进行轮询
        let kernel = py
            .import("IPython")?
            .call_method0("get_ipython")?
            .getattr("kernel")?;
        let pump_fn = bridge_module.getattr("pump")?;

        let start = Instant::now();
        let timeout = Duration::from_secs(30);

        while bridge.getattr("response")?.is_none() {
            // 修改点：调用我们封装好的 pump 而不是直接 kernel.do_one_iteration
            pump_fn.call1((&kernel,))?;

            if start.elapsed() > timeout {
                return Err(PyErr::new::<PyRuntimeError, _>("Jupyter RPC Timeout"));
            }
            std::thread::sleep(Duration::from_millis(20));
        }

        // 6. 提取结果
        let py_res = bridge.getattr("response")?;
        let res_str: String = py_res.extract()?;
        let result: R = decompress_data(&res_str)
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("Decompress error: {}", e)))?;

        Ok(result)
    }
    pub fn update(&self, py: Python, scene: &Scene) -> PyResult<()> {
        self.call(py, "update_scene", scene)
    }

    pub fn take_screenshot_colab(&self, py: Python) -> PyResult<Vec<u8>> {
        let img_buf: Vec<u8> = self.call_colab_with_return(py, "take_screenshot", None::<u8>)?;
        Ok(img_buf)
    }

    pub fn take_screenshot_jupyter(&self, py: Python) -> PyResult<Vec<u8>> {
        let img_buf: Vec<u8> = self.call_jupyter_with_return(py, "take_screenshot", None::<u8>)?;
        Ok(img_buf)
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
    wasm_logger: WasmLogger,
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
                wasm_logger: WasmLogger,
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self {
                app: Arc::new(Mutex::new(None)),
                wasm_logger: WasmLogger,
            }
        }
    }

    #[wasm_bindgen]
    pub async fn start_with_scene(
        &mut self,
        _canvas: HtmlCanvasElement,
        _scene_json: String,
    ) -> Result<(), JsValue> {
        #[cfg(target_arch = "wasm32")]
        {
            let scene: Scene =
                decompress_data(&_scene_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
            let app = Arc::clone(&self.app);

            let _ = self
                .runner
                .start(
                    _canvas,
                    eframe::WebOptions {
                        ..Default::default()
                    },
                    Box::new(move |cc| {
                        use cosmol_viewer_core::AppWrapper;

                        let mut guard = app.lock().unwrap();
                        *guard = Some(App::new(cc, &scene, WasmLogger));
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
            app.update_scene(&scene);
            app.ctx.request_repaint();
        } else {
            self.wasm_logger
                .warn("scene update received but app is not initialized");
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
    pub async fn take_screenshot(&self) -> String {
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
                    let mut buf = Vec::new();
                    self.wasm_logger.log(format!("image:{:?}", buf));
                    image
                        .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
                        .unwrap();
                    return compress_data(&buf).unwrap();
                }
            }
            drop(app_guard);
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
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
