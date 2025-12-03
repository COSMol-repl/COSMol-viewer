from cosmol_viewer import Scene, Viewer, parse_sdf, parse_mmcif, Molecules, Protein

mmcif_data  = parse_mmcif(open("./examples/2AMD.cif", "r", encoding="utf-8").read())
prot = Protein(mmcif_data).color([0.2, 0.45, 0.6])

ligand_data  = parse_sdf(open("./examples/2AMD_ligand.sdf", "r", encoding="utf-8").read())
ligand = Molecules(ligand_data)

scene = Scene()
scene.add_shape(prot, "prot")
scene.add_shape(ligand, "ligand")
scene.recenter(ligand.get_center())

viewer = Viewer.render(scene, width=800, height=500)

viewer.save_image("screenshot.png")

print("Press Any Key to exit...", end='', flush=True)
_ = input()
