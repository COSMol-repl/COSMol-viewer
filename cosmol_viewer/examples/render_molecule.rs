use cosmol_viewer::parser::sdf::{ParserOptions, parse_sdf};
use cosmol_viewer::{Scene, Viewer, shapes::Molecules};
use std::path::Path;

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

    scene.add_shape(mol, Some("mol"));

    let viewer = Viewer::render(&scene, 800.0, 500.0);

    let img = viewer.take_screenshot();

    img.save(Path::new("screenshot.png")).unwrap();

    println!("Press Enter to exit...");
    use std::io::{self, Write};
    let _ = io::stdout().flush();
    let _ = io::stdin().read_line(&mut String::new());
}
