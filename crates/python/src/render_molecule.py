from cosmol_viewer import Scene, Viewer, parse_sdf, Molecules

mol_data  = parse_sdf(open("./examples/example.sdf", "r", encoding="utf-8").read())

mol = Molecules(mol_data).centered()

scene = Scene()

scene.scale(1.0)

scene.add_shape(mol, "molecule")

viewer = Viewer.render(scene, width=800, height=500)

viewer.save_image("screenshot.png")

print("Press Any Key to exit...", end='', flush=True)
_ = input()
