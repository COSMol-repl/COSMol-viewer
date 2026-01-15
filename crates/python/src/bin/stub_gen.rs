use pyo3_stub_gen::Result;

fn main() -> Result<()> {
    let stub = cosmol_viewer::stub_info()?;
    stub.generate()?;
    Ok(())
}
