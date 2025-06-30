use base64::Engine as _;
use cosmol_viewer_core::{App, utils::VisualShape};
use eframe::{
    NativeOptions,
    egui::{Vec2, ViewportBuilder},
};
use pyo3::{ffi::c_str, prelude::*};
use uuid::Uuid;

#[pyclass]
pub struct Scene {
    inner: cosmol_viewer_core::Scene,
}

#[pymethods]
impl Scene {
    #[staticmethod]
    pub fn create_viewer() -> Self {
        Self {
            inner: cosmol_viewer_core::Scene::create_viewer(),
        }
    }

    pub fn add_spheres(&mut self, sphere: Sphere) {
        self.inner.add_shapes(sphere.inner.clone());
    }
}

#[pyclass]
#[derive(Clone)]
pub struct Sphere {
    inner: cosmol_viewer_core::Sphere,
}

#[pymethods]
impl Sphere {
    #[new]
    pub fn new(center: [f32; 3], radius: f32) -> Self {
        Self {
            inner: cosmol_viewer_core::Sphere::new(center, radius),
        }
    }

    pub fn with_color(mut slf: PyRefMut<'_, Self>, color: [f32; 3]) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.with_color(color);
        slf
    }

    pub fn clickable(mut slf: PyRefMut<'_, Self>, val: bool) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clickable(val);
        slf
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
pub struct Viewer;

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
    pub fn render(scene: &Scene, py: Python) -> PyResult<()> {
        let env_type = detect_runtime_env(py)?;
        match env_type {
            RuntimeEnv::Colab | RuntimeEnv::Jupyter => {
                let unique_id = format!("cosmol_viewer_{}", Uuid::new_v4());

                const JS_CODE: &str =
                    include_str!("../../wasm/pkg/cosmol_viewer_wasm.js");
                const WASM_BYTES: &[u8] =
                    include_bytes!("../../wasm/pkg/cosmol_viewer_wasm_bg.wasm");
                let wasm_base64 = base64::engine::general_purpose::STANDARD.encode(WASM_BYTES);
                let js_base64 = base64::engine::general_purpose::STANDARD.encode(JS_CODE);

                let html_code = format!(
                    r#"
            <canvas id="{id}" width="300" height="150" style="width:300px; height:150px;"></canvas>
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
                }});
            }})();
            "#,
                    wasm_base64 = wasm_base64,
                    js_base64 = js_base64,
                    id = unique_id,
                    SCENE_JSON = escaped
                );

                let ipython = py.import("IPython.display")?;
                let display = ipython.getattr("display")?;

                let html = ipython.getattr("HTML")?.call1((html_code,))?;
                display.call1((html,))?;

                let js = ipython.getattr("Javascript")?.call1((combined_js,))?;
                display.call1((js,))?;

                return Ok(());
            }
            RuntimeEnv::PlainScript | RuntimeEnv::IPythonTerminal => {
                let native_options = NativeOptions {
                    viewport: ViewportBuilder::default().with_inner_size(Vec2::new(400.0, 250.0)),
                    depth_buffer: 24,
                    ..Default::default()
                };

                let _ = eframe::run_native(
                    "cosmol_viewer",
                    native_options,
                    Box::new(|cc| Ok(Box::new(App::new(cc, &scene.inner)))),
                );
                
                return Ok(());
            }
            _ => {
                return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                    "Unsupported runtime environment for CosmolViewer rendering.",
                ));
            }
        }
    }
}

#[pymodule]
fn cosmol_viewer(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Scene>()?;
    m.add_class::<Sphere>()?;
    m.add_class::<Viewer>()?;
    Ok(())
}
