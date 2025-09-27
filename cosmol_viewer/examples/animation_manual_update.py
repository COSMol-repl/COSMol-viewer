from cosmol_viewer import Scene, Sphere, Viewer
import time
import math

# 初始化场景
scene = Scene()
ids = ["a", "b", "c", "d", "e", "f"]

# 添加多个球体（6个）
for id in ids:
    sphere = Sphere([0.0, 0.0, 0.0], 0.4).color([1.0, 1.0, 1.0])
    scene.add_shape(sphere, id)

scene.scale(0.2)
viewer = Viewer.render(scene, width=800.0 ,height=500.0)

# 动画主循环
t = 0.0
while True:
    for i, id in enumerate(ids):
        phase = i * math.pi / 3.0
        theta = t + phase

        # 轨迹：椭圆运动
        x = 1.5 * math.cos(theta)
        y = 0.8 * math.sin(theta)
        z = 0.5 * math.sin(theta * 2.0)

        # 半径：脉动
        radius = 0.3 + 0.15 * math.sin(theta * 1.5)

        # 动态 RGB 颜色
        r = 0.5 + 0.5 * math.sin(theta)
        g = 0.5 + 0.5 * math.cos(theta)
        b = 1.0 - r

        updated = Sphere([x, y, z], radius).color([r, g, b])
        scene.update_shape(id, updated)

    viewer.update(scene)
    time.sleep(0.01)
    t += 0.02