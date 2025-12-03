from cosmol_viewer import Scene, Sphere, Viewer
import math

ids = ["a", "b", "c", "d", "e", "f"]
interval = 0.05        # 每帧间隔 (秒)
duration = 10.0        # 动画时长 (秒)
num_frames = int(duration / interval)

frames = []

for frame_idx in range(num_frames):
    t = frame_idx * interval  # 时间直接用 interval 累积
    scene = Scene()
    scene.scale(2.0)

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
        scene.add_shape(sphere, id)

    frames.append(scene)

# 一次性提交：间隔0.02秒
Viewer.play(frames, interval=interval, loops=-1, width=800.0 ,height=500.0, smooth=True)

print("Press Any Key to exit...", end='', flush=True)
_ = input()
