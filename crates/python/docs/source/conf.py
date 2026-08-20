import importlib.metadata

project = "COSMol-viewer"
author = "COSMol-viewer Contributors"

try:
    release = importlib.metadata.version("cosmol-viewer")
except importlib.metadata.PackageNotFoundError:
    release = "development"

extensions = [
    "sphinx.ext.autodoc",
    "sphinx.ext.autosummary",
    "sphinx.ext.napoleon",
    "sphinx.ext.viewcode",
]

autosummary_generate = True
autodoc_typehints = "signature"
autoclass_content = "both"

templates_path = ["_templates"]
exclude_patterns = []

html_theme = "furo"
html_title = f"COSMol-viewer {release}"
html_theme_options = {
    "source_repository": "https://github.com/cosmol-studio/COSMol-viewer/",
    "source_branch": "main",
    "source_directory": "crates/python/docs/source/",
}
