# COSMol-viewer

<div align="center">
  <a href="https://crates.io/crates/cosmol_viewer">
    <img src="https://img.shields.io/crates/v/cosmol_viewer.svg" alt="crates.io Latest Release"/>
  </a>
  <a href="https://pypi.org/project/cosmol_viewer/">
    <img src="https://img.shields.io/pypi/v/cosmol_viewer.svg" alt="PyPi Latest Release"/>
  </a>
  <a href="https://cosmol-repl.github.io/COSMol-viewer">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg" alt="Documentation Status"/>
  </a>
</div>

**COSMol-viewer** is a high-performance molecular visualization library, written in **Rust** and powered by **WebGPU**, designed for seamless integration into **Python** workflows.  

- ⚡ **High-speed rendering** — GPU-accelerated performance at native speed  
- 🧬 **Flexible input** — Load structures from `.sdf`, `.pdb`, or dynamically generated coordinates  
- 📓 **Notebook-ready** — Fully compatible with Jupyter and Google Colab, ideal for teaching, research, and interactive demos  
- 🔁 **Dynamic visualization** — Update molecular structures on-the-fly or play smooth preloaded animations  
- 🎨 **Customizable** — Fine-grained control of rendering styles, camera, and scene parameters  

---

## Installation

```sh
pip install cosmol-viewer
```

---

## Quick Start

```python
from cosmol_viewer import Scene, Viewer, parse_sdf, Molecules

# === Step 1: Load and render a molecule ===
with open("molecule.sdf", "r") as f:
    sdf = f.read()
    mol = Molecules(parse_sdf(sdf)).centered()

scene = Scene()
scene.scale(0.1)
scene.add_shape(mol, "mol")

viewer = Viewer.render(scene, width=600, height=400)  # Launch viewer

print("Press Any Key to exit...", end='', flush=True)
_ = input()  # Keep the viewer open until you decide to close
```

---

## Animation Modes

COSMol-viewer supports **two complementary animation workflows**, depending on whether you prefer **real-time updates** or **preloaded playback**.

### 1. Real-Time Updates (Frame-by-Frame Streaming)

Update the molecule directly inside an existing scene:

```python
import time
from cosmol_viewer import Scene, Viewer, parse_sdf, Molecules

scene = Scene()
scene.scale(0.1)

# Initial load
with open("frames/frame_1.sdf", "r") as f:
    sdf = f.read()
    mol = Molecules(parse_sdf(sdf)).centered()
scene.add_shape(mol, "mol")

viewer = Viewer.render(scene, width=600, height=400)

# Update in real time
for i in range(2, 10):
    with open(f"frames/frame_{i}.sdf", "r") as f:
        sdf = f.read()
        updated_mol = Molecules(parse_sdf(sdf)).centered()

    scene.update_shape("mol", updated_mol)
    viewer.update(scene)

    time.sleep(0.033)  # ~30 FPS

print("Press Any Key to exit...", end='', flush=True)
_ = input()
```

**Use cases:**  
- Visualizing the *progress* of a simulation step-by-step  
- Interactive experiments or streaming scenarios where frames are not known in advance  

**Trade-offs:**  
- ✅ Low memory usage — no need to preload frames  
- ⚠️ Playback smoothness depends on computation / I/O speed → may stutter if frame generation is slow  

---

### 2. Preloaded Playback (One-Shot Animation) (Start from 0.1.3)

Load all frames into memory first, then play them back smoothly:

```python
from cosmol_viewer import Scene, Viewer, parse_sdf, Molecules

frames = []
interval = 0.033  # ~30 FPS

# Preload all frames
for i in range(1, 10):
    with open(f"frames/frame_{i}.sdf", "r") as f:
        sdf = f.read()
        mol = Molecules(parse_sdf(sdf)).centered()

    scene = Scene()
    scene.scale(0.1)
    scene.add_shape(mol, "mol")
    frames.append(scene)

# Playback once
Viewer.play(frames, interval=interval, loops=1, width=600, height=400)

print("Press Any Key to exit...", end='', flush=True)
_ = input()
```

**Use cases:**  
- Smooth, stable playback for presentations or teaching  
- Demonstrating precomputed trajectories (e.g., molecular dynamics snapshots)  

**Trade-offs:**  
- ✅ Very smooth playback, independent of computation speed  
- ⚠️ Requires preloading all frames → higher memory usage  
- ⚠️ Longer initial load time for large trajectories  

---

## Choosing the Right Mode

- ✅ Use **real-time updates** if your frames are generated on-the-fly or memory is limited  
- ✅ Use **preloaded playback** if you want guaranteed smooth animations and can preload your trajectory  

---

## Exiting the Viewer

> **Important:** The viewer is bound to the Python process.  
> When your script finishes, the rendering window will close automatically.

To keep the visualization alive until you are ready to exit, always add:

```python
print("Press Any Key to exit...", end='', flush=True)
_ = input()
```

This ensures:  
- The window stays open for inspection  
- The user decides when to end visualization  
- Prevents premature termination at the end of the script  

---

## Documentation

For API reference and advanced usage, please see the [latest documentation](https://cosmol-repl.github.io/COSMol-viewer).  
