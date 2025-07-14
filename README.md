# COSMol-viewer

Molecular visualization tools by rust

# Usage

## python

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
use cosmol_viewer::{Scene, Viewer, Molecules, parse_sdf, ParserOptions};

fn main() {
    // === Step 1: Load and render a molecule ===
    let sdf_string = std::fs::read_to_string("molecule.sdf").unwrap();

    let mol = Molecules::new(parse_sdf(&sdf_string, &ParserOptions::default()))
        .centered();

    let mut scene = Scene::new();
    scene.scale(0.1);
    scene.add_shape(mol, Some("mol"));

    let mut viewer = Viewer::render(&scene);

    // === Step 2: Update the same molecule dynamically ===
    for i in 1..10 {
        let path = format!("frames/frame_{}.sdf", i);
        let sdf = std::fs::read_to_string(path).unwrap();

        let updated_mol = Molecules::new(parse_sdf(&sdf, &ParserOptions::default()))
            .centered();

        scene.update_shape("mol", updated_mol);
        viewer.update(&scene);

        std::thread::sleep(std::time::Duration::from_millis(33)); // ~30 FPS
    }
}
```