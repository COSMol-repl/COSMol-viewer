use std::ffi::CStr;

use base64::Engine as _;
use pyo3::{exceptions::PyTypeError, ffi::c_str, prelude::*};

use crate::{
    parser::parse_sdf,
    shapes::{PyMolecules, PySphere, PyStick},
};
use cosmol_viewer_core::{NativeGuiViewer, scene::Scene as _Scene};
use cosmol_viewer_wasm::{WasmViewer, setup_wasm_if_needed};
use std::borrow::Borrow;

mod parser;
mod shapes;

#[derive(Clone)]
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
        } else {
            panic!("Unsupported shape type");
        }
    }

    pub fn delete_shape(&mut self, id: &str) {
        self.inner.delete_shape(id);
    }

    pub fn scale(&mut self, scale: f32) {
        self.inner.scale(scale);
    }

    pub fn set_background_color(&mut self, background_color: [f32; 3]) {
        self.inner.set_background_color(background_color);
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
    environment: RuntimeEnv,
    wasm_viewer: Option<WasmViewer>,
    native_gui_viewer: Option<NativeGuiViewer>,
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
    pub fn render(scene: &Scene, width: f32, height: f32, py: Python) -> Self {
        let env_type = detect_runtime_env(py).unwrap();
        match env_type {
            RuntimeEnv::Colab | RuntimeEnv::Jupyter => {
                print_to_notebook(
                    c_str!(
                        r#"from IPython.display import display, HTML
display(HTML("<div style='color:red;font-weight:bold;font-size:1rem;'>⚠️ Note: When running in Jupyter or Colab, animation updates may be limited by the notebook's output capacity, which can cause incomplete or delayed rendering.</div>"))"#
                    ),
                    py,
                );
                setup_wasm_if_needed(py);
                let wasm_viewer = WasmViewer::initiate_viewer(py, &scene.inner, width, height);

                Viewer {
                    environment: env_type,
                    wasm_viewer: Some(wasm_viewer),
                    native_gui_viewer: None,
                }
            }
            RuntimeEnv::PlainScript | RuntimeEnv::IPythonTerminal => Viewer {
                environment: env_type,
                wasm_viewer: None,
                native_gui_viewer: Some(NativeGuiViewer::render(&scene.inner, width, height)),
            },
            _ => panic!("Error: Invalid runtime environment"),
        }
    }

    #[staticmethod]
    pub fn play(
        frames: Vec<Scene>,
        interval: f32,
        loops: i64,
        width: f32,
        height: f32,
        py: Python,
    ) -> Self {
        let env_type = detect_runtime_env(py).unwrap();
        let rust_frames: Vec<_Scene> = frames.iter().map(|frame| frame.inner.clone()).collect();

        match env_type {
            RuntimeEnv::Colab | RuntimeEnv::Jupyter => {
                setup_wasm_if_needed(py);
                let wasm_viewer = WasmViewer::initiate_viewer_and_play(py, rust_frames, (interval * 1000.0) as u64, loops, width, height);

                Viewer {
                    environment: env_type,
                    wasm_viewer: Some(wasm_viewer),
                    native_gui_viewer: None,
                }
            }

            RuntimeEnv::PlainScript | RuntimeEnv::IPythonTerminal => {
                NativeGuiViewer::play(rust_frames, interval, loops, width, height);

                Viewer {
                    environment: env_type,
                    wasm_viewer: None,
                    native_gui_viewer: None,
                }
            }
            _ => panic!("Error: Invalid runtime environment"),
        }
    }

    pub fn update(&mut self, scene: &Scene, py: Python) {
        let env_type = self.environment;
        match env_type {
            RuntimeEnv::Colab | RuntimeEnv::Jupyter => {
                if let Some(ref wasm_viewer) = self.wasm_viewer {
                    wasm_viewer.update(py, &scene.inner);
                } else {
                    panic!("Viewer is not initialized properly")
                }
            }
            RuntimeEnv::PlainScript | RuntimeEnv::IPythonTerminal => {
                if let Some(ref mut native_gui_viewer) = self.native_gui_viewer {
                    native_gui_viewer.update(&scene.inner);
                } else {
                    panic!("Viewer is not initialized properly")
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn save_image(&self, path: &str, py: Python) {
        let env_type = self.environment;
        match env_type {
            RuntimeEnv::Colab | RuntimeEnv::Jupyter => {
                // let image = self.wasm_viewer.as_ref().unwrap().take_screenshot(py);
                print_to_notebook(
                    c_str!(
                        r#"<div style='color:red;font-weight:bold;font-size:1rem;'>⚠️ Image saving in Jupyter/Colab is not yet fully supported.</div>"))"#
                    ),
                    py,
                );
                panic!(
                    "Error saving image. Saving images from Jupyter/Colab is not yet supported."
                )
            }
            RuntimeEnv::PlainScript | RuntimeEnv::IPythonTerminal => {
                let native_gui_viewer = &self.native_gui_viewer.as_ref().unwrap();
                let img = native_gui_viewer.take_screenshot();
                if let Err(e) = img.save(path) {
                    panic!("{}", format!("Error saving image: {}", e))
                }
            }
            _ => unreachable!(),
        }
    }
}

fn print_to_notebook(msg: &CStr, py: Python) {
    let _ = py.run(msg, None, None);
}

#[pymodule]
fn cosmol_viewer(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Scene>()?;
    m.add_class::<Viewer>()?;
    m.add_class::<PySphere>()?;
    m.add_class::<PyStick>()?;
    m.add_class::<PyMolecules>()?;
    m.add_function(wrap_pyfunction!(parse_sdf, m)?)?;
    Ok(())
}
