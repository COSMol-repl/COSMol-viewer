# COSMol-viewer

Molecular visualization tools by rust

# Usage

python:
```python
! pip install cosmol-viewer

from cosmol_viewer import Scene, Sphere, CosmolViewer

scene = Scene.create_viewer()
scene.add_spheres(Sphere([0.0, 0.3, 0.0], 0.6).with_color([0.0, 1.0, 0.0]).clickable(True))
scene.add_spheres(Sphere([0.0, 0.0, 0.0], 0.7).with_color([0.0, 0.0, 1.0]).clickable(True))

CosmolViewer.render(scene)
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