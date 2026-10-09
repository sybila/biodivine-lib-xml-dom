"""Pythonic wrapper for an expanded (qualified) XML name."""

from __future__ import annotations

from typing import Optional, Union

from . import _sys
from .namespace import Namespace


class QualifiedName:
    """An immutable expanded name: a local name plus an optional namespace.

    The namespace may be given as a :class:`Namespace` or as a plain URI string::

        QualifiedName("child")
        QualifiedName("child", "http://example.com")
        QualifiedName("child", Namespace("http://example.com", "ex"))

    Equality and hashing compare the local name and the namespace *URI*, so two names that differ
    only in their prefix are equal - which is what the specification's "expanded name" means.
    """

    __slots__ = ("_name",)

    def __init__(self, local_name: str, namespace: Union[Namespace, str, None] = None) -> None:
        if isinstance(namespace, str):
            namespace = Namespace(namespace)
        self._name = _sys.QualifiedName(local_name, None if namespace is None else namespace._namespace)

    @classmethod
    def _wrap(cls, name: "_sys.QualifiedName") -> "QualifiedName":
        wrapper = cls.__new__(cls)
        wrapper._name = name
        return wrapper

    @classmethod
    def without_namespace(cls, local_name: str) -> "QualifiedName":
        """Creates a name with no namespace."""
        return cls._wrap(_sys.QualifiedName.without_namespace(local_name))

    @classmethod
    def with_namespace(cls, local_name: str, namespace: Union[Namespace, str]) -> "QualifiedName":
        """Creates a name in ``namespace``."""
        if isinstance(namespace, str):
            namespace = Namespace(namespace)
        return cls._wrap(_sys.QualifiedName.with_namespace(local_name, namespace._namespace))

    @property
    def local_name(self) -> str:
        """The local part of the name."""
        return self._name.local_name()

    @property
    def namespace(self) -> Optional[Namespace]:
        """The namespace, or ``None``."""
        inner = self._name.namespace()
        return None if inner is None else Namespace._wrap(inner)

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, QualifiedName):
            return NotImplemented
        return self._name == other._name

    def __hash__(self) -> int:
        return hash(self._name)

    def __str__(self) -> str:
        return str(self._name)

    def __repr__(self) -> str:
        return f"QualifiedName({str(self)!r})"
