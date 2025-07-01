use std::thread;

use cosmol_viewer::{utils::VisualShape, Viewer};
use cosmol_viewer_core::{scene::Scene, shapes::sphere::{Sphere, UpdateSphere}};
// scene.save_as_png("output.png").expect("Failed to save scene as PNG");

// fn main() {
//     let mut scene = Scene::create_viewer();

//     let sphere = Sphere::new([0.0, 0.3, 0.0], 0.6)
//         .with_color([0.0, 1.0, 0.0])
//         .clickable(true);
//     scene.add_sphere(sphere, Some("green"));

//     let sphere = Sphere::new([0.0, 0.5, 0.0], 0.7)
//         .with_color([0.0, 0.0, 1.0])
//         .clickable(true);

//     scene.add_sphere(sphere, Some("blue"));

//     let viewer = Viewer::render(&scene);
    
//     thread::sleep(std::time::Duration::from_secs(1));
    
//     scene.update_sphere("blue", |s| {
//         s.set_center([1.0, 0.0, 0.0]);
//     });
    
//     viewer.update(&scene);

//     thread::sleep(std::time::Duration::from_secs(1));


//     scene.update_sphere("blue", |s| {
//         s.set_radius(0.5);
//     });
    
//     viewer.update(&scene);

//     thread::sleep(std::time::Duration::from_secs(1));

//     scene.delete_sphere("green");
    
//     viewer.update(&scene);
    
//     thread::sleep(std::time::Duration::from_secs(1));
    
//     scene.delete_sphere("blue");
//     let sphere = Sphere::new([0.0, 0.0, 0.7], 0.7)
//         .with_color([1.0, 0.0, 0.0])
//         .clickable(true);

//     scene.add_sphere(sphere, Some("red"));
    
//     viewer.update(&scene);
// }

fn main() {
    let mut scene = Scene::new();

    let sphere_a = Sphere::new([-1.0, 0.0, 0.0], 0.4)
        .with_color([1.0, 0.0, 0.0]) // 红色
        .clickable(true);
    scene.add_shape(sphere_a, Some("red"));

    let sphere_b = Sphere::new([1.0, 0.0, 0.0], 0.4)
        .with_color([0.0, 0.0, 1.0]) // 蓝色
        .clickable(true);
    scene.add_shape(sphere_b, Some("blue"));

    let viewer = Viewer::render(&scene);

    let mut t = 0.0f32;
    let dt = 0.05;
    loop {
        let x = 1.0 * f32::cos(t); // -1 ~ 1

        let r = 0.5 + 0.5 * f32::cos(t); // 红通道
        let b = 0.5 + 0.5 * f32::sin(t); // 蓝通道
        let g = 1.0 - (r + b) / 2.0;     // 绿色反向调整一点（可调）

        scene.update_sphere("red", |s| {
            s.set_center([-x, 0.0, 0.0]);
            s.set_color([r, g, b, 1.0]);
        });

        scene.update_sphere("blue", |s| {
            s.set_center([x, 0.0, 0.0]);
            s.set_color([b, g, r, 1.0]);
        });

        viewer.update(&scene);
        thread::sleep(std::time::Duration::from_millis(16)); // ~60 FPS
        t += dt;
    }
}

