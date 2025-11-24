use cosmol_viewer::parser::mmcif::parse_mmcif;
use cosmol_viewer::{Scene, Viewer, shapes::Protein};

fn main() {
    // let mmcif_string = &std::fs::read_to_string("./examples/2AMD.cif").unwrap();
    let mmcif_string = include_str!("../examples/2AMD.cif");
    let mmcif_data = parse_mmcif(mmcif_string, None);

    let prot = Protein::new(mmcif_data);

    let mut scene = Scene::new();
    scene.use_black_background();
    scene.scale(0.2);
    scene.recenter(prot.get_center());
    scene.add_shape(prot, Some("prot"));

    Viewer::render(&scene, 800.0, 500.0);

    println!("Press Enter to exit...");
    use std::io::{self, Write};
    let _ = io::stdout().flush();
    let _ = io::stdin().read_line(&mut String::new());
}
