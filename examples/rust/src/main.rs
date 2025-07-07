fn main() {}

#[cfg(test)]
mod tests {
    use cosmol_viewer::{parser::sdf::{parse_sdf, ParserOptions}, scene::Scene, Viewer};
    use cosmol_viewer_core::shapes::molecules::Molecules;

    #[test]
    fn render_molecule() {
        let sdf_string = std::fs::read_to_string("../../example.sdf").unwrap();
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

        let _ = Viewer::render(&scene);
    }
}
