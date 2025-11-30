use cosmol_viewer::parser::mmcif::parse_mmcif;
use cosmol_viewer::parser::sdf::ParserOptions;
use cosmol_viewer::parser::sdf::parse_sdf;
use cosmol_viewer::utils::VisualShape;
use cosmol_viewer::{Scene, Viewer, shapes::Molecules, shapes::Protein};

fn main() {
    let mmcif_data = parse_mmcif(include_str!("../examples/2AMD.cif"), None);
    let prot = Protein::new(mmcif_data).color([0.2, 0.45, 0.6]);

    let ligand_data = parse_sdf(
        include_str!("2amd_ligand.sdf"),
        &ParserOptions {
            keep_h: true,
            multimodel: true,
            onemol: false,
        },
    );
    let ligand = Molecules::new(ligand_data);

    let mut scene = Scene::new();
    scene.recenter(ligand.get_center());
    scene.add_shape(prot, Some("prot"));
    scene.add_shape(ligand, Some("ligand"));

    Viewer::render(&scene, 800.0, 500.0);

    println!("Press Enter to exit...");
    use std::io::{self, Write};
    let _ = io::stdout().flush();
    let _ = io::stdin().read_line(&mut String::new());
}
