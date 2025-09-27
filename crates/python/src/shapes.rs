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

    pub fn set_radius(mut slf: PyRefMut<'_, Self>, radius: f32) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.set_radius(radius);
        slf
    }

    pub fn set_center(mut slf: PyRefMut<'_, Self>, center: [f32; 3]) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.set_center(center);
        slf
    }

    pub fn color(mut slf: PyRefMut<'_, Self>, color: [f32; 3]) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.color(color);
        slf
    }

    pub fn color_rgba(mut slf: PyRefMut<'_, Self>, color: [f32; 4]) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.color_rgba(color);
        slf
    }

    pub fn opacity(mut slf: PyRefMut<'_, Self>, opacity: f32) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.opacity(opacity);
        slf
    }
}

#[pyclass(name = "Stick")]
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

    pub fn color(mut slf: PyRefMut<'_, Self>, color: [f32; 3]) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.color(color);
        slf
    }

    pub fn set_thickness(mut slf: PyRefMut<'_, Self>, thickness: f32) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.set_thickness(thickness);
        slf
    }

    pub fn set_start(mut slf: PyRefMut<'_, Self>, start: [f32; 3]) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.set_start(start);
        slf
    }

    pub fn set_end(mut slf: PyRefMut<'_, Self>, end: [f32; 3]) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.set_end(end);
        slf
    }

    pub fn color_rgba(mut slf: PyRefMut<'_, Self>, color: [f32; 4]) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.color_rgba(color);
        slf
    }

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
    #[new]
    pub fn new(molecule_data: &PyMoleculeData) -> Self {
        Self {
            inner: Molecules::new(molecule_data.inner.clone()),
        }
    }

    pub fn centered(mut slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().centered();
        slf
    }

    pub fn color(mut slf: PyRefMut<'_, Self>, color: [f32; 3]) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().color(color);
        slf
    }

    pub fn color_rgba(mut slf: PyRefMut<'_, Self>, color: [f32; 4]) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().color_rgba(color);
        slf
    }


    pub fn opacity(mut slf: PyRefMut<'_, Self>, opacity: f32) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().opacity(opacity);
        slf
    }
}
