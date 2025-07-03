use std::thread;

use cosmol_viewer::{parser::sdf::{parse_sdf, ParserOptions}, utils::VisualShape, Viewer};
use cosmol_viewer_core::{
    scene::Scene,
    shapes::{
        molecules::Molecules, sphere::{Sphere, UpdateSphere}, stick::{Stick, UpdateStick}
    },
};
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

// fn main() {
//     let mut scene = Scene::new();

//     let sphere_a = Sphere::new([-1.0, 0.0, 0.0], 0.4)
//         .color([1.0, 0.0, 0.0]) // 红色
//         .clickable(true);
//     scene.add_shape(sphere_a, Some("red"));

//     let sphere_b = Sphere::new([1.0, 0.0, 0.0], 0.4)
//         .color([0.0, 0.0, 1.0]) // 蓝色
//         .clickable(true).opacity(1.0);
//     scene.add_shape(sphere_b, Some("blue"));

//     let stick = Stick::new([-1.0, 0.0, 0.0],[1.0, 0.0, 0.0], 0.1);
//     scene.add_shape(stick, Some("stick"));

//     let viewer = Viewer::render(&scene);

//     let mut t = 0.0f32;
//     let dt = 0.05;
//     loop {
//         let x = 1.0 * f32::cos(t); // -1 ~ 1

//         let r = 0.5 + 0.5 * f32::cos(t); // 红通道
//         let b = 0.5 + 0.5 * f32::sin(t); // 蓝通道
//         let g = 1.0 - (r + b) / 2.0;     // 绿色反向调整一点（可调）

//         scene.update_shape("red", sphere_a.center([-x, 0.0, 0.0]).color([r, g, b]));

//         scene.update_shape("blue", sphere_b.center([x, 0.0, 0.0]).color([b, g, r]));

//         scene.update_shape("stick", stick.start([-x, 0.0, 0.0]).end([x, 0.0, 0.0]));

//         viewer.update(&scene);
//         // thread::sleep(std::time::Duration::from_millis(16)); // ~60 FPS
//         thread::sleep(std::time::Duration::from_millis(100)); // ~60 FPS
//         t += dt;
//     }
// }

fn main() {
    let sdf_string = std::fs::read_to_string("../../example.sdf").unwrap();
    let opts = ParserOptions {
        keep_h: true,
        multimodel: true,
        onemol: false,
    };
    let mol_data  = parse_sdf(&sdf_string, &opts);

    let mol = Molecules::new(mol_data).centered();

    let mut scene = Scene::new();

    scene.scale(0.1);

    scene.add_shape(mol, Some("mol"));

    let _ = Viewer::render(&scene);
}
