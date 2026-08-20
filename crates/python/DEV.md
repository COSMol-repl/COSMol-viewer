## Generate `.pyi` Stubs (dev / abi3-py310)

From repo root:

```bash
cargo run -p cosmol_viewer_python --no-default-features --features dev-stub --bin stub_gen
```

Generated file:

```text
./crates/python/cosmol_viewer.pyi
```

## Build/Install Extension with maturin

### Dev install (editable)

From repo root:

```bash
maturin develop --uv --manifest-path crates/python/Cargo.toml
```

## Build Python Documentation

From the repository root:

```bash
uv venv .venv
uv pip install --python .venv/bin/python "maturin>=1.7,<2.0" "sphinx>=8,<10" "furo>=2024.8.6"
.venv/bin/maturin develop --manifest-path crates/python/Cargo.toml
.venv/bin/python -m sphinx -W --keep-going -b html crates/python/docs/source crates/python/docs/_build/html
```

Generated HTML:

```text
crates/python/docs/_build/html/index.html
```

On Windows, run the equivalent commands after ``conda activate COS`` and use
``maturin`` and ``python`` from that environment.
