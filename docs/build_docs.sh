#!/usr/bin/env bash
#
# Builds and checks every piece of documentation this repository ships:
#
#   1. `cargo doc` for the Rust API of both crates, with warnings denied
#   2. the `# Errors` / `# Panics` conventions (docs/check_doc_sections.py)
#   3. the book's structure and language pairs (docs/check_book.py)
#   4. the Python API reference (Sphinx autodoc over the pure-Python package)
#   5. the documentation book (Sphinx + MyST + sphinx-design)
#   6. the built book again, now checking chapter titles and the tab markup
#
# Usage:  docs/build_docs.sh          (from the repository root, or anywhere)
#         make docs                   (same thing)
#
# The Python API reference imports the package, so the native extension has to exist; this script
# builds it if it is missing rather than documenting a stale or absent extension.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

VENV="${VENV:-$ROOT/.venv}"
PYTHON="$VENV/bin/python"
PY_SYS="$ROOT/biodivine-lib-xml-dom-py-sys"

fail=0
step() { printf '\n=== %s\n' "$1"; }
report() {
    local status=$1
    if [ "$status" -ne 0 ]; then
        fail=1
        printf '!!! failed (exit %s)\n' "$status"
    fi
    return 0
}

step "tool versions"
if [ -x "$PYTHON" ]; then
    "$PYTHON" -c 'import sys, sphinx, sphinx_design, myst_parser; print("python", sys.version.split()[0]); print("sphinx", sphinx.__version__)'
    "$PYTHON" -m pytest --version 2>/dev/null || true
else
    echo "no virtualenv at $VENV (create it with: python3 -m venv .venv && .venv/bin/pip install maturin pytest sphinx sphinx-design myst-parser)"
    fail=1
fi
command -v cargo >/dev/null || source "$HOME/.cargo/env"
cargo --version || fail=1
rustc --version || fail=1
"$PYTHON" -m maturin --version 2>/dev/null || "$VENV/bin/maturin" --version 2>/dev/null || true

step "1. cargo doc (Rust API, warnings denied)"
if command -v cargo >/dev/null 2>&1 || [ -x "$HOME/.cargo/bin/cargo" ]; then
    export PATH="$HOME/.cargo/bin:$PATH"
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace
    report $?
    echo "rustdoc: $(find target/doc -name 'index.html' | wc -l) index pages under target/doc"
else
    echo "cargo is not installed"
    fail=1
fi

step "2. documentation sections (# Errors / # Panics)"
"$PYTHON" docs/check_doc_sections.py --self-test
report $?
"$PYTHON" docs/check_doc_sections.py
report $?

step "3. book sources (structure and language pairs)"
"$PYTHON" docs/check_book.py
report $?

step "4. native extension (needed by Sphinx autodoc)"
if "$PYTHON" -c 'import biodivine_lib_xml_dom' 2>/dev/null; then
    echo "biodivine_lib_xml_dom is importable"
else
    echo "the package is not importable yet; building the extension with maturin"
    ( cd "$PY_SYS" && VIRTUAL_ENV="$VENV" "$VENV/bin/maturin" develop --release )
    status=$?
    report $status
    if [ "$status" -ne 0 ]; then
        echo "cannot document a package that does not import; install maturin into $VENV and retry"
        exit 1
    fi
fi

step "5. Python API reference (Sphinx autodoc)"
rm -rf "$PY_SYS/docs/_build"
VIRTUAL_ENV="$VENV" "$PYTHON" -m sphinx -b html "$PY_SYS/docs" "$PY_SYS/docs/_build"
report $?
echo "python api docs: $(find "$PY_SYS/docs/_build" -name '*.html' | wc -l) HTML files"

step "6. the book"
rm -rf docs/book/_build
VIRTUAL_ENV="$VENV" "$PYTHON" -m sphinx -b html docs/book docs/book/_build
report $?
echo "book: $(find docs/book/_build -name '*.html' | wc -l) HTML files"

step "7. built book (chapter titles and language tabs)"
"$PYTHON" docs/check_book.py --built
report $?

printf '\n=== summary\n'
if [ "$fail" -eq 0 ]; then
    echo "all documentation built and checked"
    echo "  rustdoc            target/doc/biodivine_lib_xml_dom/index.html"
    echo "  python api docs    biodivine-lib-xml-dom-py-sys/docs/_build/index.html"
    echo "  book               docs/book/_build/index.html"
else
    echo "at least one step failed"
fi
exit "$fail"
