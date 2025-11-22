use cosmol_viewer::parser::protein::parse_mmcif;
use cosmol_viewer::{Scene, Viewer, shapes::Protein};
// use std::path::Path;

fn main() {
    // let mmcif_string = &std::fs::read_to_string("./examples/2AMD.cif").unwrap();
    let mmcif_string = include_str!("../examples/2AMD.cif");
    let mmcif_data = parse_mmcif(mmcif_string, None);

    let prot = Protein::new(mmcif_data).centered();

    let mut scene = Scene::new();
    scene.scale(0.1);
    scene.use_black_background();
    scene.add_shape(prot, Some("prot"));

    let viewer = Viewer::render(&scene, 800.0, 500.0);

    // let img = viewer.take_screenshot();

    // img.save(Path::new("screenshot.png")).unwrap();

    use std::io::{self, Write};
    println!("Press Enter to exit...");
    let _ = io::stdout().flush();
    let _ = io::stdin().read_line(&mut String::new());
}
