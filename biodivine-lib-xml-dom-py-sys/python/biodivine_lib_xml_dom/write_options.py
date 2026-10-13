"""Serialization options and the XML declaration."""

from __future__ import annotations

import enum
from dataclasses import dataclass
from typing import Optional

from . import _sys


class DeclarationStyle(enum.Enum):
    """When :meth:`Document.write` writes the ``<?xml ...?>`` declaration."""

    NEVER = "never"
    """Never write a declaration."""
    IF_PRESENT = "if_present"
    """Write a declaration if the document has one (the default)."""
    ALWAYS = "always"
    """Always write one, using :meth:`XmlDeclaration.utf8` when the document has none."""


class EmptyElementStyle(enum.Enum):
    """How :meth:`Document.write` writes an element without children."""

    SELF_CLOSING = "self_closing"
    """``<a/>`` (the default)."""
    EXPLICIT_END_TAG = "explicit_end_tag"
    """``<a></a>``."""


_DECLARATION_STYLES = {
    DeclarationStyle.NEVER: _sys.DeclarationStyle.Never,
    DeclarationStyle.IF_PRESENT: _sys.DeclarationStyle.IfPresent,
    DeclarationStyle.ALWAYS: _sys.DeclarationStyle.Always,
}

_EMPTY_ELEMENT_STYLES = {
    EmptyElementStyle.SELF_CLOSING: _sys.EmptyElementStyle.SelfClosing,
    EmptyElementStyle.EXPLICIT_END_TAG: _sys.EmptyElementStyle.ExplicitEndTag,
}


@dataclass(frozen=True)
class WriteOptions:
    """Options for :meth:`Document.write` and :meth:`Document.write_file`.

    The defaults reproduce a parsed document as closely as the data model allows::

        document.write(WriteOptions(declaration=DeclarationStyle.ALWAYS))
    """

    declaration: DeclarationStyle = DeclarationStyle.IF_PRESENT
    """When to write the XML declaration."""
    empty_elements: EmptyElementStyle = EmptyElementStyle.SELF_CLOSING
    """How to write elements without children."""

    def _unwrap(self) -> "_sys.WriteOptions":
        return _sys.WriteOptions(
            _DECLARATION_STYLES[self.declaration],
            _EMPTY_ELEMENT_STYLES[self.empty_elements],
        )


class XmlDeclaration:
    """The ``<?xml version="1.0" encoding="UTF-8"?>`` declaration of a document.

    The declaration is a property of the document rather than a node of its tree. Only XML 1.0 and
    UTF-8 are supported, so the constructor raises ``XmlSyntaxError`` for any other version or
    encoding::

        XmlDeclaration(encoding="UTF-8", standalone=True)
    """

    __slots__ = ("_declaration",)

    def __init__(
        self,
        version: str = "1.0",
        encoding: Optional[str] = None,
        standalone: Optional[bool] = None,
    ) -> None:
        self._declaration = _sys.XmlDeclaration(version, encoding, standalone)

    @classmethod
    def _wrap(cls, declaration: "_sys.XmlDeclaration") -> "XmlDeclaration":
        wrapper = cls.__new__(cls)
        wrapper._declaration = declaration
        return wrapper

    @classmethod
    def utf8(cls) -> "XmlDeclaration":
        """The declaration this library writes by default: version 1.0, UTF-8."""
        return cls._wrap(_sys.XmlDeclaration.utf8())

    @property
    def version(self) -> str:
        """The declared XML version."""
        return self._declaration.version()

    @property
    def encoding(self) -> Optional[str]:
        """The declared encoding, if the declaration has one."""
        return self._declaration.encoding()

    @property
    def standalone(self) -> Optional[bool]:
        """The declared ``standalone`` value, if the declaration has one."""
        return self._declaration.standalone()

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, XmlDeclaration):
            return NotImplemented
        return self._declaration == other._declaration

    def __hash__(self) -> int:
        return hash((self.version, self.encoding, self.standalone))

    def __str__(self) -> str:
        return str(self._declaration)

    def __repr__(self) -> str:
        return f"XmlDeclaration({str(self)!r})"


__all__ = ["DeclarationStyle", "EmptyElementStyle", "WriteOptions", "XmlDeclaration"]
