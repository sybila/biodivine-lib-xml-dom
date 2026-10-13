"""The public surface of the pure-Python package.

The point of this module is to make the package's *interface* a checked property rather than an
accident: a name that stops resolving, a native ``_sys`` object that starts leaking into the
documented surface, or a class that can no longer be produced by the documented factories all make
this file fail.
"""

from __future__ import annotations

import copy
import enum
import re
import tempfile
from pathlib import Path

import pytest

import biodivine_lib_xml_dom as xml
from biodivine_lib_xml_dom import _sys

#: The exception classes are the one kind of native type in the public surface: PyO3 has to define
#: them, and the package re-exports them under its own module name.
EXCEPTIONS = {
    "XmlError",
    "XmlSyntaxError",
    "XmlNamespaceError",
    "XmlDocumentError",
    "XmlIoError",
    "XmlValidationError",
}

#: Every class of the public surface, apart from the exceptions.
CLASSES = {
    "DeclarationStyle",
    "Document",
    "Element",
    "EmptyElementStyle",
    "GcStats",
    "Namespace",
    "Node",
    "NodeId",
    "NodeKind",
    "QualifiedName",
    "ValidationError",
    "ValidationErrors",
    "WriteOptions",
    "XmlDeclaration",
}

ROOT = Path(__file__).resolve().parents[2]


def _is_native(value: object) -> bool:
    """Whether ``value`` is an instance of a class defined by the native module."""
    return type(value).__module__.startswith("biodivine_lib_xml_dom._sys")


def test_the_version_is_the_same_in_every_layer() -> None:
    """The crates inherit `version` from `[workspace.package]`, the extension reports it, and the
    package re-exports the extension's value."""
    manifest = (ROOT / "Cargo.toml").read_text()
    workspace = manifest[manifest.index("[workspace.package]"):]
    version = re.search(r'^version = "([^"]+)"', workspace, flags=re.MULTILINE)
    assert version is not None
    assert xml.__version__ == version.group(1)
    assert _sys.__version__ == xml.__version__


def test_every_exported_name_resolves() -> None:
    assert xml.__all__, "the package must declare its public surface"
    assert len(set(xml.__all__)) == len(xml.__all__), "__all__ must not repeat a name"
    assert xml.__all__ == sorted(xml.__all__), "__all__ must stay sorted"
    for name in xml.__all__:
        assert hasattr(xml, name), f"{name} is in __all__ but does not resolve"


def test_the_public_surface_is_exactly_classes_and_exceptions() -> None:
    assert set(xml.__all__) == CLASSES | EXCEPTIONS | {"__version__"}
    for name in CLASSES:
        cls = getattr(xml, name)
        assert isinstance(cls, type)
        assert cls.__module__.startswith("biodivine_lib_xml_dom."), name
        assert not cls.__module__.startswith("biodivine_lib_xml_dom._sys"), name
    for name in EXCEPTIONS:
        cls = getattr(xml, name)
        assert issubclass(cls, xml.XmlError)
        assert cls.__module__ == "biodivine_lib_xml_dom"
    assert issubclass(xml.XmlError, Exception)
    for enum_class in (xml.NodeKind, xml.DeclarationStyle, xml.EmptyElementStyle):
        assert issubclass(enum_class, enum.Enum)


def test_no_native_object_is_ever_returned() -> None:
    document = xml.Document.parse(
        '<?xml version="1.0"?><ex:a xmlns:ex="http://example.com" b="1"><c/>t</ex:a>'
    )
    root = document.root
    values = [
        document,
        root,
        root.kind,
        root.id,
        root.qualified_name,
        root.namespace,
        root.document,
        root.children()[0],
        root.children()[1],
        root.children()[0].parent,
        document.xml_declaration,
        document.collect_garbage(),
        *root.attributes,
        *root.namespace_declarations.values(),
        *xml.Document().validation_errors(),
    ]
    for value in values:
        assert not _is_native(value), f"{value!r} is a native object"

    invalid = xml.Document()
    with pytest.raises(xml.XmlValidationError) as caught:
        invalid.validate()
    assert isinstance(caught.value.errors, xml.ValidationErrors)
    assert caught.value.args[1] is caught.value.errors
    assert all(not _is_native(problem) for problem in caught.value.errors)


