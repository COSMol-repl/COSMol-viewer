use std::ffi::CStr;

use base64::Engine as _;
use pyo3::{exceptions::PyTypeError, ffi::c_str, prelude::*};

use crate::{
    parser::parse_sdf,
    shapes::{PyMolecules, PySphere, PyStick},
};
use cosmol_viewer_core::{NativeGuiViewer, scene::Scene as _Scene};
use cosmol_viewer_wasm::{WasmViewer, setup_wasm_if_needed};

mod parser;
mod shapes;

#[pyclass]
/// A 3D scene container for visualizing molecular or geometric shapes.
///
/// This class allows adding, updating, and removing shapes in a 3D scene,
/// as well as modifying scene-level properties like scale and background color.
///
/// Supported shape types:
/// - `Sphere`
/// - `Stick`
/// - `Molecules`
///
/// Shapes can be optionally identified with a string `id`, which allows updates and deletion.
pub struct Scene {
    inner: _Scene,
}

#[pymethods]
impl Scene {
    /// Creates a new empty scene.
    ///
    /// # Example (Python)
    /// ```python
    /// scene = Scene()
    /// ```
    #[new]
    pub fn new() -> Self {
        Self {
            inner: _Scene::new(),
        }
    }

    /// Add a shape to the scene.
    ///
    /// # Arguments
    ///
    /// * `shape` - A shape instance (`PySphere`, `PyStick`, or `PyMolecules`).
    /// * `id` - Optional string ID to associate with the shape.
    ///
    /// If the `id` is provided and a shape with the same ID exists, the new shape will replace it.
    ///
    /// # Example
    /// ```python
    /// scene.add_shape(sphere)
    /// scene.add_shape(stick, id="bond1")
    /// ```
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

    /// Updates an existing shape in the scene by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the shape to update.
    /// * `shape` - New shape object to replace the existing one.
    ///
    /// # Example
    /// ```python
    /// scene.update_shape("atom1", updated_sphere)
    /// ```
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

    /// Removes a shape from the scene by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the shape to remove.
    ///
    /// # Example
    /// ```python
    /// scene.delete_shape("bond1")
    /// ```
    pub fn delete_shape(&mut self, id: &str) {
        self.inner.delete_shape(id);
    }

    /// Set the viewport size of the scene.
    ///
    /// # Arguments
    ///
    /// * `width` - Width of the viewport in pixels.
    /// * `height` - Height of the viewport in pixels.
    ///
    /// # Example
    /// ```python
    /// scene.set_viewport(600, 400)
    /// ```
    pub fn set_viewport(&mut self, width: usize, height: usize) {
        self.inner.set_viewport(width, height);
    }

    /// Sets the global scale factor of the scene.
    ///
    /// This affects the visual size of all shapes uniformly.
    ///
    /// # Arguments
    ///
    /// * `scale` - A positive float scaling factor.
    ///
    /// # Example
    /// ```python
    /// scene.scale(1.5)
    /// ```
    pub fn scale(&mut self, scale: f32) {
        self.inner.scale(scale);
    }

    /// Sets the background color of the scene.
    ///
    /// # Arguments
    ///
    /// * `background_color` - An RGB array of 3 float values between 0.0 and 1.0.
    ///
    /// # Example
    /// ```python
    /// scene.set_background_color([1.0, 1.0, 1.0])  # white background
    /// ```
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
/// A viewer that renders 3D scenes in different runtime environments (e.g., Jupyter, Colab, or native GUI).
///
/// The `Viewer` handles the logic for rendering scenes either through a browser-based WebAssembly canvas
/// or via a native GUI window depending on the execution environment.
///
/// Use `Viewer.render(scene)` to create and display a viewer instance.
///
/// # Examples:
/// ```python
/// from cosmol_viewer import Viewer, Scene, Sphere
/// scene = Scene()
/// scene.add_shape(Sphere(...))
/// viewer = Viewer.render(scene)
/// ```
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
    /// Get the current runtime environment as a string.
    ///
    /// Returns:
    ///     str: One of "Jupyter", "Colab", "PlainScript", or "IPythonTerminal".
    ///
    /// Examples:
    /// ```python
    /// env = Viewer.get_environment()
    /// print(env)  # e.g., "Jupyter"
    /// ```
    #[staticmethod]
    pub fn get_environment(py: Python) -> PyResult<String> {
        let env = detect_runtime_env(py)?;
        Ok(env.to_string())
    }

