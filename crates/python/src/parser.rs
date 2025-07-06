use cosmol_viewer_core::parser::sdf::MoleculeData;
use cosmol_viewer_core::parser::sdf::parse_sdf as _parse_sdf;
use cosmol_viewer_core::parser::sdf::ParserOptions;
use pyo3::pyclass;
use pyo3::pyfunction;

#[pyclass]
pub struct PyMoleculeData {
    pub inner: MoleculeData,
}

#[pyfunction]
#[pyo3(signature = (sdf, keep_h=true, multimodel=true, onemol=false))]
pub fn parse_sdf(sdf: &str, keep_h: bool, multimodel: bool,onemol: bool) -> PyMoleculeData {
    let mol_data = _parse_sdf(sdf, &ParserOptions {
        keep_h,
        multimodel,
        onemol,
    });

    PyMoleculeData{
        inner: mol_data,
    }
}