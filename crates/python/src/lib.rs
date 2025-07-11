use std::{env, fs::File, io::Write};
use base64::Engine as _;
use ipc_channel::ipc::{IpcOneShotServer, IpcSender};
use pyo3::{ffi::c_str, prelude::*};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use cosmol_viewer_core::{scene::Scene as _Scene};
use crate::{parser::parse_sdf, shapes::{PyMolecules, PySphere, PyStick}};

mod parser;
mod shapes;

#[pyclass]
pub struct Scene {
    inner: _Scene,
}

#[pymethods]
impl Scene {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: _Scene::new(),
        }
    }

    #[pyo3(signature = (shape, id=None))]
    pub fn add_shape(&mut self, shape: &Bound<'_, PyAny>, id: Option<&str>) {
        if let Ok(sphere) = shape.extract::<PyRef<PySphere>>() {
            self.inner.add_shape(sphere.inner.clone(), id);
        } else if let Ok(stick) = shape.extract::<PyRef<PyStick>>() {
            self.inner.add_shape(stick.inner.clone(), id);
        } else if let Ok(molecules) = shape.extract::<PyRef<PyMolecules>>() {
            self.inner.add_shape(molecules.inner.clone(), id);
        }
        ()
    }

    pub fn update_shape(&mut self, id: &str, shape: &Bound<'_, PyAny>) {
        if let Ok(sphere) = shape.extract::<PyRef<PySphere>>() {
            self.inner.update_shape(id, sphere.inner.clone());
        } else if let Ok(stick) = shape.extract::<PyRef<PyStick>>() {
            self.inner.update_shape(id, stick.inner.clone());
        } else if let Ok(molecules) = shape.extract::<PyRef<PyMolecules>>() {
            self.inner.update_shape(id, molecules.inner.clone());
        }
    }

    pub fn delete_shape(&mut self, id: &str) {
        self.inner.delete_shape(id);
    }

    pub fn scale(&mut self, scale: f32) {
        self.inner.scale(scale);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeEnv {
    Colab,
    Jupyter,
    IPythonTerminal,
    IPythonOther,
    PlainScript,
    Unknown,
}

impl std::fmt::Display for RuntimeEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            RuntimeEnv::Colab => "Colab",
            RuntimeEnv::Jupyter => "Jupyter",
            RuntimeEnv::IPythonTerminal => "IPython-Terminal",
            RuntimeEnv::IPythonOther => "Other IPython",
            RuntimeEnv::PlainScript => "Plain Script",
            RuntimeEnv::Unknown => "Unknown",
        };
        write!(f, "{}", s)
    }
}

#[pyclass]
#[pyo3(crate = "pyo3", unsendable)]
pub struct Viewer {
    sender: Option<IpcSender<_Scene>>,
    environment: RuntimeEnv,
    canvas_id: Option<String>,
}

fn detect_runtime_env(py: Python) -> PyResult<RuntimeEnv> {
    let code = c_str!(
        r#"
def detect_env():
    import sys
    try:
        from IPython import get_ipython
        ipy = get_ipython()
        if ipy is None:
            return 'PlainScript'
        shell = ipy.__class__.__name__
        if 'google.colab' in sys.modules:
            return 'Colab'
        if shell == 'ZMQInteractiveShell':
            return 'Jupyter'
        elif shell == 'TerminalInteractiveShell':
            return 'IPython-Terminal'
        else:
            return f'IPython-{shell}'
    except:
        return 'PlainScript'
"#
    );

    let env_module = PyModule::from_code(py, code, c_str!("<detect_env>"), c_str!("env_module"))?;
    let fun = env_module.getattr("detect_env")?;
    let result: String = fun.call1(())?.extract()?;

    let env = match result.as_str() {
        "Colab" => RuntimeEnv::Colab,
        "Jupyter" => RuntimeEnv::Jupyter,
        "IPython-Terminal" => RuntimeEnv::IPythonTerminal,
        s if s.starts_with("IPython-") => RuntimeEnv::IPythonOther,
        "PlainScript" => RuntimeEnv::PlainScript,
        _ => RuntimeEnv::Unknown,
    };

    Ok(env)
}

#[pymethods]
impl Viewer {
    #[staticmethod]
    pub fn get_environment(py: Python) -> PyResult<String> {
        let env = detect_runtime_env(py)?;
        Ok(env.to_string())
    }

