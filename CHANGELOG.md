# Changelog

<!-- release-header:start -->
**COSMol-viewer** is a molecular visualization library for Rust, Python, and the web.

[Source repository](https://github.com/cosmol-studio/COSMol-viewer) ·
[Documentation](https://cosmol-studio.github.io/COSMol-viewer/) ·
[Web tools](https://tools.cosmol.org/) ·
[Rust crate](https://crates.io/crates/cosmol_viewer) ·
[Python package](https://pypi.org/project/cosmol-viewer/).
<!-- release-header:end -->

All notable changes to COSMol-viewer are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

The text between the `release-header` markers is prepended to every GitHub
Release. The release workflow extracts the section matching the pushed tag.

## [0.2.25] - 2026-08-20

### Added

- Added an isolated Mesa softpipe rendering profile for Colab and detailed
  offscreen EGL/GL stage tracing for native crash diagnostics.
- Added a complete Sphinx/Furo documentation site covering installation,
  scenes, camera and lighting controls, molecule representations, protein
  surfaces, geometric shapes, static rendering, interactive viewers,
  animation, Rust usage, and the generated Python API reference.
- Added strict documentation builds and GitHub Pages deployment for changes on
  the main branch.
- Added a dedicated GitHub Release workflow that validates release versions and
  uses the matching changelog section as the release notes.

### Changed

- Single-sample offscreen rendering now explicitly disables OpenGL
  multisampling so framebuffer configuration and GL state remain consistent.
- Updated COSMolKit integration from 0.2.11 to 0.2.12.
- Updated Rust dependencies used for spatial indexing, asset generation, and
  Linux dynamic loading.

### Fixed

- Fixed Google Colab static rendering crashes caused by Mesa llvmpipe even
  after the framebuffer had fallen back to a single sample.
- Fixed the documentation workflow's mismatched source and upload paths and
  prevented pull requests from deploying GitHub Pages.

[0.2.25]: https://github.com/cosmol-studio/COSMol-viewer/compare/v0.2.24...v0.2.25
