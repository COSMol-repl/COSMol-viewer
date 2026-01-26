use cosmol_viewer::shapes::Sphere;
use cosmol_viewer::utils::VisualShape;
use cosmol_viewer::{Scene, Viewer};
use std::{f32::consts::PI, thread, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the scene
    let mut scene = Scene::new();

    // Add multiple spheres (6 in total)
    let ids = ["a", "b", "c", "d", "e", "f"];
    for id in ids.iter() {
        let sphere = Sphere::new([0.0, 0.0, 0.0], 0.4).color([1.0, 1.0, 1.0]);
        scene.add_shape_with_id(*id, sphere);
    }

    scene.set_scale(2.0);

    let viewer = Viewer::render(&scene, 800.0, 500.0);

    // Animation main loop
    let mut t: f32 = 0.0;
    loop {
        for (i, id) in ids.iter().enumerate() {
            let phase = i as f32 * PI / 3.0;
            let theta = t + phase;

            // Trajectory: elliptical motion
            let x = 1.5 * f32::cos(theta);
            let y = 0.8 * f32::sin(theta);
            let z = 0.5 * f32::sin(theta * 2.0);

            // Radius: pulsating change
            let radius = 0.3 + 0.15 * f32::sin(theta * 1.5);

            // Color: dynamic RGB gradient
            let r = 0.5 + 0.5 * f32::sin(theta);
            let g = 0.5 + 0.5 * f32::cos(theta);
            let b = 1.0 - r;

            let sphere = Sphere::new([x, y, z], radius).color([r, g, b]);
            scene.replace_shape(id, sphere)?;
        }

        viewer.update(&scene);

        thread::sleep(Duration::from_millis(20));

        t += 0.02;

        if t > 1000.0 {
            t -= 1000.0;
        }
    }
}
