use cosviewer_core::{self, CosViewer, Scene, Sphere};
use cosviewer_core::utils::VisualShape;

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

    CosViewer::render(&scene);

    let sphere = Sphere::new([0.0, 0.0, 0.0], 0.7)
        .with_color([0.0, 0.0, 1.0])
        .clickable(true);
    scene.add_spheres(sphere);

    CosViewer::update(&scene);
}
