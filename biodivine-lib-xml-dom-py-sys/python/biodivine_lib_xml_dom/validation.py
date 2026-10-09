"""Pythonic wrappers for validation results."""

from __future__ import annotations

from typing import List, Optional

from . import _sys
from .node_id import NodeId


class ValidationError:
    """One problem found by :meth:`Document.validate`."""

    __slots__ = ("_error",)

    def __init__(self, error: "_sys.ValidationError") -> None:
        self._error = error

    @property
    def kind(self) -> str:
        """A stable snake_case name for the kind of problem, e.g. ``"undeclared_prefix"``."""
        return self._error.kind()

    @property
    def rule(self) -> str:
        """The rule file in ``specification/rules/`` this problem comes from."""
        return self._error.rule()

    @property
    def node(self) -> Optional[NodeId]:
        """The node the problem is attached to, or ``None`` for a document-wide problem."""
        inner = self._error.node()
        return None if inner is None else NodeId._wrap(inner)

    @property
    def message(self) -> str:
        """A human-readable description."""
        return self._error.message()

    def __str__(self) -> str:
        return str(self._error)

    def __repr__(self) -> str:
        node = self.node
        return f"ValidationError(kind={self.kind!r}, node={None if node is None else int(node)}, message={self.message!r})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, ValidationError):
            return NotImplemented
        return self._error == other._error

    def __hash__(self) -> int:
        return hash((self.kind, self.message, None if self.node is None else int(self.node)))


class ValidationErrors(list):
    """Every problem found by one validation run.

    A ``list`` of :class:`ValidationError`, so it can be iterated, indexed, compared with ``[]`` and
    unpacked like any other sequence::

        errors = document.validation_errors()
        if errors:
            for problem in errors:
                print(problem.kind, problem.message)
    """

    def __init__(self, errors: List[ValidationError]) -> None:
        super().__init__(errors)

    @classmethod
    def _from_sys(cls, errors: "_sys.ValidationErrors") -> "ValidationErrors":
        return cls([ValidationError(error) for error in errors.errors()])

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
