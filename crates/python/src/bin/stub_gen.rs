use pyo3_stub_gen::Result;
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    let stub = cosmol_viewer::stub_info()?;
    stub.generate()?;

    let pyi_path = Path::new("./cosmol_viewer.pyi");

    let mut text = fs::read_to_string(pyi_path)?;

    let future_line = "from __future__ import annotations\n";
    if !text.contains(future_line) {
        text = format!("{}{}", future_line, text);
    }

    fs::write(pyi_path, text)?;

    Ok(())
}
