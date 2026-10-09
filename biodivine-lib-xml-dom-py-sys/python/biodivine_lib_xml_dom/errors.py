"""The exception hierarchy raised by this library.

All exceptions derive from :class:`XmlError`:

* :class:`XmlSyntaxError` - the input document is not well-formed XML (bad names, illegal
  characters, malformed markup, an unsupported encoding, an undeclared entity reference, ...).
* :class:`XmlNamespaceError` - a namespace is used in a way the specification forbids.
* :class:`XmlDocumentError` - a document operation is not allowed (attaching a node of another
  document, creating a cycle, an out-of-range index, attaching the root, ...).
* :class:`XmlIoError` - an underlying I/O operation failed.
* :class:`XmlValidationError` - whole-document validation found problems; ``args[1]`` is a
  :class:`~biodivine_lib_xml_dom.validation.ValidationErrors` sequence and ``args[0]`` is a
  human-readable multi-line summary.
"""

from __future__ import annotations

from . import _sys

XmlError = _sys.XmlError
XmlSyntaxError = _sys.XmlSyntaxError
XmlNamespaceError = _sys.XmlNamespaceError
XmlDocumentError = _sys.XmlDocumentError
XmlIoError = _sys.XmlIoError
XmlValidationError = _sys.XmlValidationError

__all__ = [
    "XmlError",
    "XmlSyntaxError",
    "XmlNamespaceError",
    "XmlDocumentError",
    "XmlIoError",
    "XmlValidationError",
]
