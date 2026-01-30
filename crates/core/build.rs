use std::{env, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let svg_path = PathBuf::from("assets/icon.svg");
    let png_path = out_dir.join("icon.png");

    println!("cargo:rerun-if-changed=assets/icon.svg");

    svg_to_png(&svg_path, &png_path, 128, 128);
}

fn svg_to_png(svg: &std::path::Path, png: &std::path::Path, width: u32, height: u32) {
    use std::fs;
    use tiny_skia::{Pixmap, Transform};
    use usvg::{Options, Tree};

    let svg_data = fs::read(svg).expect("Failed to read SVG");

    let opt = Options::default();
    let tree = Tree::from_data(&svg_data, &opt).expect("Failed to parse SVG");

    let mut pixmap = Pixmap::new(width, height).expect("Failed to create pixmap");

    let scale_x = width as f32 / tree.size().width();
    let scale_y = height as f32 / tree.size().height();
    let transform = Transform::from_scale(scale_x, scale_y);

    resvg::render(&tree, transform, &mut pixmap.as_mut());

    pixmap.save_png(png).expect("Failed to save PNG");
}
