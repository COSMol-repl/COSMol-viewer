use cosmol_viewer::utils::VisualShape;
use cosmol_viewer::{Scene, Viewer, shapes::Protein};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prot = Protein::from_mmcif(include_str!("../examples/6fi1.cif"))?.color([0.2, 0.45, 0.6]);

    let mut scene = Scene::new();
    // scene.use_black_background();
    scene.set_scale(0.2);
    scene.recenter(prot.get_center());
    scene.add_shape_with_id("prot", prot);

    Viewer::render(&scene, 800.0, 500.0);

    println!("Press Enter to exit...");
    use std::io::{self, Write};
    let _ = io::stdout().flush();
    let _ = io::stdin().read_line(&mut String::new());

    Ok(())
}
