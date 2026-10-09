"""biodivine-lib-xml-dom: an XML DOM library with safe namespace handling.

This package is the idiomatic Python layer on top of the native ``biodivine_lib_xml_dom._sys``
extension. It is organised in three parts, mirroring the Rust project:

* ``biodivine_lib_xml_dom._sys`` - a thin, one-to-one mirror of the Rust API. It is importable but
  not part of the documented surface; use the Python classes below.
* the classes in this package - Pythonic wrappers that accept several argument types and convert
  them automatically, expose ``len``/iteration/indexing where it makes sense, and return wrapped
  objects so that ``isinstance`` always works.
* nothing else: no XML logic is reimplemented in Python, so the behaviour is exactly the Rust
  library's.

Quick start::

    import biodivine_lib_xml_dom as xml

    document = xml.Document()
    root = document.create_element("root")
    document.set_root(root)

    # An expanded name carries its namespace, including the prefix:
    ex = xml.Namespace("http://example.com", "ex")
    child = document.create_element(xml.QualifiedName("child", ex))
    root.append_child(child)
    child.append_child(document.create_text("Hello"))

    # But nothing declares that prefix yet, so the document is not valid:
    assert not document.is_valid()
    print(document.validation_errors()[0].kind)   # 'undeclared_prefix'

    root.declare_namespace(ex)
    assert document.is_valid()

    print(xml.write(document))                     # <root ...><ex:child>Hello</ex:child></root>

Thread safety
-------------

A document owns exactly one lock, every method acquires and releases it, and no operation ever
holds two document locks - so a document can be shared freely between threads, and no operation can
deadlock. Parsing, serializing, validating and cross-document copies release the GIL while they
work, so a long operation in one thread does not freeze the others.
"""

from . import _sys as _sys
from .document import Document
from .element import Element
from .errors import (
    XmlDocumentError,
    XmlError,
    XmlIoError,
    XmlNamespaceError,
    XmlSyntaxError,
    XmlValidationError,
)
from .name import QualifiedName
from .namespace import Namespace
from .node import Node, NodeKind
from .node_id import NodeId
from .parsing import parse, parse_file, write, write_file
from .validation import ValidationError, ValidationErrors
from .write_options import DeclarationStyle, EmptyElementStyle, WriteOptions, XmlDeclaration

__version__ = "0.1.0"

__all__ = [
    "Document",
    "Element",
    "Namespace",
    "Node",
    "NodeId",
    "NodeKind",
    "QualifiedName",
    "ValidationError",
    "ValidationErrors",
    "WriteOptions",
    "DeclarationStyle",
    "EmptyElementStyle",
    "XmlDeclaration",
    "XmlDocumentError",
    "XmlError",
    "XmlIoError",
    "XmlNamespaceError",
    "XmlSyntaxError",
    "XmlValidationError",
    "parse",
    "parse_file",
    "write",
    "write_file",
    "__version__",
]
