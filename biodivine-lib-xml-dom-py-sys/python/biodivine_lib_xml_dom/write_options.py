"""Serialization options and the XML declaration."""

from __future__ import annotations

from . import _sys

DeclarationStyle = _sys.DeclarationStyle
"""When to write the ``<?xml ...?>`` declaration: ``Never``, ``IfPresent`` or ``Always``."""

EmptyElementStyle = _sys.EmptyElementStyle
"""How to write an element without children: ``SelfClosing`` (``<a/>``) or ``ExplicitEndTag``."""

WriteOptions = _sys.WriteOptions
"""Options for the ``write_*`` functions: ``declaration`` and ``empty_elements``.

The defaults reproduce a parsed document as closely as the data model allows.
"""

XmlDeclaration = _sys.XmlDeclaration
"""The ``<?xml version="1.0" encoding="UTF-8"?>`` declaration of a document."""

__all__ = ["DeclarationStyle", "EmptyElementStyle", "WriteOptions", "XmlDeclaration"]
