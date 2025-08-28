use crate::parser::PyMoleculeData;
use cosmol_viewer_core::{
    shapes::{Molecules, Sphere, Stick},
    utils::VisualShape,
};
use pyo3::{PyRefMut, pyclass, pymethods};

#[pyclass(name = "Sphere")]
/// Sphere(center: [f32; 3], radius: f32)
/// 
/// A sphere in the scene.
/// 
/// # Arguments
/// * `center` - The center of the sphere.
/// * `radius` - The radius of the sphere.
/// 
/// # Examples
/// ```
/// scene = Scene()
/// sphere = Sphere([0.0, 0.0, 0.0], 0.1).color([1.0, 1.0, 1.0])
/// scene.add_shape(sphere, id)
/// viewer = Viewer.render(scene)
/// ```
/// 
#[derive(Clone)]
pub struct PySphere {
    pub inner: Sphere,
}

#[pymethods]
impl PySphere {
    #[new]
    pub fn new(center: [f32; 3], radius: f32) -> Self {
        Self {
            inner: Sphere::new(center, radius),
        }
    }

    /// Set the radius of the sphere.
    ///
    /// # Arguments
    ///
    /// * `radius` - The new radius of the sphere.
    ///
    /// # Returns
    /// `Sphere`: The updated sphere object.
    pub fn set_radius(mut slf: PyRefMut<'_, Self>, radius: f32) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.set_radius(radius);
        slf
    }

    /// Set the color of the sphere.
    ///
    /// # Arguments
    ///
    /// * `color` - The new color of the sphere in RGB format.
    ///
    /// # Returns
    /// `Sphere`: The updated sphere object.
    pub fn color(mut slf: PyRefMut<'_, Self>, color: [f32; 3]) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.color(color);
        slf
    }

    /// Set the color of the sphere with opacity.
    ///
    /// # Arguments
    ///
    /// * `color` - The new color of the sphere in RGBA format.
    ///
    /// # Returns
    /// `Sphere`: The updated sphere object.
    pub fn color_rgba(mut slf: PyRefMut<'_, Self>, color: [f32; 4]) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.color_rgba(color);
        slf
    }

    /// Set the opacity of the sphere.
    ///
    /// # Arguments
    ///
    /// * `opacity` - The new opacity of the sphere.
    ///
    /// # Returns
    /// `Sphere`: The updated sphere object.
    pub fn opacity(mut slf: PyRefMut<'_, Self>, opacity: f32) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.opacity(opacity);
        slf
    }
}

#[pyclass(name = "Stick")]
/// Stick(start: [f32; 3], end: [f32; 3], thickness: f32)
///
/// A cylindrical stick (or capsule) connecting two points in 3D space.
///
/// # Arguments
/// * `start` - The starting point of the stick.
/// * `end` - The ending point of the stick.
/// * `thickness` - The thickness (radius) of the stick.
///
/// # Examples
/// ```python
/// from cosmol_viewer import Scene, Stick, Viewer
///
/// stick = Stick([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 0.05)
/// stick = stick.color([0.5, 0.8, 1.0]).opacity(0.7)
///
/// scene = Scene()
/// scene.add_shape(stick)
/// viewer = Viewer.render(scene)
/// ```
#[derive(Clone)]
pub struct PyStick {
    pub inner: Stick,
}

#[pymethods]
impl PyStick {
    #[new]
    pub fn new(start: [f32; 3], end: [f32; 3], thickness: f32) -> Self {
        Self {
            inner: Stick::new(start, end, thickness),
        }
    }

    /// Set the color of the stick in RGB format.
    ///
    /// # Arguments
    /// * `color` - RGB values as a list of 3 floats.
    ///
    /// # Returns
    /// `Stick`: The updated stick object.
    ///
    /// # Examples
    /// ```python
    /// stick.color([1.0, 0.0, 0.0])
    /// ```
    pub fn color(mut slf: PyRefMut<'_, Self>, color: [f32; 3]) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.color(color);
        slf
    }

