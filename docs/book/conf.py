"""Sphinx configuration for the `biodivine-lib-xml-dom` documentation book.

Build with::

    .venv/bin/python -m sphinx -b html docs/book docs/book/_build

The book is written in MyST (Markdown) and uses `sphinx-design` for the language switch: every
example is a tab set with a Rust and a Python tab that share sync groups, so choosing a language
switches every example on the page.
"""

project = "biodivine-lib-xml-dom"
copyright = "2026, Sybila"
author = "Sybila"
# Read from the crate manifest (via `[workspace.package]`), so the book and the code agree.
_version = [
    line.split("=", 1)[1].strip().strip('"')
    for line in open("Cargo.toml").read().splitlines()
    if line.startswith("version")
][0]
version = _version
release = version

extensions = [
    "myst_parser",
    "sphinx_design",
    "sphinx.ext.intersphinx",
]

myst_enable_extensions = ["colon_fence", "deflist", "attrs_inline"]
# Heading anchors, so that cross-chapter links like `namespaces.md#no-magic` resolve.
myst_heading_anchors = 3
source_suffix = {".md": "markdown"}
master_doc = "index"

html_theme = "alabaster"
html_title = "biodivine-lib-xml-dom — documentation"
exclude_patterns = ["_build", "examples/**"]
