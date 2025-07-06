# COSMol-viewer

Molecular visualization tools by rust

# Usage

python:
```python
! pip install cosmol-viewer==0.1.1.dev4

from cosmol_viewer import Scene, Viewer, parse_sdf, Molecules

# === Step 1: Load and render a molecule ===
with open("molecule.sdf", "r") as f:
    sdf = f.read()
    mol = Molecules(parse_sdf(sdf)).centered()

scene = Scene.create_viewer()
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

rust:
```rust
use cosmol_viewer::{viewer, Scene, Sphere, utils::VisualShape};

fn main() {
    let mut scene = Scene::create_viewer();

    let sphere = Sphere::new([0.0, 0.3, 0.0], 0.6)
        .with_color([0.0, 1.0, 0.0])
        .clickable(true);
    scene.add_spheres(sphere);

    let sphere = Sphere::new([0.0, 0.0, 0.0], 0.7)
        .with_color([0.0, 0.0, 1.0])
        .clickable(true);
    scene.add_spheres(sphere);

    viewer::render(&scene);
}
```