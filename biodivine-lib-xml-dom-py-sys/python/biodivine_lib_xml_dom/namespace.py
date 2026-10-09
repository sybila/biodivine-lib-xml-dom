"""Pythonic wrapper for an XML namespace."""

from __future__ import annotations

from typing import Optional

from . import _sys


class Namespace:
    """An immutable XML namespace: a URI plus an optional prefix.

    Accepts the same arguments as the Rust constructor and raises `XmlNamespaceError` for an
    invalid pair::

        Namespace("http://example.com")               # default namespace
        Namespace("http://example.com", "ex")         # prefixed
        Namespace.prefixed("http://example.com", "ex")

    Equality is structural (URI *and* prefix); use :meth:`is_equal_ns` for the specification's
    notion of namespace equality, which ignores the prefix.
    """

    __slots__ = ("_namespace",)

    def __init__(self, uri: str, prefix: Optional[str] = None) -> None:
        self._namespace = _sys.Namespace(uri, prefix)

    @classmethod
    def _wrap(cls, namespace: "_sys.Namespace") -> "Namespace":
        wrapper = cls.__new__(cls)
        wrapper._namespace = namespace
        return wrapper

    @classmethod
    def without_prefix(cls, uri: str) -> "Namespace":
        """Creates a default namespace (no prefix)."""
        return cls._wrap(_sys.Namespace.without_prefix(uri))

    @classmethod
    def prefixed(cls, uri: str, prefix: str) -> "Namespace":
        """Creates a namespace with a prefix."""
        return cls._wrap(_sys.Namespace.prefixed(uri, prefix))

    @property
    def uri(self) -> str:
        """The namespace URI."""
        return self._namespace.uri()

    @property
    def prefix(self) -> Optional[str]:
        """The namespace prefix, or ``None`` for a default namespace."""
        return self._namespace.prefix()

    def is_equal_ns(self, other: "Namespace") -> bool:
        """Whether this namespace and ``other`` have the same URI."""
        return self._namespace.is_equal_ns(_sys.Namespace(other.uri, other.prefix))

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Namespace):
            return NotImplemented
        return self._namespace == other._namespace

    def __hash__(self) -> int:
        return hash(self._namespace)

    def __str__(self) -> str:
        return str(self._namespace)

    def __repr__(self) -> str:
        if self.prefix is None:
            return f"Namespace({self.uri!r})"
        return f"Namespace({self.uri!r}, prefix={self.prefix!r})"
