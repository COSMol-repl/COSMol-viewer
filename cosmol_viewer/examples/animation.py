import math

from cosmol_viewer import Animation, Scene, Sphere, Viewer

ids = ["a", "b", "c", "d", "e", "f"]
interval = 0.05
num_frames = int(10.0 / interval)

animation = Animation(interval, -1, True)

for frame_idx in range(num_frames):
    t = frame_idx * interval  # 时间直接用 interval 累积
    scene = Scene()
    scene.set_scale(2.0)

    for i, id in enumerate(ids):
        phase = i * math.pi / 3.0
        theta = t + phase

        # 椭圆轨迹
        x = 1.5 * math.cos(theta)
        y = 0.8 * math.sin(theta)
        z = 0.5 * math.sin(theta * 2.0)

        # 半径脉动
        radius = 0.3 + 0.15 * math.sin(theta * 1.5)

        # 动态颜色
        r = 0.5 + 0.5 * math.sin(theta)
        g = 0.5 + 0.5 * math.cos(theta)
        b = 1.0 - r

        sphere = Sphere([x, y, z], radius).color([r, g, b])
        scene.add_shape_with_id(id, sphere)

    animation.add_frame(scene)

# 一次性提交：间隔0.02秒
Viewer.play(animation, width=800.0, height=500.0)

print("Press Any Key to exit...", end="", flush=True)
_ = input()
