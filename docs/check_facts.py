#!/usr/bin/env python3
"""Checks the hand-written numbers in the hand-off documents against the artefacts.

Run from the repository root:

    python3 docs/check_facts.py

The lesson this script exists for: `REPORT.md` said "27 commits" while the branch had 30, because a
number that a human typed in prose is not a number that anything verifies. Everything checkable is
therefore checked here, and `scripts/verify.sh` runs it, so the drift cannot recur.

What is verified:

* the commit counts quoted next to a named commit (`git rev-list --count <commit> ^master`);
* the number of gates stated in `docs/design/VERIFICATION.md`, against both its own gate table and
  the `-> exit` lines of the transcript it embeds;
* the test counts stated in `docs/design/VERIFICATION.md`, against the `test result:` lines of that
  same transcript;
* the rule numbers (`specification/rules/` file count, inventory rows, enforcement rows and per-layer
  totals) against the generated files themselves;
* the book's chapter and language-pair counts, by running the book checker;
* the version, single-sourced: the workspace manifest, the crates, and the number the Python test
  asserts.
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
VERIFICATION = ROOT / "docs/design/VERIFICATION.md"
REPORT = ROOT / "REPORT.md"
INVENTORY = ROOT / "docs/design/evidence/rule-inventory.md"
ENFORCEMENT = ROOT / "docs/design/evidence/rule-enforcement.md"
RULES = ROOT / "specification/rules"

problems: list[str] = []


def fail(message: str) -> None:
    problems.append(message)


def git(*args: str) -> str:
    """Runs git and returns stdout, or `""` if it failed (so -1 shows up as a mismatch)."""
    result = subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True)
    return result.stdout.strip() if result.returncode == 0 else ""


def check_commit_counts() -> None:
    """Every `git rev-list --count <sha> ^master` claim must match the repository."""
    for path in (REPORT, VERIFICATION):
        if not path.is_file():
            fail(f"{path.name} is missing")
            continue
        text = path.read_text(encoding="utf-8")
        for match in re.finditer(r"`git rev-list --count ([0-9a-f]{7,40}) \^master`\s*=\s*(\d+)", text):
            sha, claimed = match.group(1), int(match.group(2))
            actual = git("rev-list", "--count", sha, "^master")
            actual = int(actual) if actual else -1
            if actual != claimed:
                fail(f"{path.name}: claims {claimed} commits for {sha}, git says {actual}")
            else:
                print(f"{path.name}: {sha} -> {claimed} commits over master (verified)")


def check_verification_numbers() -> None:
    text = VERIFICATION.read_text(encoding="utf-8")
    gate_table = text[text.index("| command | exit |"):text.index("## Test counts")]
    count_table = text[text.index("## Test counts"):text.index("## Requirement by requirement")]
    rows = len(re.findall(r"^\| `.+` \| \d+ \|$", gate_table, re.M))
    stated = re.search(r"All (\d+) gates passed", text)
    transcript = text[text.index("## Transcript"):]

    # The transcript, split per command: `$ <command>` followed by output and `-> exit <code>`.
    blocks: list[tuple[str, str]] = []
    current, buffer = None, []
    for line in transcript.split("\n"):
        if line.startswith("$ "):
            if current is not None:
                blocks.append((current, "\n".join(buffer)))
            current, buffer = line[2:].strip(), []
        elif line.startswith("-> exit "):
            if current is not None:
                blocks.append((current, "\n".join(buffer)))
            current, buffer = None, []
        elif current is not None:
            buffer.append(line)

    exits = len(blocks)
    if stated and int(stated.group(1)) != rows:
        fail(f"VERIFICATION.md: states {stated.group(1)} gates but the table has {rows} rows")
    if exits != rows:
        fail(f"VERIFICATION.md: the table has {rows} gates but the transcript has {exits} commands")
    print(f"VERIFICATION.md: {rows} gates (table, stated count and transcript agree)")

    # Each stated test count must equal what that command actually printed.
    for name, claimed in re.findall(r"^\| `([^`]+)` \| (\d+) \|$", count_table, re.M):
        claimed = int(claimed)
        matching = [
            (command, output)
            for command, output in blocks
            if (command == name if name.startswith("cargo") else name in command)
        ]
        if not matching:
            fail(f"VERIFICATION.md: no transcript entry for `{name}`")
            continue
        total = 0
        for _, output in matching:
            total += sum(int(n) for n in re.findall(r"test result: ok\. (\d+) passed", output))
            total += sum(int(n) for n in re.findall(r"=+ (\d+) passed in", output))
        if total != claimed:
            fail(f"VERIFICATION.md: `{name}` claims {claimed} tests, the transcript has {total}")
        else:
            print(f"VERIFICATION.md: `{name}` -> {total} tests (verified against the transcript)")


def check_rule_numbers() -> None:
    files = len(list(RULES.glob("rule.*.md")))
    inventory = INVENTORY.read_text(encoding="utf-8")
    rows = len(re.findall(r"^\| `[^`]+` \| [A-Z]", inventory, re.M))
    if rows != files:
        fail(f"rule-inventory.md has {rows} rows but there are {files} rule files")
    called_out = re.search(r"from (\d+) rule files", inventory)
    if called_out and int(called_out.group(1)) != files:
        fail(f"rule-inventory.md says it was generated from {called_out.group(1)} files, found {files}")
    print(f"rule-inventory.md: {rows} rows for {files} rule files")

    enforcement = ENFORCEMENT.read_text(encoding="utf-8")
    enforced_rows = len(re.findall(r"\| (enforced|partial|deferred|n/a) \|", enforcement))
    summary = dict(re.findall(r"^- (\d+) (enforced|partial|deferred|not applicable)", enforcement, re.M))
    summary_total = sum(
        int(count)
        for count in re.findall(
            r"^- (\d+) (?:enforced|partial|deferred|not applicable)", enforcement, re.M
        )
    )
    if summary_total != enforced_rows:
        fail(f"rule-enforcement.md: rows say {enforced_rows}, summary says {summary_total}")
    if re.search(r"(\d+) total layer-B/C/D rules", enforcement):
        total = int(re.search(r"(\d+) total layer-B/C/D rules", enforcement).group(1))
        if total != enforced_rows:
            fail(f"rule-enforcement.md: {total} total vs {enforced_rows} rows")
    print(f"rule-enforcement.md: {enforced_rows} rules with a verdict ({summary_total} in the summary)")


def check_book_numbers() -> None:
    result = subprocess.run(
        [sys.executable, str(ROOT / "docs/check_book.py")], cwd=ROOT, capture_output=True, text=True
    )
    if result.returncode != 0:
        fail(f"docs/check_book.py failed:\n{result.stdout}{result.stderr}")
        return
    chapters = int(re.search(r"(\d+) chapters", result.stdout).group(1))
    pairs = int(re.search(r"(\d+) language pairs", result.stdout).group(1))
    text = VERIFICATION.read_text(encoding="utf-8")
    if f"{chapters} chapters" not in text:
        fail(f"VERIFICATION.md does not mention the book's {chapters} chapters")
    if f"{pairs} Rust/Python" not in text and f"{pairs} language pairs" not in text:
        fail(f"VERIFICATION.md does not mention the book's {pairs} example pairs")
    print(f"book: {chapters} chapters, {pairs} language pairs (verified against the book itself)")


def check_version() -> None:
    manifest = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    workspace_version = re.search(r'^version = "([^"]+)"', manifest, re.M).group(1)
    for crate in (ROOT / "Cargo.toml", ROOT / "biodivine-lib-xml-dom-py-sys/Cargo.toml"):
        if "version.workspace = true" not in crate.read_text(encoding="utf-8"):
            fail(f"{crate} does not inherit the workspace version")
    test = (ROOT / "biodivine-lib-xml-dom-py-sys/tests-python/test_public_surface.py").read_text(encoding="utf-8")
    if f'"{workspace_version}"' not in test:
        fail(f"the public-surface test does not assert the workspace version {workspace_version}")
    if "dynamic = [\"version\"]" not in (ROOT / "biodivine-lib-xml-dom-py-sys/pyproject.toml").read_text(encoding="utf-8"):
        fail("pyproject.toml does not take its version from the manifest")
    print(f"version: {workspace_version}, inherited by both crates, asserted from Python")


def main() -> int:
    for check in (
        check_commit_counts,
        check_verification_numbers,
        check_rule_numbers,
        check_book_numbers,
        check_version,
    ):
        check()
    if problems:
        print(f"\n{len(problems)} fact problem(s):", file=sys.stderr)
        for problem in problems:
            print(f"  - {problem}", file=sys.stderr)
        return 1
    print("every stated number matches the artefact")
    return 0


if __name__ == "__main__":
    sys.exit(main())
