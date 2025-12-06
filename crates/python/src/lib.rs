use pyo3::exceptions::PyRuntimeError;
use pyo3::exceptions::PyValueError;
use std::ffi::CStr;

use pyo3::{ffi::c_str, prelude::*};

use crate::{
    parser::{parse_mmcif, parse_sdf},
    shapes::{PyMolecules, PyProtein, PySphere, PyStick},
};
use cosmol_viewer_core::{NativeGuiViewer, scene::Scene as _Scene};
use cosmol_viewer_wasm::{WasmViewer, setup_wasm_if_needed};

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
    pub fn add_shape(&mut self, shape: &Bound<'_, PyAny>, id: Option<&str>) -> PyResult<()> {
        macro_rules! try_add {
            ($py_type:ty) => {{
                if let Ok(py_obj) = shape.extract::<PyRef<$py_type>>() {
                    self.inner.add_shape(py_obj.inner.clone(), id);
                    return Ok(());
                }
            }};
        }

        try_add!(PySphere);
        try_add!(PyStick);
        try_add!(PyMolecules);
        try_add!(PyProtein);

        let type_name = shape
            .get_type()
            .name()
            .map(|name| name.to_string())
            .unwrap_or("<unknown type>".to_string());

        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
            "add_shape(): unsupported shape type '{type_name}'. \
             Expected one of: Sphere, Stick, Molecules, Protein"
        )))
    }

    pub fn update_shape(&mut self, id: &str, shape: &Bound<'_, PyAny>) -> PyResult<()> {
        macro_rules! update_with {
            ($py_type:ty) => {{
                if let Ok(py_obj) = shape.extract::<PyRef<$py_type>>() {
                    self.inner.update_shape(id, py_obj.inner.clone());
                    return Ok(());
                }
            }};
        }

        update_with!(PySphere);
        update_with!(PyStick);
        update_with!(PyMolecules);
        update_with!(PyProtein);

        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
            "update_shape(): unsupported type {}",
            shape.get_type().name()?
        )))
    }

    pub fn delete_shape(&mut self, id: &str) {
        self.inner.delete_shape(id);
    }

    pub fn recenter(&mut self, center: [f32; 3]) {
        self.inner.recenter(center);
    }

    pub fn scale(&mut self, scale: f32) {
        self.inner.scale(scale);
    }

    pub fn set_background_color(&mut self, background_color: [f32; 3]) {
        self.inner.set_background_color(background_color);
    }

    pub fn use_black_background(&mut self) {
        self.inner.use_black_background();
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
    first_update: bool,
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
    pub fn render(scene: &Scene, width: f32, height: f32, py: Python) -> PyResult<Self> {
        let env_type = detect_runtime_env(py)?;
        match env_type {
            RuntimeEnv::Colab | RuntimeEnv::Jupyter => {
                setup_wasm_if_needed(py);
                let wasm_viewer = WasmViewer::initiate_viewer(py, &scene.inner, width, height);

                Ok(Viewer {
                    environment: env_type,
                    wasm_viewer: Some(wasm_viewer),
                    native_gui_viewer: None,
                    first_update: true,
                })
            }
            RuntimeEnv::PlainScript | RuntimeEnv::IPythonTerminal => {
                let native_gui_viewer = std::panic::catch_unwind(|| {
                    NativeGuiViewer::render(&scene.inner, width, height)
                });

                if let Err(err) = native_gui_viewer {
                    return Err(PyRuntimeError::new_err(format!(
                        "Error: Failed to initialize native GUI viewer: {:?}",
                        err
                    )));
                }

                Ok(Viewer {
                    environment: env_type,
                    wasm_viewer: None,
                    native_gui_viewer: Some(native_gui_viewer.unwrap()),
                    first_update: true,
                })
            }
            _ => Err(PyValueError::new_err("Error: Invalid runtime environment")),
        }
    }

    #[staticmethod]
    pub fn play(
        frames: Vec<Scene>,
        interval: f32,
        loops: i64,
        width: f32,
        height: f32,
        smooth: bool,
        py: Python,
    ) -> PyResult<Self> {
        let env_type = detect_runtime_env(py).unwrap();
        let rust_frames: Vec<_Scene> = frames.iter().map(|frame| frame.inner.clone()).collect();

        match env_type {
            RuntimeEnv::Colab | RuntimeEnv::Jupyter => {
                setup_wasm_if_needed(py);
                let wasm_viewer = WasmViewer::initiate_viewer_and_play(
                    py,
                    rust_frames,
                    (interval * 1000.0) as u64,
                    loops,
                    width,
                    height,
                    smooth,
                );

                Ok(Viewer {
                    environment: env_type,
                    wasm_viewer: Some(wasm_viewer),
                    native_gui_viewer: None,
                    first_update: false,
                })
            }

            RuntimeEnv::PlainScript | RuntimeEnv::IPythonTerminal => {
                NativeGuiViewer::play(rust_frames, interval, loops, width, height, smooth);

                Ok(Viewer {
                    environment: env_type,
                    wasm_viewer: None,
                    native_gui_viewer: None,
                    first_update: false,
                })
            }
            _ => Err(PyErr::new::<PyRuntimeError, _>(
                "Error: Invalid runtime environment",
            )),
        }
    }

    pub fn update(&mut self, scene: &Scene, py: Python) {
        let env_type = self.environment;
        match env_type {
            RuntimeEnv::Colab | RuntimeEnv::Jupyter => {
                if self.first_update {
                    print_to_notebook(
                        c_str!(
                            r###"print("\033[33m⚠️ Note: When running in Jupyter or Colab, animation updates may be limited by the notebook's output capacity, which can cause incomplete or delayed rendering.\033[0m")"###
                        ),
                        py,
                    );
                    self.first_update = false;
                }
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
                print_to_notebook(
                    c_str!(
                        r###"print("\033[33m⚠️ Image saving in Jupyter/Colab is not yet fully supported.\033[0m")"###
                    ),
                    py,
                );
                panic!("Error saving image. Saving images from Jupyter/Colab is not yet supported.")
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
    m.add_class::<PyProtein>()?;
    m.add_function(wrap_pyfunction!(parse_sdf, m)?)?;
    m.add_function(wrap_pyfunction!(parse_mmcif, m)?)?;
    Ok(())
}
