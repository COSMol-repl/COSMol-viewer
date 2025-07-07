use cosmol_viewer_core::{
    shapes::{molecules::Molecules, sphere::Sphere, stick::Stick},
    utils::VisualShape,
};
use pyo3::{PyRefMut, pyclass, pymethods};

use crate::parser::PyMoleculeData;
// use cosmol_viewer_core::shapes::stick::Stick;

#[pyclass(name = "Sphere")]
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

    pub fn clickable(mut slf: PyRefMut<'_, Self>, val: bool) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clickable(val);
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
    pub fn new(start: [f32; 3], end: [f32; 3], radius: f32) -> Self {
        Self {
            inner: Stick::new(start, end, radius)
        }
    }

    pub fn clickable(mut slf: PyRefMut<'_, Self>, val: bool) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clickable(val);
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
            inner: Molecules::new(molecule_data.inner.clone())
        }
    }

    pub fn centered(mut slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().centered();
        slf
    }
}