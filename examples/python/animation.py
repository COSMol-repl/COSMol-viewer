from cosmol_viewer import Scene, Sphere, Viewer
import time
import math

# 初始化
scene = Scene.create_viewer()
green_sphere = Sphere([0.0, 0.3, 0.0], 0.6).with_color([0.0, 1.0, 0.0])
scene.add_shape(green_sphere, "green")

blue_sphere = Sphere([0.0, 0.0, 0.9], 0.6).with_color([0.0, 0.0, 1.0])
scene.add_shape(blue_sphere, "blue")

viewer = Viewer.render(scene)

# 主线程动画循环
t = 0.0
while True:
    z = 0.9 + 0.3 * math.sin(t)
    r = abs(math.sin(t))
    g = abs(math.cos(t))
    b = 1.0 - r

    updated = Sphere([0.0, 0.0, z], 0.6).with_color([r, g, b])
    scene.update_shape("blue", updated)
    viewer.update(scene)

    time.sleep(0.013)  # ≈30fps
    t += 0.1
