"""Sphinx configuration for the Biodivine XML DOM documentation book.

Build with::

    .venv/bin/python -m sphinx -b html docs/book docs/book/_build

The book is written in MyST (Markdown), uses the Shibuya theme, and uses `sphinx-design` for the
language switch: every example is a tab set with a Rust and a Python tab that share sync groups, so
choosing a language switches every example on the page.
"""

project = "Biodivine XML DOM"
copyright = "2026, Sybila"
author = "Sybila"
# Read from the crate manifest (via `[workspace.package]`), so the book and the code agree. The
# path is resolved relative to this file, so the build does not depend on the working directory.
from pathlib import Path

_manifest = (Path(__file__).resolve().parents[2] / "Cargo.toml").read_text(encoding="utf-8")
version = [
    line.split("=", 1)[1].strip().strip('"')
    for line in _manifest.splitlines()
    if line.startswith("version")
][0]
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

html_theme = "shibuya"
html_title = "Biodivine XML DOM"
# Shibuya's options: https://shibuya.lepture.com/. The accent colour and the automatic colour mode
# are presentation choices; the repository links are what a reader of a chapter usually wants next;
# the AI/ads links the theme can add are switched off, because a library book should not advertise
# third-party services.
html_theme_options = {
    "accent_color": "violet",
    "color_mode": "auto",
    "show_ai_links": False,
    "github_url": "https://github.com/sybila/biodivine-lib-xml-dom",
    "nav_links": [
        {"title": "Rust API", "url": "https://docs.rs/biodivine-lib-xml-dom"},
        {"title": "Python package", "url": "python-usage.html"},
    ],
}
exclude_patterns = ["_build", "examples/**"]
