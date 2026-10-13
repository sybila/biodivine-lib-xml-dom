"""The exception hierarchy raised by this library.

All exceptions of this library derive from :class:`XmlError`:

* :class:`XmlSyntaxError` - the input document is not well-formed XML (bad names, illegal
  characters, malformed markup, an unsupported encoding, an undeclared entity reference, ...).
* :class:`XmlNamespaceError` - a namespace is used in a way the specification forbids.
* :class:`XmlDocumentError` - a document operation is not allowed (attaching a node of another
  document, creating a cycle, an out-of-range index, attaching the root, ...).
* :class:`XmlIoError` - an underlying I/O operation failed.
* :class:`XmlValidationError` - whole-document validation found problems. Its ``errors`` attribute
  (also ``args[1]``) is a :class:`~biodivine_lib_xml_dom.ValidationErrors` list, and ``args[0]`` is
  a human-readable multi-line summary.

A long operation (parsing, serializing, validating, deep copies, garbage collection) can be
interrupted by a Python signal handler, for example by Ctrl+C. The exception the handler raises
(``KeyboardInterrupt`` for Ctrl+C) is propagated unchanged; it is not an :class:`XmlError`.
"""

from __future__ import annotations

from . import _sys

XmlError = _sys.XmlError
XmlSyntaxError = _sys.XmlSyntaxError
XmlNamespaceError = _sys.XmlNamespaceError
XmlDocumentError = _sys.XmlDocumentError
XmlIoError = _sys.XmlIoError
XmlValidationError = _sys.XmlValidationError

# The classes are defined by the native module; present them where users import them from.
for _exception in (
    XmlError,
    XmlSyntaxError,
    XmlNamespaceError,
    XmlDocumentError,
    XmlIoError,
    XmlValidationError,
):
    _exception.__module__ = "biodivine_lib_xml_dom"
del _exception

__all__ = [
    "XmlError",
    "XmlSyntaxError",
    "XmlNamespaceError",
    "XmlDocumentError",
    "XmlIoError",
    "XmlValidationError",
]
