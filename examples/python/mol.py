from cosmol_viewer import Scene, Viewer, parse_sdf, Molecules

with open("example.sdf", "r", encoding="utf-8") as f:
    sdf_string = f.read()

    mol_data  = parse_sdf(sdf_string)

    mol = Molecules(mol_data)

    scene = Scene.create_viewer()
    
    scene.scale(0.1)
    
    scene.add_shape(mol, "molecule")

    viewer = Viewer.render(scene)