# COSMol-viewer: Molecular visualization tools by rust

<div align="center">
  <a href="https://crates.io/crates/cosmol_viewer">
    <img src="https://img.shields.io/crates/v/cosmol_viewer.svg" alt="crates.io Latest Release"/>
  </a>
  <a href="https://pypi.org/project/cosmol_viewer/">
    <img src="https://img.shields.io/pypi/v/cosmol_viewer.svg" alt="PyPi Latest Release"/>
  </a>
  <a href="https://cosmol-repl.github.io/COSMol-viewer">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg" alt="Documentation Status"/>
  </a>
</div>

# Usage

## python
See examples in [Google Colab](https://colab.research.google.com/drive/1Sw72QWjQh_sbbY43jGyBOfF1AQCycmIx?usp=sharing).

Install with `pip install cosmol-viewer`

```python
from cosmol_viewer import Scene, Viewer, parse_sdf, Molecules

# === Step 1: Load and render a molecule ===
with open("molecule.sdf", "r") as f:
    sdf = f.read()
    mol = Molecules(parse_sdf(sdf)).centered()

scene = Scene()
scene.scale(0.1)
scene.add_shape(mol, "mol")

viewer = Viewer.render(scene)  # Launch the viewer

# === Step 2: Update the same molecule dynamically ===
# Useful for molecular dynamics simulations or other animations
# where the molecule changes over time.
import time

for i in range(1, 10):  # Simulate multiple frames
    with open(f"frames/frame_{i}.sdf", "r") as f:
        sdf = f.read()
        updated_mol = Molecules(parse_sdf(sdf)).centered()

    scene.update_shape("mol", updated_mol)
    viewer.update(scene)

    time.sleep(0.033)  # ~30 FPS
```

## Rust

Install with `cargo add cosmol_viewer`

```rust
use cosmol_viewer::{Scene, Viewer, shapes::Molecules};
use cosmol_viewer::parser::sdf::{ParserOptions, parse_sdf};

fn main() {
    // === Step 1: Load and render a molecule ===
    let sdf_string = std::fs::read_to_string("molecule.sdf").unwrap();

    let parser_options = ParserOptions {
        multimodel: true,
        ..Default::default()
    };

    let mol = Molecules::new(parse_sdf(&sdf_string, &parser_options))
        .centered();

    let mut scene = Scene::new();
    scene.scale(0.1);
    scene.add_shape(mol, Some("mol"));

    let viewer = Viewer::render(&scene);

    // === Step 2: Update the same molecule dynamically ===
    // Useful for molecular dynamics simulations or other animations
    // where the molecule changes over time.
    for i in 1..10 {
        let path = format!("frames/frame_{}.sdf", i);
        let sdf = std::fs::read_to_string(path).unwrap();

        let updated_mol = Molecules::new(parse_sdf(&sdf, &parser_options))
            .centered();

        scene.update_shape("mol", updated_mol);
        viewer.update(&scene);

        std::thread::sleep(std::time::Duration::from_millis(33)); // ~30 FPS
    }

    use std::io::{self, Write};
    println!("Press Enter to exit...");
    let _ = io::stdout().flush();
    let _ = io::stdin().read_line(&mut String::new());
}
```

more examples can be found in the [examples](https://github.com/COSMol-repl/COSMol-viewer/tree/main/cosmol_viewer/examples) folder:
```bash
cd cosmol_viewer
cargo run --example animation
```