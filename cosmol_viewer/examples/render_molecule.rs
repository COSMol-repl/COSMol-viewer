use std::path::Path;

use cosmol_viewer::utils::VisualShape;
use cosmol_viewer::{Scene, Viewer, shapes::Molecules};
use cosmol_viewer::parser::sdf::{ParserOptions, parse_sdf};

fn main() {
    let sdf_string = std::fs::read_to_string("./examples/example.sdf").unwrap();
    let opts = ParserOptions {
        keep_h: true,
        multimodel: true,
        onemol: false,
    };
    let mol_data = parse_sdf(&sdf_string, &opts);

    let mol = Molecules::new(mol_data).centered();

    let mut scene = Scene::new();

    scene.scale(0.1);

    scene.add_shape(mol, Some("mol"));

    let viewer = Viewer::render(&scene);

    let img = viewer.take_screenshot();

    img.save(Path::new("screenshot.png")).unwrap();

    use std::io::{self, Write};
    println!("Press Enter to exit...");
    let _ = io::stdout().flush();
    let _ = io::stdin().read_line(&mut String::new());
}
