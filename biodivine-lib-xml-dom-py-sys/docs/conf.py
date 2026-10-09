"""Sphinx configuration for the `biodivine_lib_xml_dom` Python API reference.

Build with::

    ../.venv/bin/python -m sphinx -b html docs docs/_build

The package under test must be importable, which means the native extension has to be built first
(`maturin develop`); `docs/index.md` says so explicitly and the repository's `make docs` does it.
"""

project = "biodivine-lib-xml-dom"
copyright = "2026, Sybila"
author = "Sybila"
version = "0.2.0"
release = version

extensions = [
    "sphinx.ext.autodoc",
    "sphinx.ext.napoleon",
    "sphinx.ext.intersphinx",
    "myst_parser",
    "sphinx_design",
]

autodoc_default_options = {
    "members": True,
    "undoc-members": False,
    "show-inheritance": True,
    "member-order": "alphabetical",
}
autodoc_typehints = "description"
napoleon_google_docstring = True
napoleon_numpy_docstring = False

myst_enable_extensions = ["colon_fence", "deflist"]
source_suffix = {".md": "markdown", ".rst": "restructuredtext"}
master_doc = "index"

html_theme = "alabaster"
html_title = "biodivine-lib-xml-dom Python API"
exclude_patterns = ["_build"]

intersphinx_mapping = {"python": ("https://docs.python.org/3", None)}
