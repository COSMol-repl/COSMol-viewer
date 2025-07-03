from cosmol_viewer import Scene, Sphere, Viewer

scene = Scene.create_viewer()

green_sphere = Sphere([0.0, 0.3, 0.0], 0.6).color([0.0, 1.0, 0.0])
scene.add_shape(green_sphere, "green")
blue_sphere = Sphere([0.0, 0.0, 0.9], 0.6).color([0.0, 0.0, 1.0])
scene.add_shape(blue_sphere, "blue")

viewer = Viewer.render(scene)

scene.update_shape("blue", blue_sphere.color([1.0, 0.0, 0.0]))

viewer.update(scene)