#!/usr/bin/env bash
#
# Builds and checks the documentation this repository ships:
#
#   1. `cargo doc` for the Rust API of both crates, with warnings denied
#   2. the `# Errors` / `# Panics` conventions (docs/check_doc_sections.py)
#   3. the diagrams embedded in the design documents (scripts/gen_diagrams.py)
#
# Usage:  docs/build_docs.sh          (from the repository root, or anywhere)
#         make docs                   (same thing)
#
# The tutorial book and the Python API reference are built elsewhere: the book is part of the
# `biodivine` workspace, and the Python package (with its API reference) is in `biodivine-py`.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
[ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"
PYTHON="${PYTHON:-python3}"

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

step "1. cargo doc (Rust API, warnings denied)"
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace
report $?

step "2. documentation sections (# Errors / # Panics)"
"$PYTHON" docs/check_doc_sections.py --self-test
report $?
"$PYTHON" docs/check_doc_sections.py
report $?

step "3. diagrams embedded in the design documents"
"$PYTHON" scripts/gen_diagrams.py --check
report $?

printf '\n=== summary\n'
if [ "$fail" -eq 0 ]; then
    echo "all documentation built and checked"
    echo "  rustdoc            target/doc/biodivine_lib_xml_dom/index.html"
else
    echo "at least one step failed"
fi
exit "$fail"