    /// Set the thickness of the stick.
    ///
    /// # Arguments
    /// * `thickness` - The thickness (radius) of the stick.
    ///
    /// # Returns
    /// `Stick`: The updated stick object.
    pub fn set_thickness(mut slf: PyRefMut<'_, Self>, thickness: f32) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.set_thickness(thickness);
        slf
    }

    /// Set the start point of the stick.
    ///
    /// # Arguments
    /// * `start` - Set the start position as `[x, y, z]`.
    ///
    /// # Returns
    /// `Stick`: The updated stick object.
    pub fn set_start(mut slf: PyRefMut<'_, Self>, start: [f32; 3]) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.set_start(start);
        slf
    }

    /// Set the end point of the stick.
    ///
    /// # Arguments
    /// * `end` - Set the end position as `[x, y, z]`.
    ///
    /// # Returns
    /// `Stick`: The updated stick object.
    pub fn set_end(mut slf: PyRefMut<'_, Self>, end: [f32; 3]) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.set_end(end);
        slf
    }

    /// Set the color of the stick in RGBA format.
    ///
    /// # Arguments
    /// * `color` - RGBA values as a list of 4 floats.
    ///
    /// # Returns
    /// `Stick`: The updated stick object.
    pub fn color_rgba(mut slf: PyRefMut<'_, Self>, color: [f32; 4]) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.color_rgba(color);
        slf
    }

    /// Set the opacity of the stick.
    ///
    /// # Arguments
    /// * `opacity` - A float between 0.0 (fully transparent) and 1.0 (fully opaque).
    ///
    /// # Returns
    /// `Stick`: The updated stick object.
    pub fn opacity(mut slf: PyRefMut<'_, Self>, opacity: f32) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.opacity(opacity);
        slf
    }
}

#[pyclass(name = "Molecules")]
#[derive(Clone)]
pub struct PyMolecules {
    pub inner: Molecules,
}

#[pymethods]
impl PyMolecules {
    ///
    /// # Arguments
    ///
    /// * `molecule_data` - The molecule data to use.
    /// 
    /// # Returns
    ///
    /// The molecules shape.
    ///
    #[new]
    pub fn new(molecule_data: &PyMoleculeData) -> Self {
        Self {
            inner: Molecules::new(molecule_data.inner.clone()),
        }
    }

    /// Center the molecule.
    ///
    /// # Returns
    ///
    /// The molecule with the centering applied.
    ///
    pub fn centered(mut slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().centered();
        slf
    }

    /// Set the color of the molecule.
    ///
    /// # Arguments
    ///
    /// * `color` - The new color of the molecule in RGB format.
    ///
    /// ⚠️ **Note:** This method will override the original color of the molecule.
    /// Set to None to reset to the original color.
    ///
    /// # Returns
    ///
    /// The molecule with the color applied.
    pub fn color(mut slf: PyRefMut<'_, Self>, color: [f32; 3]) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().color(color);
        slf
    }

    /// Set the color of the molecule with opacity.
    ///
    /// # Arguments
    ///
    /// * `color` - The new color of the molecule in RGBA format.
    ///
    /// ⚠️ **Note:** This method will override the original color of the molecule.
    /// Set to None to reset to the original color.
    ///
    /// # Returns
    ///
    /// The molecule with the color applied.
    pub fn color_rgba(mut slf: PyRefMut<'_, Self>, color: [f32; 4]) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().color_rgba(color);
        slf
    }

    /// Set the opacity of the molecule.
    ///
    /// # Arguments
    ///
    /// * `opacity` - The new opacity of the molecule.
    ///
    /// # Returns
    ///
    /// The molecule with the opacity applied.
    pub fn opacity(mut slf: PyRefMut<'_, Self>, opacity: f32) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().opacity(opacity);
        slf
    }
}
