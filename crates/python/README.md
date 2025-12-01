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

A high-performance molecular viewer for `Python` and `Rust`, backed by `Rust`.
Supports both static rendering and smooth animation playback — including inside Jupyter notebooks.


A compact, high-performance renderer for molecular and scientific shapes with two usage patterns:

- **Static rendering + update** — push individual scene updates from your application or simulation.
- **Play (recommended for demonstrations & smooth playback)** — precompute frames and hand the sequence to the viewer to play back with optional interpolation (`smooth`).

---

## Quick concepts

- **Scene**: container for shapes (molecules, spheres, lines, etc.).
- **Viewer.render(scene, ...)**: create a static viewer bound to a canvas (native or notebook). Good for static visualization.
- **viewer.update(scene)**: push incremental changes (real-time / streaming use-cases).
- **Viewer.play(frames, interval, loops, width, height, smooth)**: *recommended* for precomputed animations and demonstrations. The viewer takes care of playback timing and looping.

**Why prefer `play` for demos?**
- Single call API (hand off responsibility to the viewer).
- Built-in timing & loop control.
- Optional `smooth` interpolation between frames for visually pleasing playback even when input frame rate is low.

**Why keep `update`?**
- `update` is ideal for real-time simulations, MD runs, or streaming data where frames are not precomputed. It provides strict fidelity (no interpolation) and minimal latency.

---

## Installation

```sh
pip install cosmol-viewer
```

---

## Quick Start
See examples in [Google Colab](https://colab.research.google.com/drive/1Sw72QWjQh_sbbY43jGyBOfF1AQCycmIx?usp=sharing).
### 1. Static molecular rendering

```python
from cosmol_viewer import Scene, Viewer, parse_sdf, Molecules
    
mol_data  = parse_sdf(open("molecule.sdf", "r", encoding="utf-8").read())

mol = Molecules(mol_data).centered()

scene = Scene()
scene.add_shape(mol, "mol")

viewer = Viewer.render(scene, width=600, height=400)

print("Press Any Key to exit...", end='', flush=True)
_ = input()  # Keep the viewer open until you decide to close
```

### 2. Animation playback with `Viewer.play`

```python
from cosmol_viewer import Scene, Viewer, parse_sdf, Molecules
import time

interval = 0.033   # ~30 FPS

frames = []

for i in range(1, 10):
    with open(f"frames/frame_{i}.sdf", "r") as f:
        sdf = f.read()
        mol = Molecules(parse_sdf(sdf)).centered()

    scene = Scene()
    scene.add_shape(mol, "mol")
    frames.append(scene)

Viewer.play(frames, interval=interval, loops=1, width=600, height=400, smooth=True)
```

## Documentation

For API reference and advanced usage, please see the [latest documentation](https://cosmol-repl.github.io/COSMol-viewer).
