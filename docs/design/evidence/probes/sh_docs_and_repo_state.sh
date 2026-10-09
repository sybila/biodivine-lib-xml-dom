#!/usr/bin/env bash
# PROBE `sh_docs_and_repo_state` — documentation book and repository hygiene gaps.
#
# Reproduce:
#   docs/design/evidence/probes/run_probe.sh docs/design/evidence/probes/sh_docs_and_repo_state.sh
#
# Expected (gaps present): no docs book / Sphinx configuration, no `rewrite` branch, no
# `rust-toolchain.toml`, no MSRV declaration in `Cargo.toml`, and a leftover demo binary
# plus an empty `output.xml` in the crate root.

set -u
cd "$(dirname "$0")/../../../.."

report() { echo "--- $1"; }

report "documentation book / Sphinx configuration"
find . -maxdepth 4 \( -name 'conf.py' -o -name 'mkdocs.yml' -o -name 'book.toml' -o -name 'index.rst' \) \
    -not -path './target/*' -not -path './.git/*' | sed 's/^/  /'
echo "  (above should be empty; requirement (7) asks for a docs book)"

report "rustdoc warning configuration / missing-docs lint"
grep -rn "missing_docs\|RUSTDOCFLAGS\|deny(" --include='*.rs' --include='*.toml' --include='*.yml' . 2>/dev/null \
    | grep -v '^./target/' | grep -v '^./docs/' | sed 's/^/  /'
echo "  (above should be empty)"

report "rust-toolchain.toml / MSRV declaration"
ls rust-toolchain.toml rust-toolchain 2>/dev/null | sed 's/^/  /'
grep -n "rust-version" Cargo.toml | sed 's/^/  /'
echo "  Cargo.toml:"
sed 's/^/    /' Cargo.toml
echo "  CI pins (from .github/workflows/build.yml):"
grep -E "rust-version|min-rust-version" .github/workflows/*.yml | sed 's/^/    /'

report "leftover artefacts in the crate root"
ls -la src/main.rs output.xml 2>/dev/null | sed 's/^/  /'
echo "  output.xml size: $(wc -c < output.xml 2>/dev/null) bytes"
echo "  src/main.rs line count: $(wc -l < src/main.rs 2>/dev/null)"

report "git branches"
git branch -a | sed 's/^/  /'
echo '  (requirement (8) asks for a `rewrite` branch; only master/remotes exist)'

report "git working tree"
git status --short | sed 's/^/  /'

report "repository review tooling"
ls -a .coderabbit.yaml .github 2>/dev/null | sed 's/^/  /'
