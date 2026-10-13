#!/usr/bin/env python3
"""Copies the shared diagrams from `docs/assets/` into the documents that show them.

Run from the repository root:

    python3 scripts/gen_diagrams.py            # write the diagrams into the documents
    python3 scripts/gen_diagrams.py --check    # fail if a document is out of date

The diagrams are SVG files under `docs/assets/`. They have to appear *inline* in the documents that
show them, because those documents are rendered twice: as markdown by the documentation book and,
verbatim, as rustdoc by the `documentation` module of the crate. A markdown image reference works in
the book (Sphinx copies the file) but silently becomes a dead image in rustdoc, so the documents
carry the SVG itself and this script keeps the copy in step with the file:

    <!-- diagram: model.svg -->
    <svg ...>...</svg>
    <!-- /diagram -->

The file under `docs/assets/` stays the single source of truth; the script reports a document that
has drifted from it, which is what `--check` is for.
"""

import argparse
import difflib
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
ASSETS = ROOT / "docs" / "assets"

#: document -> the diagrams it shows, in the order they appear in the file.
DOCUMENTS = {
    "docs/design/architecture.md": ["model.svg"],
    "docs/design/bindings.md": ["layers.svg"],
    "docs/book/building-documents.md": ["model.svg"],
    "docs/book/python-usage.md": ["layers.svg"],
}

BEGIN = "<!-- diagram: {name} -->"
END = "<!-- /diagram -->"


def rendered(document: str, diagrams: list[str]) -> str:
    """The document with every diagram block filled in from `docs/assets/`."""
    text = (ROOT / document).read_text(encoding="utf-8")
    for name in diagrams:
        svg = (ASSETS / name).read_text(encoding="utf-8").strip()
        begin, end = BEGIN.format(name=name), END
        if begin not in text:
            sys.exit(f"{document} has no `{begin}` marker")
        pattern = re.compile(re.escape(begin) + r".*?" + re.escape(end), re.S)
        text = pattern.sub(f"{begin}\n{svg}\n{end}", text)
    return text


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="fail instead of writing when out of date")
    arguments = parser.parse_args()

    failures = 0
    for document, diagrams in DOCUMENTS.items():
        path = ROOT / document
        current = path.read_text(encoding="utf-8")
        generated = rendered(document, diagrams)
        if current == generated:
            print(f"{document}: up to date ({', '.join(diagrams)})")
            continue
        if arguments.check:
            failures += 1
            print(
                f"{document} is out of date: run scripts/gen_diagrams.py",
                file=sys.stderr,
            )
            for line in list(
                difflib.unified_diff(
                    current.splitlines(), generated.splitlines(), "on disk", "generated", lineterm=""
                )
            )[:20]:
                print(line, file=sys.stderr)
            continue
        path.write_text(generated, encoding="utf-8")
        print(f"{document}: written ({', '.join(diagrams)})")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
