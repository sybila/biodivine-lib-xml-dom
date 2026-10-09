#!/usr/bin/env bash
#
# Runs every verification gate of this repository and prints each command with its exit code.
#
#     scripts/verify.sh          # from the repository root, or anywhere
#     make verify                # same thing
#
# The transcript in `docs/design/VERIFICATION.md` is generated from a run of this script, so the
# document and the artefact cannot disagree: re-run it and you get the same evidence.
#
# Scope notes (see `docs/design/VERIFICATION.md` for why):
#   * `--all-features` is applied to the *core* crate only. The binding crate's `extension-module`
#     feature is deliberately enabled only by maturin for wheel builds; turning it on for `cargo
#     clippy` would check the configuration that is explicitly not the in-process-test one.
#   * clippy runs with `-D warnings`, so the "0 warnings" claim is a gate rather than a snapshot.
#   * the MSRV run uses the oldest toolchain the project declares (see `[workspace.package]`).
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
[ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"

VENV="${VENV:-$ROOT/.venv}"
PYTHON="$VENV/bin/python"
MSRV="1.88.0"
Pinned="1.95.0"

failures=0
run() {
    printf '\n$ %s\n' "$*"
    "$@"
    local status=$?
    printf -- '-> exit %s\n' "$status"
    if [ "$status" -ne 0 ]; then
        failures=$((failures + 1))
        printf '!!! FAILED: %s\n' "$*"
    fi
    return 0
}

printf '=== environment\n'
rustc --version
cargo --version
for toolchain in "$MSRV" "$Pinned"; do
    rustc "+$toolchain" --version 2>/dev/null || echo "toolchain $toolchain is not installed"
done
"$PYTHON" -V
"$VENV/bin/maturin" --version 2>/dev/null || "$PYTHON" -m maturin --version
"$PYTHON" -m pytest --version 2>/dev/null
"$PYTHON" -c 'import sphinx; print("sphinx", sphinx.__version__)' 2>/dev/null

# --- Rust: format, lint, test -----------------------------------------------------------------
run cargo fmt --check
run cargo clippy --workspace --all-targets -- -D warnings
run cargo clippy -p biodivine-lib-xml-dom --all-targets --all-features -- -D warnings
run cargo test --workspace
run cargo test --workspace --release

# --- MSRV and the CI-pinned toolchain ----------------------------------------------------------
if rustc "+$MSRV" --version >/dev/null 2>&1; then
    run cargo "+$MSRV" test --workspace
else
    printf '\n!!! toolchain %s is not installed: cannot verify the declared MSRV\n' "$MSRV"
    failures=$((failures + 1))
fi
if rustc "+$Pinned" --version >/dev/null 2>&1; then
    run cargo "+$Pinned" test --workspace
else
    printf '\n!!! toolchain %s is not installed: cannot verify the CI-pinned toolchain\n' "$Pinned"
    failures=$((failures + 1))
fi

# --- examples (the ones the book includes) -----------------------------------------------------
run cargo build --examples
for example in $(ls examples/book_*.rs examples/tour.rs | sed 's|examples/||; s|\.rs||'); do
    run cargo run --quiet --example "$example"
done

# --- documentation ----------------------------------------------------------------------------
run env RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace
run "$PYTHON" docs/check_doc_sections.py --self-test
run "$PYTHON" docs/check_doc_sections.py
run "$PYTHON" docs/check_book.py
run bash docs/build_docs.sh
run "$PYTHON" docs/check_book.py --built

# --- Python -----------------------------------------------------------------------------------
if "$PYTHON" -c 'import biodivine_lib_xml_dom' 2>/dev/null; then
    run "$PYTHON" -m pytest biodivine-lib-xml-dom-py-sys/tests-python
else
    printf '\n!!! the Python package is not importable: build the extension with `make python-extension`\n'
    failures=$((failures + 1))
fi

printf '\n=== summary\n'
if [ "$failures" -eq 0 ]; then
    echo "all gates passed"
else
    echo "$failures gate(s) failed"
fi
exit "$failures"