def test_handles_have_no_public_constructor() -> None:
    with pytest.raises(TypeError):
        xml.Node()
    with pytest.raises(TypeError):
        xml.Element()


def test_value_classes_are_constructible() -> None:
    assert isinstance(xml.Document(), xml.Document)
    assert isinstance(xml.Namespace("http://example.com", "ex"), xml.Namespace)
    assert isinstance(xml.QualifiedName("child"), xml.QualifiedName)
    assert isinstance(xml.NodeId(3), xml.NodeId)
    assert xml.WriteOptions() == xml.WriteOptions(
        xml.DeclarationStyle.IF_PRESENT, xml.EmptyElementStyle.SELF_CLOSING
    )
    assert hash(xml.WriteOptions()) == hash(xml.WriteOptions())
    assert xml.XmlDeclaration() == xml.XmlDeclaration("1.0")
    with pytest.raises(xml.XmlSyntaxError):
        xml.XmlDeclaration("1.1")
    with pytest.raises(xml.XmlSyntaxError):
        xml.XmlDeclaration(encoding="UTF-16")
    with pytest.raises(ValueError):
        xml.NodeId(-1)


def test_isinstance_works_for_the_wrapper_hierarchy() -> None:
    document = xml.Document()
    root = document.create_element("root")
    document.set_root(root)
    text = document.create_text("x")

    assert isinstance(root, xml.Element)
    assert isinstance(root, xml.Node)  # Element subclasses Node in the Python layer
    assert isinstance(text, xml.Node)
    assert not isinstance(text, xml.Element)
    root.append_child(text)
    assert isinstance(root.children()[0], xml.Node)
    assert isinstance(root.children()[0].parent, xml.Element)
    assert root.kind is xml.NodeKind.ELEMENT
    assert text.kind is xml.NodeKind.TEXT


def test_parsing_and_writing_are_document_methods() -> None:
    assert xml.Document.parse("<a/>").write() == "<a/>"
    assert xml.Document.parse(b"<a/>").write() == "<a/>"
    assert xml.Document.parse(bytearray(b"<a/>")).write() == "<a/>"
    assert xml.Document.parse(memoryview(b"<a/>")).write() == "<a/>"
    with pytest.raises(TypeError):
        xml.Document.parse(3)  # type: ignore[arg-type]
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "doc.xml"
        xml.Document.parse("<a/>").write_file(path)
        assert xml.Document.parse_file(path).write() == "<a/>"


def test_copy_shares_and_deepcopy_copies() -> None:
    document = xml.Document.parse('<?xml version="1.0"?><a><b/></a>')
    assert copy.copy(document) is document
    duplicate = copy.deepcopy(document)
    assert duplicate != document
    assert duplicate.write() == document.write()
    duplicate.root.append_child(duplicate.create_element("c"))
    assert document.write() == '<?xml version="1.0"?><a><b/></a>'


def test_argument_coercion_is_part_of_the_surface() -> None:
    """The name spellings promised by the documentation all work, everywhere a name is taken."""
    document = xml.Document()
    namespace = xml.Namespace("http://example.com", "ex")
    for name in ("child", ("child", None), ("child", "http://example.com"), ("child", namespace),
                 xml.QualifiedName("child", namespace)):
        element = document.create_element(name)
        assert isinstance(element, xml.Element)

    assert document.create_element("child").qualified_name.namespace is None
    assert document.create_element(("child", namespace)).qualified_name.namespace == namespace
    assert (
        document.create_element(("child", "http://example.com")).qualified_name.namespace.uri
        == "http://example.com"
    )

    root = document.create_element("root")
    for name in ("class", ("class", None), ("class", namespace),
                 xml.QualifiedName("class", namespace)):
        root.set_attribute(name, "value")
        assert root.get(name) == "value"
    with pytest.raises(TypeError):
        document.create_element(("too", "many", "parts"))
    with pytest.raises(TypeError):
        document.create_element(3)  # type: ignore[arg-type]
