#!/usr/bin/env bash
#
# Runs a single reproduction probe from this directory against the repository working tree.
#
#   usage: run_probe.sh <probe-file>
#
# Probe file kinds (by filename prefix):
#   it_*    integration test   -> copied to tests/__probe.rs, run with `cargo test --release --test __probe`
#   unit_<module>_*            -> crate-internal test that needs private items of `src/<module>.rs`:
#                                 copied to src/<module>/__probe.rs, `#[cfg(test)] mod __probe;`
#                                 appended to src/<module>.rs, run with `cargo test --release --lib`,
#                                 then both files are restored. The probe body must start with
#                                 `use super::*;`.
#   cf_*    compile-fail probe -> copied to tests/__probe.rs, built with `cargo test --no-run --test __probe`
#   sh_*    shell probe        -> executed directly with bash
#
# The repository working tree is restored before the script exits. `git` is never used to revert,
# so uncommitted work is safe.
#
# Exit code: 0 if the probe behaved as its own expectations dictate, non-zero otherwise. Probes whose
# purpose is to DEMONSTRATE a defect are expected to report a failure; that failure IS the evidence.
set -u

PROBE="${1:?usage: run_probe.sh <probe-file>}"
HERE="$(cd "$(dirname "${PROBE}")" && pwd)"
REPO="$(cd "${HERE}/../../../.." && pwd)"
cd "${REPO}"

# shellcheck disable=SC1090
[ -f "${HOME}/.cargo/env" ] && source "${HOME}/.cargo/env"

BASE="$(basename "${PROBE}")"
KIND="${BASE%%_*}"
REST="${BASE#*_}"
MODULE=""
if [ "${KIND}" = "unit" ]; then
    MODULE="${REST%%_*}"
    REST="${REST#*_}"
fi
BACKUP_LIB=""
cleanup() {
    rm -f "${REPO}/tests/__probe.rs"
    if [ -n "${MODULE}" ]; then
        rm -f "${REPO}/src/${MODULE}/__probe.rs"
        rmdir "${REPO}/src/${MODULE}" 2>/dev/null || true
    fi
    if [ -n "${BACKUP_LIB}" ] && [ -f "${BACKUP_LIB}" ]; then
        mv -f "${BACKUP_LIB}" "${REPO}/src/${MODULE}.rs"
    fi
}
trap cleanup EXIT

echo "==================================================================="
echo "PROBE        : ${BASE}"
echo "KIND         : ${KIND}"
echo "REPO         : ${REPO}"
echo "COMMIT       : $(git rev-parse --short HEAD) $(git log -1 --format=%s)"
echo "RUSTC        : $(rustc --version)"
echo "CARGO        : $(cargo --version)"
echo "DATE (UTC)   : $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "==================================================================="

rc=0
case "${KIND}" in
it)
    echo "\$ cargo test --release --test __probe -- --nocapture --test-threads=1"
    echo
    cp "${PROBE}" "${REPO}/tests/__probe.rs"
    cargo test --release --test __probe -- --nocapture --test-threads=1
    rc=$?
    ;;
unit)
    echo "\$ cargo test --release --lib __probe -- --nocapture --test-threads=1"
    echo
    BACKUP_LIB="$(mktemp)"
    cp "${REPO}/src/${MODULE}.rs" "${BACKUP_LIB}"
    mkdir -p "${REPO}/src/${MODULE}"
    cp "${PROBE}" "${REPO}/src/${MODULE}/__probe.rs"
    printf '\n#[cfg(test)]\nmod __probe;\n' >> "${REPO}/src/${MODULE}.rs"
    cargo test --release --lib __probe -- --nocapture --test-threads=1
    rc=$?
    ;;
cf)
    echo "\$ cargo test --release --no-run --test __probe   (expected: compile error)"
    echo
    cp "${PROBE}" "${REPO}/tests/__probe.rs"
    cargo test --release --no-run --test __probe
    rc=$?
    ;;
sh)
    echo "\$ bash ${BASE}"
    echo
    bash "${PROBE}"
    rc=$?
    ;;
*)
    echo "unknown probe kind '${KIND}' (expected one of it_/unit_/cf_/sh_)" >&2
    rc=2
    ;;
esac

echo
echo "-------------------------------------------------------------------"
echo "PROBE_EXIT=${rc}"
echo "-------------------------------------------------------------------"
exit "${rc}"
