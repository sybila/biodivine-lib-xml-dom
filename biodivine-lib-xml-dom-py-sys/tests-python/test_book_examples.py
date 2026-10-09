"""Runs every Python example of the documentation book.

The book includes these files verbatim, so this module is what makes "the examples work" a checked
property: a chapter cannot show a snippet that no longer runs.
"""

from __future__ import annotations

import runpy
from pathlib import Path

import pytest

BOOK_EXAMPLES = Path(__file__).resolve().parents[2] / "docs" / "book" / "examples" / "python"
EXAMPLES = sorted(path for path in BOOK_EXAMPLES.glob("*.py"))


def test_the_book_has_python_examples() -> None:
    """A mis-globbed directory would silently run nothing, so the count is asserted."""
    assert EXAMPLES, f"no Python examples found in {BOOK_EXAMPLES}"
    assert len(EXAMPLES) >= 7


@pytest.mark.parametrize("example", EXAMPLES, ids=lambda path: path.stem)
def test_book_example_runs(example: Path) -> None:
    runpy.run_path(str(example), run_name="__main__")
