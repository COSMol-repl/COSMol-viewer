use std::path::PathBuf;

fn main() {
    let svg_path = PathBuf::from("assets/icon.svg");
    let png_path = PathBuf::from("assets/icon.png");

    println!("cargo:rerun-if-changed=assets/icon.svg");

    svg_to_png(&svg_path, &png_path, 128, 128);
}

fn svg_to_png(svg: &std::path::Path, png: &std::path::Path, width: u32, height: u32) {
    use std::fs;
    use tiny_skia::{Pixmap, Transform};
    use usvg::{Options, Tree};

    // 1. 读 SVG
    let svg_data = fs::read(svg).expect("Failed to read SVG");

    let opt = Options::default();
    let tree = Tree::from_data(&svg_data, &opt).expect("Failed to parse SVG");

    // 2. 创建“拥有内存”的 Pixmap
    let mut pixmap = Pixmap::new(width, height).expect("Failed to create pixmap");

    // 3. 缩放
    let scale_x = width as f32 / tree.size().width();
    let scale_y = height as f32 / tree.size().height();
    let transform = Transform::from_scale(scale_x, scale_y);

    // 4. 渲染
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // 5. 保存
    pixmap.save_png(png).expect("Failed to save PNG");
}