    #[staticmethod]
    pub fn render(scene: &Scene, py: Python) -> Self {
        let env_type = detect_runtime_env(py).unwrap();
        match env_type {
            RuntimeEnv::Colab | RuntimeEnv::Jupyter => {
                let unique_id = format!("cosmol_viewer_{}", Uuid::new_v4());

                const JS_CODE: &str = include_str!("../../wasm/pkg/cosmol_viewer_wasm.js");
                const WASM_BYTES: &[u8] =
                    include_bytes!("../../wasm/pkg/cosmol_viewer_wasm_bg.wasm");
                let wasm_base64 = base64::engine::general_purpose::STANDARD.encode(WASM_BYTES);
                let js_base64 = base64::engine::general_purpose::STANDARD.encode(JS_CODE);

                let html_code = format!(
                    r#"
            <canvas id="{id}" width="600" height="400" style="width:600px; height:400px;"></canvas>
            "#,
                    id = unique_id
                );

                let scene_json = serde_json::to_string(&scene.inner).unwrap();
                let escaped = serde_json::to_string(&scene_json).unwrap();

                let combined_js = format!(
                    r#"
            (function() {{
                const wasmBase64 = "{wasm_base64}";
                const jsBase64 = "{js_base64}";

                // 创建 Blob 链接
                const jsCode = atob(jsBase64);
                const blob = new Blob([jsCode], {{ type: 'application/javascript' }});
                const blobUrl = URL.createObjectURL(blob);

                import(blobUrl).then(async (mod) => {{
                    const wasmBytes = Uint8Array.from(atob(wasmBase64), c => c.charCodeAt(0));
                    await mod.default(wasmBytes);

                    const canvas = document.getElementById('{id}');
                    const app = new mod.WebHandle();
                    const sceneJson = {SCENE_JSON};
                    console.log("Starting cosmol_viewer with scene:", sceneJson);
                    await app.start_with_scene(canvas, sceneJson);

                    // ✅ 注册到全局，方便后续更新
                    window.cosmol_viewer_instances = window.cosmol_viewer_instances || {{}};
                    window.cosmol_viewer_instances["{id}"] = {{
                        app: app,
                        canvas: canvas,
                    }};
                }});
            }})();
            "#,
                    wasm_base64 = wasm_base64,
                    js_base64 = js_base64,
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

                Viewer {
                    sender: None,
                    environment: env_type,
                    canvas_id: Some(unique_id),
                }
            }
            RuntimeEnv::PlainScript | RuntimeEnv::IPythonTerminal => {
                let (server, server_name) = IpcOneShotServer::<IpcSender<_Scene>>::new().unwrap();

                extract_and_run_gui(&server_name)
                    .expect("Failed to extract and run GUI executable");

                let (_, sender) = server.accept().unwrap();
                sender.send(scene.inner.clone()).unwrap();
                Viewer {
                    sender: Some(sender),
                    environment: env_type,
                    canvas_id: None,
                }
            }
            _ => Viewer {
                sender: None,
                environment: env_type,
                canvas_id: None,
            },
        }
    }

    pub fn update(&mut self, scene: &Scene, py: Python) -> PyResult<String> {
        let env_type = self.environment;
        match env_type {
            RuntimeEnv::Colab | RuntimeEnv::Jupyter => {
                let scene_json = serde_json::to_string(&scene.inner).unwrap();
                let escaped = serde_json::to_string(&scene_json).unwrap();
                let combined_js = format!(
                    r#"
(function() {{
    const instances = window.cosmol_viewer_instances || {{}};
    const handle = instances["{id}"];
    if (handle) {{
        const sceneJson = {SCENE_JSON};
        handle.app.update_scene(sceneJson);
    }} else {{
        console.error("No app found for ID {id}");
    }}
}})();
"#,
                    id = self.canvas_id.clone().unwrap(),
                    SCENE_JSON = escaped
                );

                let ipython = py.import("IPython.display").unwrap();
                let display = ipython.getattr("display").unwrap();

                let js = ipython
                    .getattr("Javascript")
                    .unwrap()
                    .call1((combined_js,))
                    .unwrap();
                display.call1((js,)).unwrap();

                Ok("Scene updated successfully".to_string())
            }
            RuntimeEnv::PlainScript | RuntimeEnv::IPythonTerminal => {
                if let Some(sender) = &self.sender {
                    sender.send(scene.inner.clone()).unwrap();
                    Ok("Scene updated successfully".to_string())
                } else {
                    Err(pyo3::exceptions::PyRuntimeError::new_err(
                        "Viewer is not initialized with a sender",
                    ))
                }
            }
            _ => Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Viewer is not initialized with a sender",
            )),
        }
    }
}

#[pymodule]
fn cosmol_viewer(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Scene>()?;
    m.add_class::<PySphere>()?;
    m.add_class::<PyStick>()?;
    m.add_class::<PyMolecules>()?;
    m.add_class::<Viewer>()?;
    m.add_function(wrap_pyfunction!(parse_sdf, m)?)?;
    Ok(())
}

#[cfg(all(debug_assertions, target_os = "windows"))]
const GUI_EXE_BYTES: &[u8] = include_bytes!("../../../target/debug/cosmol_viewer_gui.exe");

#[cfg(all(debug_assertions, target_os = "linux"))]
const GUI_EXE_BYTES: &[u8] = include_bytes!("../../../target/debug/cosmol_viewer_gui");

#[cfg(all(not(debug_assertions), target_os = "windows"))]
const GUI_EXE_BYTES: &[u8] = include_bytes!("../../../target/release/cosmol_viewer_gui.exe");

#[cfg(all(not(debug_assertions), target_os = "linux"))]
const GUI_EXE_BYTES: &[u8] = include_bytes!("../../../target/release/cosmol_viewer_gui");

fn calculate_gui_hash() -> String {
    let result = Sha256::digest(GUI_EXE_BYTES);
    hex::encode(result)
}

fn extract_and_run_gui(arg: &str) -> std::io::Result<()> {
    let tmp_dir = env::temp_dir();
    let exe_path = tmp_dir.join(format!("cosmol_temp_gui_{}.exe", calculate_gui_hash()));

    if !exe_path.exists() {
        let mut file = File::create(&exe_path)?;
        file.write_all(GUI_EXE_BYTES)?;
    }

    println!("Launching GUI from: {}", exe_path.display());

    std::process::Command::new(&exe_path)
        .arg(arg)
        .spawn()
        .expect("Failed to launch GUI process");

    Ok(())
}
