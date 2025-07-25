use cosmol_viewer::shapes::Sphere;
use cosmol_viewer::utils::VisualShape;
use cosmol_viewer::{Viewer, Scene};
use std::{f32::consts::PI, thread, time::Duration};

fn main() {
    // 初始化场景
    let mut scene = Scene::new();

    // 添加多个球体（6个）
    let ids = ["a", "b", "c", "d", "e", "f"];
    for id in ids.iter() {
        let sphere = Sphere::new([0.0, 0.0, 0.0], 0.4).color([1.0, 1.0, 1.0]);
        scene.add_shape(sphere, Some(id));
    }

    scene.scale(0.2);

    scene.set_viewport(800, 500);

    let viewer = Viewer::render(&scene);

    // 动画主循环
    let mut t: f32 = 0.0;
    loop {
        for (i, id) in ids.iter().enumerate() {
            let phase = i as f32 * PI / 3.0;
            let theta = t + phase;

            // 轨迹：椭圆运动
            let x = 1.5 * f32::cos(theta);
            let y = 0.8 * f32::sin(theta);
            let z = 0.5 * f32::sin(theta * 2.0);

            // 半径：脉动变化
            let radius = 0.3 + 0.15 * f32::sin(theta * 1.5);

            // 颜色：动态 RGB 渐变
            let r = 0.5 + 0.5 * f32::sin(theta);
            let g = 0.5 + 0.5 * f32::cos(theta);
            let b = 1.0 - r;

            let sphere = Sphere::new([x, y, z], radius).color([r, g, b]);
            scene.update_shape(id, sphere);
        }

        viewer.update(&scene);

        thread::sleep(Duration::from_millis(20));

        t += 0.02;

        if t > 1000.0 {
            t -= 1000.0;
        }
    }
}
