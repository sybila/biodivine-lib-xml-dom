"""Validation results."""

from __future__ import annotations

from dataclasses import dataclass
from typing import List, Optional

from . import _sys
from .node_id import NodeId


@dataclass(frozen=True)
class ValidationError:
    """One problem found by :meth:`Document.validate`.

    The fields are plain values, so a problem can be compared, hashed and kept after the document
    is gone. ``node`` can be turned back into a node with :meth:`Document.node`.
    """

    kind: str
    """A stable snake_case name for the kind of problem, e.g. ``"undeclared_prefix"``."""
    rule: str
    """The identifier of the specification rule this problem comes from."""
    node: Optional[NodeId]
    """The node the problem is attached to, or ``None`` for a document-wide problem."""
    message: str
    """A human-readable description."""

    @classmethod
    def _wrap(cls, error: "_sys.ValidationError") -> "ValidationError":
        node = error.node()
        return cls(
            kind=error.kind(),
            rule=error.rule(),
            node=None if node is None else NodeId._wrap(node),
            message=error.message(),
        )

    def __str__(self) -> str:
        if self.node is None:
            return f"{self.message} [{self.rule}]"
        return f"node {self.node}: {self.message} [{self.rule}]"


class ValidationErrors(list):
    """Every problem found by one validation run.

    A ``list`` of :class:`ValidationError`, so it can be iterated, indexed, compared with ``[]`` and
    unpacked like any other sequence::

        errors = document.validation_errors()
        if errors:
            for problem in errors:
                print(problem.kind, problem.message)
    """

    def kinds(self) -> List[str]:
        """The kinds of all problems, in order."""
        return [problem.kind for problem in self]

    def messages(self) -> List[str]:
        """The messages of all problems, in order."""
        return [problem.message for problem in self]

    def __str__(self) -> str:
        if not self:
            return "no validation problems"
        return "\n".join(f"- {problem}" for problem in self)
