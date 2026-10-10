#!/usr/bin/env python3
"""Sweeps the repository for documentation that does not belong in a published project.

Run from the repository root:

    python3 docs/audit_docs.py                 # the source sweep
    python3 docs/audit_docs.py --include-book  # additionally sweep docs/book/**
    python3 docs/audit_docs.py --self-test     # prove that every category can fail

The categories, each of which is one pure function over a piece of text so that the self-test can
exercise it directly:

1. **instruction language** — `requirement (`, `advisor`, `condition C`, `REVIEW D`, `PLAN.md` and
   the like. These refer to the instructions a change was produced under, which a reader of the
   project cannot follow.
2. **rule markdown paths** — a path like `specification/rules/rule.x.y.md` or `rule.x.y.md` outside
   the places that legitimately work with the files themselves (the two generators, the generated
   module list, the rule files, and the checkers that glob them). The *exact* set of permitted
   files is pinned below: a file that starts or stops matching has to be added to or removed from
   the list, so the sweep cannot quietly stop covering something.
2b. **repository document paths in crate documentation** — a `docs/…` path in a doc comment. The
   only permitted occurrences are the `include_str!` directives that pull the documents into the
   documentation module; their exact number is pinned to the number of documents, so a narrative
   reference anywhere else fails.
3. **internal module names in public documentation** — `crate::arena`, `crate::io`, `crate::ref_count`
   and friends name private modules, and the public documentation must not mention them. Checked in
   two ways: in the rendered rustdoc output (excluding the source view, which prints the real file
   paths), and in the doc comments of the files whose documentation is public.
4. **migration and back-compatibility wording** — the library has not been published, so nothing in
   it describes moving from an older version.
5. **ASCII-art diagrams** — box-drawing characters in documentation; diagrams are SVG files under
   `docs/assets/` instead.
6. **path case** — every file a `include_str!` directive names must be tracked under exactly that
   spelling. The filesystems this project is developed on are case-insensitive, so a wrong-case path
   would compile here and fail on a case-sensitive checkout; comparing against `git ls-files` catches
   it.

Every category reports the file, the line and the offending text, and the script exits non-zero if
anything was found. `--self-test` feeds each category a positive and a negative sample and asserts
that it fires only on the positive one, so "zero hits" always means "this check looked and found
nothing", never "this check is broken".
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CRATE = "biodivine-lib-xml-dom"
SYS = "biodivine-lib-xml-dom-py-sys"
DOCS = [ROOT / "target" / "doc" / name for name in ("biodivine_lib_xml_dom", "biodivine_lib_xml_dom_sys")]

#: Directories under `target/doc` that hold pages of *internal* modules (which a public build does
#: not produce at all — their absence is checked) and the design documents, which are about the
#: implementation and are allowed to name it.
DOCS_SKIP = {
    "arena", "document", "documentation", "element", "error", "interner", "io", "namespace",
    "node", "node_id", "qualified_name", "ref_count", "validation",
}

#: Files that may mention a rule markdown path. A file that starts or stops matching must be added
#: to or removed from this list, which is what keeps the allowlist honest.
RULE_PATH_FILES = {
    "scripts/gen_rule_set.py",
    "scripts/gen_rule_coverage.py",
    "docs/check_facts.py",
    "src/xml_spec/rule_set.rs",
    "src/xml_spec/rules.rs",
}

#: Every way a rule file name appears in the sweep: a path (`rule.a.b.md`) or a glob (`rule.*.md`).
RULE_PATH_ANY = re.compile(r"rule\.[^\s\"'`)\]]*\.md")

#: Files whose doc comments name internal modules on purpose because they implement them. Every
#: other file's doc comments are public documentation, and the rendered-prose pass checks the
#: public module `documentation` as well, so an internal name cannot hide there.
PRIVATE_DOC_FILES = {
    "src/arena.rs",
    "src/arena/gc.rs",
    "src/arena/slot.rs",
    "src/arena/snapshot.rs",
    "src/arena/config.rs",
    "src/document/io.rs",
    "src/io.rs",
    "src/io/parse.rs",
    "src/io/write.rs",
    "src/interner.rs",
    "src/ref_count.rs",
}

INSTRUCTION_PATTERNS = [
    (re.compile(r"requirement \("), "a reference to the instructions behind a change"),
    (re.compile(r"\badvisor\b", re.I), "a reference to a review process"),
    (re.compile(r"condition C\d"), "a reference to a review condition"),
    (re.compile(r"REVIEW D\d"), "a reference to an internal review"),
    (re.compile(r"PLAN\.md"), "a reference to a deleted document"),
]

#: A `docs/…` path as it can appear in prose: with or without a trailing file name, with wildcards
#: (`docs/design/*.md`) and with a trailing slash (`docs/book/`).
DOC_PATH = re.compile(r"docs/(?:design|book)?/?[A-Za-z0-9._*-]*")
#: The one place allowed to hold a document path: the include directives of the documentation
#: module. Their number is checked against the number of documents.
DOC_PATH_ALLOWED = re.compile(r'include_str!\("\.\./docs/design/[a-z-]+\.md"\)')

RULE_PATH = re.compile(r"(?:specification/rules/)?rule\.[a-z0-9-]+\.[a-z0-9-]+\.md")
#: The private modules of the crate. Their types are public, but the *paths* are not part of the
#: public API, so documentation must name the type rather than the module it is defined in.
INTERNAL_MODULES = (
    "arena",
    "document",
    "element",
    "error",
    "interner",
    "io",
    "namespace",
    "node",
    "node_id",
    "qualified_name",
    "ref_count",
    "validation",
)
INTERNAL_MODULE = re.compile(r"crate::(?:" + "|".join(INTERNAL_MODULES) + r")\b")
#: The lookbehind keeps public sub-modules out of it: `xml_spec::validation::check_…` is a public
#: path, while a bare `validation::check_…` or `crate::validation` names the private module.
INTERNAL_MODULE_RENDERED = re.compile(
    r"(?<![\w:])(?:crate::)?(?:" + "|".join(INTERNAL_MODULES) + r")::[a-z_]"
)
MIGRATION = re.compile(r"\bmigrat|back[- ]?compat|backwards? compatib", re.I)
ASCII_ART = re.compile(r"[│├└─┌┐┘┴┬┼┤╭╮╰╯]")

violations: list[str] = []


def report(category: str, path: Path, number: int, line: str) -> None:
    relative = path.relative_to(ROOT)
    violations.append(f"{category}: {relative}:{number}: {line.strip()[:120]}")


# --------------------------------------------------------------------------------------------
# Category 1: instruction language
# --------------------------------------------------------------------------------------------

def instruction_language(text: str) -> list[str]:
    """Every piece of instruction-referencing text in `text`."""
    return [why for pattern, why in INSTRUCTION_PATTERNS if pattern.search(text)]


# --------------------------------------------------------------------------------------------
# Category 2: rule markdown paths
# --------------------------------------------------------------------------------------------

def rule_paths(text: str) -> list[str]:
    """Every rule markdown path in `text`."""
    return RULE_PATH.findall(text)


def rule_path_allowed(relative: str) -> bool:
    """Whether `relative` may mention a rule markdown path."""
    return relative in RULE_PATH_FILES or relative.startswith("specification/")


def doc_paths(text: str) -> list[str]:
    """Every `docs/…` path in `text`, ignoring the sanctioned include directives."""
    return [match.group(0) for match in DOC_PATH.finditer(DOC_PATH_ALLOWED.sub("", text))]


def doc_path_includes(text: str) -> list[str]:
    """Every sanctioned `include_str!` directive for a repository document in `text`."""
    return DOC_PATH_ALLOWED.findall(text)


# --------------------------------------------------------------------------------------------
# Category 3: internal module names
# --------------------------------------------------------------------------------------------

def internal_modules(text: str) -> list[str]:
    """Every mention of an internal module path in `text`."""
    return INTERNAL_MODULE.findall(text)


def rendered_text(html: str) -> str:
    """The prose of a rendered page: tags (and therefore their attributes) removed."""
    return re.sub(r"<[^>]+>", " ", html)


def internal_modules_rendered(text: str) -> list[str]:
    """Every mention of an internal module in the prose of a rendered page.

    Tag attributes are ignored: rustdoc puts the defining path of a re-exported type into a `title`
    attribute, which no reader sees, while a mention in the text itself is exactly what this
    category is about.
    """
    return INTERNAL_MODULE_RENDERED.findall(rendered_text(text))


# --------------------------------------------------------------------------------------------
# Category 4: migration wording
# --------------------------------------------------------------------------------------------

def migration_wording(text: str) -> list[str]:
    """Every piece of migration or back-compatibility wording in `text`."""
    return MIGRATION.findall(text)


# --------------------------------------------------------------------------------------------
# Category 5: ASCII art
# --------------------------------------------------------------------------------------------

def ascii_art(text: str) -> list[str]:
    """Every box-drawing character in `text`."""
    return ASCII_ART.findall(text)


# --------------------------------------------------------------------------------------------
# Category 6: path case
# --------------------------------------------------------------------------------------------

def include_paths(text: str, source: Path) -> list[str]:
    """Every path an `include_str!` directive in `text` names, as a repository-relative path."""
    paths = []
    for match in re.finditer(r'include_str!\("([^"]+)"\)', text):
        candidate = (source.parent / match.group(1)).resolve()
        try:
            paths.append(candidate.relative_to(ROOT).as_posix())
        except ValueError:
            paths.append(match.group(1))
    return paths


# --------------------------------------------------------------------------------------------
# Walking the repository
# --------------------------------------------------------------------------------------------

def sources(include_book: bool) -> list[Path]:
    """The files the source sweep reads, in a deterministic order."""
    files = []
    for pattern in (
        "src/**/*.rs",
        "tests/**/*.rs",
        "examples/*.rs",
        f"{SYS}/src/**/*.rs",
        f"{SYS}/python/**/*.py",
        f"{SYS}/docs/*.md",
        "docs/*.py",
        "docs/design/*.md",
        "scripts/*.sh",
        "scripts/*.py",
        "*.md",
        "Cargo.toml",
        "clippy.toml",
        "Makefile",
    ):
        files.extend(sorted(ROOT.glob(pattern)))
    if include_book:
        files.extend(sorted(ROOT.glob("docs/book/*.md")))
    excluded = {"CHANGELOG.md", "AGENTS.md", "REPORT.md", "audit_docs.py"}
    return [path for path in files if path.is_file() and path.name not in excluded]


def doc_comments(path: Path) -> list[tuple[int, str]]:
    """The `///` and `//!` lines of a Rust file, with their line numbers."""
    if path.suffix != ".rs":
        return []
    return [
        (number, line)
        for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1)
        if line.lstrip().startswith("///") or line.lstrip().startswith("//!")
    ]


def sweep_sources(include_book: bool) -> None:
    """Runs every source-level category."""
    matched_rule_path_files = set()
    includes = set()
    for path in sources(include_book):
        relative = path.relative_to(ROOT).as_posix()
        text = path.read_text(encoding="utf-8")
        includes.update(doc_path_includes(text))

        for number, line in enumerate(text.splitlines(), 1):
            if instruction_language(line) and path.name not in {"CHANGELOG.md", "AGENTS.md"}:
                report("instruction language", path, number, line)
            if RULE_PATH_ANY.search(line) or rule_paths(line):
                matched_rule_path_files.add(relative)
                if not rule_path_allowed(relative):
                    report("rule markdown path", path, number, line)
            if migration_wording(line):
                report("migration wording", path, number, line)
            if ascii_art(line):
                report("ASCII art", path, number, line)

        for number, line in doc_comments(path):
            if relative not in PRIVATE_DOC_FILES and internal_modules(line):
                report("internal module in public docs", path, number, line)
            if doc_paths(line):
                report("repository document path in crate docs", path, number, line)

    # Both allowlists are pinned to what is actually needed, so neither can go stale silently.
    documents = sorted((ROOT / "docs" / "design").glob("*.md"))
    if len(includes) != len(documents):
        report(
            "document includes",
            ROOT / "src/documentation.rs",
            0,
            f"{len(includes)} include directives for {len(documents)} documents",
        )
    unused = sorted(RULE_PATH_FILES - matched_rule_path_files)
    if unused:
        report(
            "stale allowlist",
            ROOT / "docs/audit_docs.py",
            0,
            f"these files no longer mention a rule path, remove them: {', '.join(unused)}",
        )


def sweep_rendered() -> None:
    """Sweeps the rendered rustdoc output for internal module names.

    Only pages a reader can reach from the crate index are inspected: the pages of internal modules
    and the design notes are skipped, the former because a public build does not produce them at all
    (which is asserted) and the latter because they are about the implementation.
    """
    for docs in DOCS:
        if not docs.is_dir():
            violations.append(
                "rendered docs: target/doc is missing; run `cargo doc` before this sweep "
                "(scripts/verify.sh does)"
            )
            return
        # A public build may contain redirect stubs for a re-exported item, but it never contains a
        # *module* page for an internal module; `--document-private-items` does.
        private_pages = [
            module
            for module in sorted(DOCS_SKIP - {"documentation"})
            if (docs / module / "index.html").is_file()
        ]
        if private_pages:
            report(
                "rendered docs",
                docs,
                0,
                "this looks like a `--document-private-items` build (pages for "
                + ", ".join(private_pages)
                + " exist): rebuild the public documentation before sweeping it",
            )
            continue
        for path in sorted(docs.rglob("*.html")):
            parts = path.relative_to(docs).parts
            if parts[0] in DOCS_SKIP or "src" in parts:
                continue
            text = path.read_text(encoding="utf-8", errors="replace")
            found = internal_modules_rendered(text)
            if found:
                report("internal module in rendered docs", path, 0, ", ".join(sorted(set(found))))
            for image in re.findall(r'<img[^>]+src="([^"]+)"', text):
                if image.startswith(("http:", "https:", "data:")):
                    continue
                target = (path.parent / image).resolve()
                if not target.is_file():
                    report(
                        "dangling image",
                        path,
                        0,
                        f"`{image}` was not copied into the output: a markdown image in "
                        "documentation that rustdoc renders is a dead image, so the diagram has to "
                        "be inline",
                    )
        # The documentation pages must show their diagrams for real.
        for page in ("architecture", "bindings"):
            index = docs / "documentation" / page / "index.html"
            if index.is_file() and "<svg" not in index.read_text(encoding="utf-8", errors="replace"):
                report(
                    "missing diagram",
                    index,
                    0,
                    f"the {page} page carries no inline SVG",
                )


# --------------------------------------------------------------------------------------------
# Self-test
# --------------------------------------------------------------------------------------------

def tracked_files() -> set[str] | None:
    """Every file git tracks, with its exact spelling, or `None` outside a git checkout."""
    if not (ROOT / ".git").exists():
        return None
    result = subprocess.run(["git", "ls-files"], cwd=ROOT, capture_output=True, text=True)
    if result.returncode != 0:
        return None
    return set(result.stdout.split())


def untracked_includes(text: str, source: Path, tracked: set[str] | None) -> list[str]:
    """The `include_str!` paths of `text` that are not tracked under exactly that spelling.

    Without a git checkout there is nothing to compare against, so the category accepts everything
    and says so in the report.
    """
    if tracked is None:
        return []
    return [relative for relative in include_paths(text, source) if relative not in tracked]


def sweep_path_case(include_book: bool) -> None:
    """Every included file must be tracked under exactly the spelling the directive uses."""
    tracked = tracked_files()
    if tracked is None:
        print("path case: not a git checkout, the category is skipped")
        return
    for path in sources(include_book):
        if path.suffix != ".rs":
            continue
        for relative in untracked_includes(path.read_text(encoding="utf-8"), path, tracked):
            report(
                "path case",
                path,
                0,
                f"`{relative}` is not tracked under this spelling: the checkout that finds it "
                "may be case-insensitive",
            )


def self_test() -> int:
    """Feeds every category a positive and a negative sample, and asserts the outcome."""
    cases: list[tuple[str, object, str, str]] = [
        ("instruction language", instruction_language, "see requirement (3) for why", "see the design notes"),
        (
            "repository document path (file)",
            doc_paths,
            "see `docs/design/architecture.md`",
            'see [`architecture`](crate::documentation::architecture)',
        ),
        ("repository document path (glob)", doc_paths, "in `docs/design/*.md`", "in the design documents"),
        ("repository document path (directory)", doc_paths, "in `docs/book/`", "in the book"),
        ("internal module (public type module)", internal_modules, "see [`crate::node::Node`]", "see [`Node`]"),
        ("rule markdown path", rule_paths, "as in rule.attributes.id-must-be-name.md", "as in `rule.attributes.id-must-be-name`"),
        ("internal module", internal_modules, "see [`crate::arena`] for the model", "see [`Document`] for the model"),
        ("internal module (rendered)", internal_modules_rendered, "<p>see arena::gc for the model</p>", '<p title="arena::gc">see <a href="x">GcConfig</a></p>'),
        ("migration wording", migration_wording, "migrating from 0.1 is easy", "the library has not been published"),
        ("ASCII art", ascii_art, "root\n├── child", "root (one child)"),
        (
            "path case",
            lambda text: untracked_includes(
                text, ROOT / "src" / "lib.rs", {"docs/assets/model.svg"}
            ),
            'include_str!("../src/does-not-exist.rs")',
            'include_str!("../docs/assets/model.svg")',
        ),
    ]
    problems = []
    for name, check, positive, negative in cases:
        if not check(positive):
            problems.append(f"{name}: the self-test sample was not detected")
        if check(negative):
            problems.append(f"{name}: the clean sample was reported as a violation")
    if problems:
        for problem in problems:
            print(f"self-test failed: {problem}", file=sys.stderr)
        return 1
    print(f"self-test: {len(cases)} categories detect a violation and accept a clean sample")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--include-book", action="store_true", help="also sweep docs/book/**")
    parser.add_argument("--self-test", action="store_true", help="prove that every category can fail")
    parser.add_argument("--skip-rendered", action="store_true", help="do not read target/doc")
    arguments = parser.parse_args()

    if arguments.self_test:
        return self_test()

    sweep_sources(arguments.include_book)
    sweep_path_case(arguments.include_book)
    if not arguments.skip_rendered:
        sweep_rendered()

    if violations:
        print(f"{len(violations)} documentation violation(s):", file=sys.stderr)
        for violation in violations:
            print(f"  - {violation}", file=sys.stderr)
        return 1
    scope = "the crate, tests, examples, scripts, README and docs" + (
        ", including the book" if arguments.include_book else ""
    )
    print(f"documentation sweep ({scope}): no violations")
    return 0


if __name__ == "__main__":
    sys.exit(main())
