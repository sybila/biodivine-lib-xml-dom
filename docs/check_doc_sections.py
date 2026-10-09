#!/usr/bin/env python3
"""Checks the two documentation conventions AGENTS.md requires on every public item.

Run from the repository root:

    python3 docs/check_doc_sections.py

The conventions are:

1. a public function that can fail (its signature mentions ``Result``) documents *when* it fails in
   an ``# Errors`` section;
2. a public function that can panic documents *when* in an ``# Panics`` section. The crate marks the
   ergonomic, panicking twins of its ``_checked`` operations with ``#[track_caller]``, so that
   attribute is what this check looks for;
3. the same two rules apply to the PyO3 bindings in ``biodivine-lib-xml-dom-py-sys``, because their
   doc comments are the docstrings Sphinx renders as the Python API — there a binding that raises
   may use ``# Errors`` or ``Raises:``.

The check is deliberately textual and conservative: it only looks at the doc block immediately
preceding a declaration, and it counts what it inspected, so a refactor that makes it stop finding
anything fails here instead of passing silently.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

#: Files to inspect, and the minimum number of public items each corpus must contain. The floors are
#: a sanity check on the parser, not a target: they are set well below the current counts.
CORPORA = [
    ("the core crate", Path("src"), 60),
    ("the binding crate", Path("biodivine-lib-xml-dom-py-sys/src"), 40),
]

#: Sections that satisfy each rule.
ERRORS = ("# Errors", "# errors", "Raises:")
PANICS = ("# Panics", "# panics")

FUNCTION = re.compile(r"^(?P<indent>\s*)pub (?:const )?fn (?P<name>\w+)(?P<sig>.*)$")


def doc_block(lines: list[str], index: int) -> str:
    """The doc comment immediately above `lines[index]`, plus any attributes in between."""
    block: list[str] = []
    cursor = index - 1
    while cursor >= 0:
        stripped = lines[cursor].lstrip()
        if stripped.startswith("///") or stripped.startswith("#["):
            block.append(stripped)
            cursor -= 1
            continue
        break
    return "\n".join(reversed(block))


def signature(lines: list[str], index: int) -> str:
    """The declaration starting at `lines[index]`, up to the opening brace or the semicolon."""
    collected: list[str] = []
    for line in lines[index:]:
        collected.append(line.strip())
        if line.rstrip().endswith("{") or line.rstrip().endswith(";"):
            break
    return " ".join(collected)


def check(label: str, directory: Path, floor: int) -> list[str]:
    problems: list[str] = []
    inspected = 0
    for path in sorted(directory.rglob("*.rs")):
        lines = path.read_text(encoding="utf-8").split("\n")
        for index, line in enumerate(lines):
            match = FUNCTION.match(line)
            if not match:
                continue
            name = match.group("name")
            if name.startswith("__") and name not in ("__init__",):
                # Dunder methods: their doc comment is a Python docstring, and none of them returns
                # a `Result`, so neither rule can apply. They are still counted below.
                pass
            block = doc_block(lines, index)
            decl = signature(lines, index)
            inspected += 1

            if "Result<" in decl and not any(section in block for section in ERRORS):
                problems.append(
                    f"{path}:{index + 1}: `{name}` returns a `Result` but has no `# Errors` section"
                )
            if "#[track_caller]" in block and not any(section in block for section in PANICS):
                problems.append(
                    f"{path}:{index + 1}: `{name}` is a panicking twin (`#[track_caller]`) but has "
                    f"no `# Panics` section"
                )
    if inspected < floor:
        problems.append(
            f"{label}: inspected only {inspected} public functions, expected at least {floor} — "
            f"the checker probably stopped understanding the source layout"
        )
    print(f"{label}: inspected {inspected} public functions in {directory}")
    return problems


def self_test() -> list[str]:
    """Checks the checker: the floor must fire, and a missing section must be reported.

    Runs on a temporary corpus, so it is safe to call from a gate. Kept next to the checker rather
    than in the Python test suite because it exercises a Rust-source parser.
    """
    import tempfile

    problems: list[str] = []
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        (root / "good.rs").write_text(
            "/// Does nothing.\n"
            "///\n"
            "/// # Errors\n"
            "///\n"
            "/// Never.\n"
            "pub fn good() -> Result<(), ()> { Ok(()) }\n"
        )
        (root / "bad.rs").write_text(
            "/// Fails.\n"
            "pub fn bad() -> Result<(), ()> { Err(()) }\n"
            "/// Panics.\n"
            "#[track_caller]\n"
            "pub fn panicky() {}\n"
        )
        # The problems reported for the bad corpus are the *evidence* that the checker works, so
        # they are inspected rather than collected.
        reported = check("self-test corpus", root, 100_000)
        if not any("bad.rs" in problem for problem in reported):
            problems.append("self-test: a missing `# Errors` section was not reported")
        if not any("panicky" in problem for problem in reported):
            problems.append("self-test: a panicking twin without `# Panics` was not reported")
        if not any("inspected only" in problem for problem in reported):
            problems.append("self-test: the sanity floor did not fire")

        # And the positive direction: the well-documented corpus alone must pass.
        (root / "bad.rs").unlink()
        (root / "good.rs").unlink()
        (root / "ok.rs").write_text(
            "/// Does nothing.\n"
            "///\n"
            "/// # Errors\n"
            "///\n"
            "/// Never.\n"
            "pub fn good() -> Result<(), ()> { Ok(()) }\n"
        )
        if check("self-test corpus", root, 1):
            problems.append("self-test: a well-documented corpus was reported as broken")
    if problems:
        return [f"the self-test failed: {problem}" for problem in problems]
    print("self-test: the checker reports missing sections and enforces its floor")
    return []


def main() -> int:
    if "--self-test" in sys.argv:
        problems = self_test()
        for problem in problems:
            print(f"  - {problem}", file=sys.stderr)
        return 1 if problems else 0

    problems: list[str] = []
    for label, directory, floor in CORPORA:
        if not directory.is_dir():
            problems.append(f"{label}: {directory} does not exist")
            continue
        problems.extend(check(label, directory, floor))

    if problems:
        print(f"\n{len(problems)} documentation problem(s):", file=sys.stderr)
        for problem in problems:
            print(f"  - {problem}", file=sys.stderr)
        return 1
    print("all public items follow the `# Errors` / `# Panics` conventions")
    return 0


if __name__ == "__main__":
    sys.exit(main())
