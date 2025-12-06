from cosmol_viewer import Protein, Scene, Viewer, parse_mmcif

mmcif_data = parse_mmcif(open("./examples/2AMD.cif", "r", encoding="utf-8").read())

prot = Protein(mmcif_data).color([0.2, 0.45, 0.6])

scene = Scene()
# scene.use_black_background()
scene.scale(0.2)
scene.recenter(prot.get_center())
scene.add_shape(prot, "prot")

viewer = Viewer.render(scene, width=800, height=500)

print("Press Any Key to exit...", end="", flush=True)
_ = input()
