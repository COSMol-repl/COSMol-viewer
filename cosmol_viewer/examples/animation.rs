use cosmol_viewer::shapes::Sphere;
use cosmol_viewer::utils::VisualShape;
use cosmol_viewer::{Viewer, Scene};
use std::f32::consts::PI;

fn main() {
    // 球体 ID
    let ids = ["a", "b", "c", "d", "e", "f"];

    // 动画参数
    let interval: f32 = 0.02; // 每帧间隔 (秒)
    let duration: f32 = 10.0; // 总时长 (秒)
    let num_frames = (duration / interval) as usize;

    // 存储所有帧
    let mut frames: Vec<Scene> = Vec::with_capacity(num_frames);

    for frame_idx in 0..num_frames {
        let t = frame_idx as f32 * interval;

        let mut scene = Scene::new();
        scene.scale(0.2);

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
            scene.add_shape(sphere, Some(id));
        }

        frames.push(scene);
    }

    // 一次性提交所有帧，由 Viewer 控制播放
    Viewer::play(frames, interval, 1, 800.0, 500.0, true);
}
