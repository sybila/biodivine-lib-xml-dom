#!/usr/bin/env bash
# PROBE `sh_python_bindings` — there is no PyO3 layer of any kind.
#
# Reproduce:
#   docs/design/evidence/probes/run_probe.sh docs/design/evidence/probes/sh_python_bindings.sh
#
# Expected (gap present): every search below comes back empty and the probe exits non-zero.
# Requirement (6) asks for three layers: the PyO3-free Rust core, a thin
# `biodivine-lib-xml-dom-py-sys` wrapper crate, and a pure-Python `biodivine_lib_xml_dom`
# package. None of them exist.

set -u
cd "$(dirname "$0")/../../../.."

fail=0
report() { echo "--- $1"; }

report "grep -ri pyo3 (rust sources + manifests)"
if grep -ri "pyo3" --include='*.rs' --include='*.toml' . 2>/dev/null | grep -v '^./target/' | grep -v '^./docs/'; then
    echo "  ^^ found PyO3 references"
else
    echo "  (none)"
fi

report "cargo metadata workspace members"
cargo metadata --no-deps --format-version 1 2>/dev/null \
    | python3 -c 'import json,sys; d=json.load(sys.stdin); print(" ", "\n  ".join(p["name"] for p in d["packages"]))'

report "look for a -py-sys crate or python package"
find . -maxdepth 3 \( -name '*py-sys*' -o -name 'pyproject.toml' -o -name 'python' -o -name '*.pyi' \) \
    -not -path './target/*' -not -path './.git/*' | sed 's/^/  /'
echo "  (above should be empty)"

report "python packages installed in this environment matching the library"
python3 -c 'import importlib.util as u; print("  biodivine_lib_xml_dom_sys:", u.find_spec("biodivine_lib_xml_dom_sys")); print("  biodivine_lib_xml_dom:", u.find_spec("biodivine_lib_xml_dom"))'

report "any Python source/test files in the repo"
find . -name '*.py' -not -path './target/*' -not -path './.git/*' | sed 's/^/  /'
echo "  (above should be empty)"

exit "${fail}"
