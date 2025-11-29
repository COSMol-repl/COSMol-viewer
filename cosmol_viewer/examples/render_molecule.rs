use std::path::Path;

use cosmol_viewer::parser::sdf::{ParserOptions, parse_sdf};
use cosmol_viewer::{Scene, Viewer, shapes::Molecules};

fn main() {
    let sdf_string = std::fs::read_to_string("./examples/example.sdf").unwrap();
    // let sdf_string = include_str!("../examples/example.sdf");
    let opts = ParserOptions {
        keep_h: true,
        multimodel: true,
        onemol: false,
    };
    let mol_data = parse_sdf(&sdf_string, &opts);

    let mol = Molecules::new(mol_data).centered();

    let a = mol.atoms.clone()[61];

    println!(
        "{:?}",
        serde_json::from_str::<Molecules>(
            &serde_json::to_string(&mol).expect("json serialize failed")
        )
        .unwrap()
        .atoms[61]
            - a
    );

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
