from cosmol_viewer import Scene, Sphere, CosmolViewer

scene = Scene.create_viewer()
scene.add_spheres(Sphere([0.0, 0.3, 0.0], 0.6).with_color([0.0, 1.0, 0.0]).clickable(True))
scene.add_spheres(Sphere([0.0, 0.0, 0.0], 0.7).with_color([0.0, 0.0, 1.0]).clickable(True))

CosmolViewer.render(scene)