    #[staticmethod]
    /// Render a 3D scene based on the current environment.
    ///
    /// If running inside Jupyter or Colab, the scene will be displayed inline using WebAssembly.
    /// If running from a script or terminal, a native GUI window is used (if supported).
    ///
    /// Args:
    ///     scene (Scene): The scene to render.
    ///
    /// Returns:
    ///     Viewer: The created viewer instance.
    ///
    /// Examples:
    /// ```python
    /// from cosmol_viewer import Viewer, Scene, Sphere
    ///
    /// scene = Scene()
    /// scene.add_shape(Sphere(center=[0.0, 0.0, 0.0], radius=1.0))
    ///
    /// viewer = Viewer.render(scene)
    /// ```
    pub fn render(scene: &Scene, py: Python) -> Self {
        let env_type = detect_runtime_env(py).unwrap();
        match env_type {
            RuntimeEnv::Colab | RuntimeEnv::Jupyter => {
                print_to_notebook(
                    c_str!(
                        r#"from IPython.display import display, HTML
display(HTML("<div style='color:red;font-weight:bold;'>⚠️ Note: When running in Jupyter or Colab, animation updates may be limited by the notebook's output capacity, which can cause incomplete or delayed rendering.</div>"))"#
                    ),
                    py,
                );
                setup_wasm_if_needed(py);
                let wasm_viewer = WasmViewer::initate_viewer(py, &scene.inner);

                Viewer {
                    environment: env_type,
                    wasm_viewer: Some(wasm_viewer),
                    native_gui_viewer: None,
                }
            }
            RuntimeEnv::PlainScript | RuntimeEnv::IPythonTerminal => Viewer {
                environment: env_type,
                wasm_viewer: None,
                native_gui_viewer: Some(NativeGuiViewer::render(&scene.inner)),
            },
            _ => panic!("Error: Invalid runtime environment"),
        }
    }

    /// Update the viewer with a new scene.
    ///
    /// Works for both Web-based rendering (Jupyter/Colab) and native GUI windows.
    ///
    /// ⚠️ **Note (Jupyter/Colab)**:
    /// When running in notebook environments, animation updates may be limited by
    /// the output rendering capacity of the frontend. This may result in delayed or
    /// incomplete rendering during frequent scene updates.
    ///
    /// Args:
    ///     scene (Scene): The updated scene to apply.
    ///
    /// Examples:
    /// ```python
    /// scene.add_shape(Sphere(center=[1.0, 1.0, 1.0], radius=0.5))
    /// viewer.update(scene)
    /// ```
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

    /// Save the current image to a file.
    ///
    /// Args:
    ///     path (str): The path to save the image to.
    ///
    /// Examples:
    /// ```python
    /// viewer = Viewer.render(scene)
    /// viewer.save_image("image.png")
    /// ```
    pub fn save_image(&self, path: &str, py: Python) {
        let env_type = self.environment;
        match env_type {
            RuntimeEnv::Colab | RuntimeEnv::Jupyter => {
                // let image = self.wasm_viewer.as_ref().unwrap().take_screenshot(py);
                print_to_notebook(
                    c_str!(
                        r#"<div style='color:red;font-weight:bold;'>⚠️ Image saving in Jupyter/Colab is not yet fully supported. This feature is still under development.</div>"))"#
                    ),
                    py,
                );
                panic!("Error saving image. Saving images from Jupyter/Colab is not yet supported. This feature is still under development.")
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
    m.add_class::<PySphere>()?;
    m.add_class::<PyStick>()?;
    m.add_class::<PyMolecules>()?;
    m.add_class::<Viewer>()?;
    m.add_function(wrap_pyfunction!(parse_sdf, m)?)?;
    Ok(())
}
