import math
import time

from cosmol_viewer import Scene, Sphere, Viewer

# Initialize the scene
scene = Scene()
ids = ["a", "b", "c", "d", "e", "f"]

# Add multiple spheres (6 in total)
for id in ids:
    sphere = Sphere([0.0, 0.0, 0.0], 0.4).color([1.0, 1.0, 1.0])
    scene.add_shape_with_id(id, sphere)

scene.set_scale(2.0)
viewer = Viewer.render(scene, width=800.0, height=500.0)

# Animation main loop
t = 0.0
while True:
    for i, id in enumerate(ids):
        phase = i * math.pi / 3.0
        theta = t + phase

        # Trajectory: elliptical motion
        x = 1.5 * math.cos(theta)
        y = 0.8 * math.sin(theta)
        z = 0.5 * math.sin(theta * 2.0)

        # Radius: pulsating
        radius = 0.3 + 0.15 * math.sin(theta * 1.5)

        # Dynamic RGB color
        r = 0.5 + 0.5 * math.sin(theta)
        g = 0.5 + 0.5 * math.cos(theta)
        b = 1.0 - r

        updated = Sphere([x, y, z], radius).color([r, g, b])
        scene.replace_shape(id, updated)

    viewer.update(scene)
    time.sleep(0.02)
    t += 0.02
