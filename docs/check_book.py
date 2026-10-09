#!/usr/bin/env python3
"""Checks the documentation book: structure, language pairs, and the built HTML.

Run from the repository root:

    python3 docs/check_book.py            # source-level checks
    python3 docs/check_book.py --built    # additionally inspects docs/book/_build

Requirement (7) asks for a book in which *every* example is available in both Rust and Python with a
switch between them. That is a structural property, so it is checked rather than promised:

* every ``tab-set`` in the book contains exactly two ``tab-item`` blocks, one labelled Rust and one
  Python, each in its own sync group (so choosing a language in one example switches all of them);
* every example is included with ``literalinclude`` from a real file, the two members point at
  *different* files, and the Rust member is a ``.rs`` file while the Python member is a ``.py`` file;
* every chapter the book promises exists, and the book contains at least one example pair (a
  mis-globbed directory would otherwise make all of the above vacuous);
* with ``--built``, the generated HTML contains the chapters and sphinx-design tab markup.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

BOOK = Path("docs/book")
BUILD = BOOK / "_build"

#: Chapters the book promises (the titles are also searched for in the built HTML).
CHAPTERS = {
    "index.md": "biodivine-lib-xml-dom",
    "getting-started.md": "Getting started",
    "building-documents.md": "Building documents",
    "traversing-and-editing.md": "Traversing and editing",
    "namespaces.md": "Namespaces",
    "parsing-and-serializing.md": "Parsing and serializing",
    "validation.md": "Validating a document",
    "thread-safety.md": "Thread safety",
    "python-usage.md": "Using the Python package",
    "migration.md": "Migrating from 0.1",
    "limitations.md": "Known limitations and gotchas",
    "design-notes.md": "Design notes",
}

TAB_SET = re.compile(r"^(?P<fence>:{3,})\{tab-set\}\s*$")
TAB_ITEM = re.compile(r"^(?P<fence>:{3,})\{tab-item\}\s*(?P<label>.*?)\s*$")
COLON_ONLY = re.compile(r"^(?P<fence>:{3,})\s*$")
SYNC = re.compile(r"^\s*:sync:\s*(\S+)\s*$")
INCLUDE = re.compile(r"literalinclude\}\s+(\S+)")


def parse_tab_sets(lines: list[str]) -> list[list[tuple[str, str, str]]]:
    """Every tab set in `lines`, as a list of `(label, sync_group, included_file)` items.

    The fence lengths disambiguate the nesting: a tab item closes with a shorter colon-only line
    than the tab set that contains it (``:::`` versus ``::::``).
    """
    sets: list[list[tuple[str, str, str]]] = []
    index = 0
    while index < len(lines):
        opening = TAB_SET.match(lines[index])
        if not opening:
            index += 1
            continue
        set_fence = len(opening.group("fence"))
        index += 1
        items: list[tuple[str, str, str]] = []
        while index < len(lines):
            closer = COLON_ONLY.match(lines[index])
            if closer and len(closer.group("fence")) >= set_fence:
                index += 1
                break
            item = TAB_ITEM.match(lines[index])
            if not item:
                index += 1
                continue
            item_fence = len(item.group("fence"))
            label = item.group("label")
            sync = ""
            include = ""
            index += 1
            while index < len(lines):
                closer = COLON_ONLY.match(lines[index])
                if closer and len(closer.group("fence")) < set_fence:
                    index += 1
                    break
                if not sync and (found := SYNC.match(lines[index])):
                    sync = found.group(1)
                if not include and (found := INCLUDE.search(lines[index])):
                    include = found.group(1)
                index += 1
            items.append((label, sync, include))
        sets.append(items)
    return sets


def check_sources() -> list[str]:
    problems: list[str] = []
    pairs = 0
    for chapter, title in CHAPTERS.items():
        path = BOOK / chapter
        if not path.is_file():
            problems.append(f"missing chapter {chapter} (expected to be about `{title}`)")
            continue
        lines = path.read_text(encoding="utf-8").split("\n")
        if title not in "\n".join(lines[:6]):
            problems.append(f"{chapter}: does not start with the title `{title}`")

        for items in parse_tab_sets(lines):
            pairs += 1
            if len(items) != 2:
                problems.append(
                    f"{chapter}: a tab-set has {len(items)} tab(s); every example needs exactly "
                    f"two (Rust and Python)"
                )
                continue
            labels = {label.lower() for label, _, _ in items}
            if labels != {"rust", "python"}:
                problems.append(f"{chapter}: a tab-set is labelled {sorted(labels)}, not Rust/Python")
            syncs = {sync for _, sync, _ in items}
            if syncs != {"rust", "python"}:
                problems.append(
                    f"{chapter}: a tab-set uses sync groups {sorted(syncs)}; both members must be in "
                    f"their own group (`rust` / `python`) so the switch is linked"
                )
            paths = {}
            for label, _, include in items:
                if not include:
                    problems.append(f"{chapter}: the {label} tab has no `literalinclude`")
                    continue
                target = (BOOK / include).resolve()
                if not target.is_file():
                    problems.append(f"{chapter}: the {label} tab includes {include}, which is missing")
                    continue
                paths[label.lower()] = target
            if len(paths) == 2:
                rust, python = paths.get("rust"), paths.get("python")
                if rust == python:
                    problems.append(f"{chapter}: both tabs include the same file ({rust})")
                elif rust is not None and rust.suffix != ".rs":
                    problems.append(f"{chapter}: the Rust tab includes {rust.name}, not a `.rs` file")
                elif python is not None and python.suffix != ".py":
                    problems.append(f"{chapter}: the Python tab includes {python.name}, not a `.py` file")
    if pairs == 0:
        problems.append("the book contains no example pair at all — is the layout right?")
    print(f"book sources: {len(CHAPTERS)} chapters, {pairs} language pairs")
    return problems


def check_built() -> list[str]:
    problems: list[str] = []
    if not BUILD.is_dir():
        return [f"{BUILD} does not exist; build the book before using --built"]
    pages = {path.stem: path for path in BUILD.rglob("*.html")}
    for chapter, title in CHAPTERS.items():
        stem = Path(chapter).stem
        if stem not in pages:
            problems.append(f"the built book has no page for {chapter}")
            continue
        html = pages[stem].read_text(encoding="utf-8", errors="ignore")
        if title not in html:
            problems.append(f"the built page {stem}.html does not contain the title `{title}`")
    # The language switch is sphinx-design's, so the markup has to be there.
    tab_sets = sum(
        page.read_text(encoding="utf-8", errors="ignore").count("sd-tab-set")
        for page in pages.values()
    )
    if tab_sets == 0:
        problems.append("the built book contains no sphinx-design tab sets")
    print(f"built book: {len(pages)} pages, {tab_sets} tab-set occurrences")
    return problems


def main() -> int:
    problems = check_sources()
    if "--built" in sys.argv:
        problems += check_built()
    if problems:
        print(f"\n{len(problems)} problem(s):", file=sys.stderr)
        for problem in problems:
            print(f"  - {problem}", file=sys.stderr)
        return 1
    print("the book is well-formed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
