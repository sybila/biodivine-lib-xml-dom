#!/usr/bin/env bash
#
# Runs every verification gate of this repository and prints each command with its exit code.
#
#     scripts/verify.sh          # from the repository root, or anywhere
#     make verify                # same thing
#
# Scope notes:
#   * The Python package and the documentation book live elsewhere (`biodivine-py` and the
#     `biodivine` workspace); this script checks the two crates of this repository.
#   * clippy runs with `-D warnings`. Both profiles are linted: a debug-only import or an unreachable arm can be dead code in one
#     profile and live code in the other, so the dev gate alone would not notice it.
#   * the MSRV run uses the oldest toolchain the project declares (see `[workspace.package]`).
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
[ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"

PYTHON="${PYTHON:-python3}"
MSRV="1.88.0"
PINNED="1.95.0"

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
for toolchain in "$MSRV" "$PINNED"; do
    rustc "+$toolchain" --version 2>/dev/null || echo "toolchain $toolchain is not installed"
done
"$PYTHON" -V

# --- Rust: format, lint, test -----------------------------------------------------------------
run cargo fmt --check
run cargo clippy --workspace --all-targets -- -D warnings
run cargo clippy --workspace --all-targets --release -- -D warnings
run cargo test --workspace
run cargo test --workspace --release

# --- MSRV and the CI-pinned toolchain ----------------------------------------------------------
if rustc "+$MSRV" --version >/dev/null 2>&1; then
    run cargo "+$MSRV" test --workspace
else
    printf '\n!!! toolchain %s is not installed: cannot verify the declared MSRV\n' "$MSRV"
    failures=$((failures + 1))
fi
if rustc "+$PINNED" --version >/dev/null 2>&1; then
    run cargo "+$PINNED" test --workspace
else
    printf '\n!!! toolchain %s is not installed: cannot verify the CI-pinned toolchain\n' "$PINNED"
    failures=$((failures + 1))
fi

# --- Miri (undefined behaviour in the data structures) ----------------------------------------
# Miri needs a nightly toolchain with the `miri` component; when it is not installed the run is
# reported as a failure rather than silently skipped. A pinned toolchain that has Miri is
# preferred, nightly is the fallback.
MIRI_TOOLCHAIN=""
for toolchain in "$PINNED" nightly; do
    if cargo "+$toolchain" miri --version >/dev/null 2>&1; then
        MIRI_TOOLCHAIN="$toolchain"
        break
    fi
done
if [ -n "$MIRI_TOOLCHAIN" ]; then
    printf '\n=== Miri (toolchain %s)\n' "$MIRI_TOOLCHAIN"
    run env MIRIFLAGS="-Zmiri-disable-isolation" cargo "+$MIRI_TOOLCHAIN" miri test --lib
    # The two-thread garbage-collection test cannot run under Miri: parking_lot aborts in Miri's
    # futex shim. Everything else in that file is single-threaded and does run.
    run cargo "+$MIRI_TOOLCHAIN" miri test --test gc -- --skip handles_released_by_another_thread
else
    printf '\n!!! miri is not installed for any toolchain: the undefined-behaviour runs are SKIPPED\n'
    printf '    install it with: rustup toolchain install nightly --component miri,rust-src\n'
    failures=$((failures + 1))
fi

# --- examples ---------------------------------------------------------------------------------
run cargo build --examples
for example in $(ls examples/*.rs | sed 's|examples/||; s|\.rs||'); do
    run cargo run --quiet --example "$example"
done

# --- documentation ----------------------------------------------------------------------------
# Each crate is documented on its own as well as in the workspace: a workspace build unifies the
# features of shared dependencies, which can hide a link that only resolves with another crate's
# features (docs.rs builds the core crate on its own).
run env RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p biodivine-lib-xml-dom
run env RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace
run env RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace --document-private-items
run "$PYTHON" scripts/gen_rule_set.py --check
run "$PYTHON" scripts/gen_rule_coverage.py --check
run "$PYTHON" scripts/gen_diagrams.py --check
run "$PYTHON" docs/check_doc_sections.py --self-test
run "$PYTHON" docs/check_doc_sections.py

printf '\n=== summary\n'
if [ "$failures" -eq 0 ]; then
    echo "all gates passed"
else
    echo "$failures gate(s) failed"
fi
exit "$failures"
