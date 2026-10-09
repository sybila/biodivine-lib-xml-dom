"""The public surface of the pure-Python package.

The point of this module is to make the package's *interface* a checked property rather than an
accident: a name that stops resolving, a native ``_sys`` type that starts leaking into the
documented surface, or a class that can no longer be produced by the documented factories all make
this file fail. It is the Python counterpart of the `rule-enforcement` and `BINDINGS` audits on the
Rust side.
"""

from __future__ import annotations

import inspect
import types

import pytest

import biodivine_lib_xml_dom as xml
from biodivine_lib_xml_dom import _sys


#: Native objects that are deliberately re-exported instead of wrapped, with the reason. They are
#: value types (an enum and two plain data holders) whose native form is already idiomatic in
#: Python, and the exception hierarchy, which PyO3 has to define as native types. Everything else in
#: `__all__` must be a Python class or function defined by this package.
NATIVE_RE_EXPORTS = {
    "WriteOptions",
    "DeclarationStyle",
    "EmptyElementStyle",
    "XmlDeclaration",
    "XmlError",
    "XmlSyntaxError",
    "XmlNamespaceError",
    "XmlDocumentError",
    "XmlIoError",
    "XmlValidationError",
}


def _is_native(obj: type) -> bool:
    """Whether ``obj`` comes from the native module.

    PyO3 sets ``__module__`` to the name passed to ``create_exception!``/``#[pyclass]``, which for
    this crate is ``"_sys"`` for the exception types and ``"biodivine_lib_xml_dom._sys"`` for the
    classes, so both spellings have to be accepted.
    """
    module = getattr(obj, "__module__", "")
    return module == "_sys" or module.startswith("biodivine_lib_xml_dom._sys")


def test_every_exported_name_resolves() -> None:
    assert xml.__all__, "the package must declare its public surface"
    assert len(set(xml.__all__)) == len(xml.__all__), "__all__ must not repeat a name"
    assert xml.__all__ == sorted(xml.__all__), "__all__ must stay sorted"
    for name in xml.__all__:
        assert hasattr(xml, name), f"{name} is in __all__ but does not resolve"


def test_the_public_surface_is_wrappers_and_functions_only() -> None:
    """Every exported object is a wrapper class, a function, or an explicitly allow-listed native
    value type/exception — never an accidental `_sys` leak."""
    wrappers, functions = [], []
    for name in xml.__all__:
        obj = getattr(xml, name)
        if name in NATIVE_RE_EXPORTS:
            assert isinstance(obj, type), f"{name} should be a native type"
            assert _is_native(obj), f"{name} is allow-listed as a native re-export but is not one"
            continue
        if isinstance(obj, type):
            assert obj.__module__.startswith("biodivine_lib_xml_dom."), (
                f"{name} leaks a type from {obj.__module__}"
            )
            assert not _is_native(obj), (
                f"{name} leaks a native type; add it to NATIVE_RE_EXPORTS if that is deliberate"
            )
            wrappers.append(name)
        elif callable(obj) or isinstance(obj, types.FunctionType):
            functions.append(name)
        elif isinstance(obj, str):
            assert name == "__version__"
        else:  # pragma: no cover - only on an unexpected export
            pytest.fail(f"{name} is neither a wrapper class, a function nor a string")

    # The audit is only meaningful if it actually classifies things.
    assert set(wrappers) >= {
        "Document",
        "Element",
        "Namespace",
        "Node",
        "NodeId",
        "NodeKind",
        "QualifiedName",
        "ValidationError",
        "ValidationErrors",
    }
    assert set(functions) == {"parse", "parse_file", "write", "write_file"}
    # Nothing that the Rust spec layer or the native module keeps internal may appear here.
    for leaked in ("NCName", "Text", "Comment", "CData", "PiTarget", "PiData", "NodeContent"):
        assert leaked not in xml.__all__, f"{leaked} must not be part of the public surface"


def test_the_native_value_types_are_exactly_the_ones_we_decided_on() -> None:
    """The allow-list is a decision, so a change to it must be a change to this test."""
    native = {
        name
        for name in xml.__all__
        if isinstance(getattr(xml, name), type) and _is_native(getattr(xml, name))
    }
    assert native == NATIVE_RE_EXPORTS

    # The exception hierarchy is the documented one.
    assert issubclass(xml.XmlSyntaxError, xml.XmlError)
    assert issubclass(xml.XmlNamespaceError, xml.XmlError)
    assert issubclass(xml.XmlDocumentError, xml.XmlError)
    assert issubclass(xml.XmlIoError, xml.XmlError)
    assert issubclass(xml.XmlValidationError, xml.XmlError)
    assert issubclass(xml.XmlError, Exception)


def test_classes_are_constructible_or_produced_by_a_documented_factory() -> None:
    # Directly constructible.
    assert isinstance(xml.Document(), xml.Document)
    assert isinstance(xml.Document.empty(), xml.Document)
    assert isinstance(xml.Namespace("http://example.com"), xml.Namespace)
    assert isinstance(xml.Namespace("http://example.com", "ex"), xml.Namespace)
    assert isinstance(xml.QualifiedName("child"), xml.QualifiedName)
    assert isinstance(xml.WriteOptions(), xml.WriteOptions)
    assert isinstance(xml.XmlDeclaration.utf8(), xml.XmlDeclaration)
    assert xml.DeclarationStyle.Never is not None
    assert xml.EmptyElementStyle.SelfClosing is not None
    assert [member for member in ("Element", "Text", "Comment", "CData", "ProcessingInstruction")
            if hasattr(xml.NodeKind, member)] == [
        "Element", "Text", "Comment", "CData", "ProcessingInstruction"
    ]

    # Produced by the documented factories: these have no public constructor, exactly like their
    # Rust counterparts (an `Element` cannot exist without a document).
    document = xml.Document()
    root = document.create_element("root")
    document.set_root(root)
    assert isinstance(root, xml.Element)
    assert isinstance(root.node, xml.Node)
    assert isinstance(root.id, xml.NodeId)
    assert isinstance(root.document, xml.Document)
    text = document.create_text("text")
    root.append_child(text)
    assert isinstance(root.children()[0], xml.Node)
    assert isinstance(document.root, xml.Element)
    assert isinstance(text, xml.Node)
    assert not isinstance(text, xml.Element)

    # Validation results appear on both paths: as a list, and inside the raised exception.
    invalid = xml.Document()
    problem_list = invalid.validation_errors()
    assert isinstance(problem_list, xml.ValidationErrors)
    assert isinstance(problem_list, list)
    assert isinstance(problem_list[0], xml.ValidationError)
    with pytest.raises(xml.XmlValidationError):
        invalid.validate()
    assert set(problem_list.kinds()) == {"missing_root"}


def test_isinstance_works_for_the_wrapper_hierarchy() -> None:
    document = xml.Document()
    root = document.create_element("root")
    document.set_root(root)

    assert isinstance(root, xml.Element)
    assert isinstance(root, xml.Node)  # Element subclasses Node in the Python layer
    # `Element.node` returns the element itself (an element *is* a node), so this holds too; a text
    # node is the case where the two classes differ.
    assert isinstance(root.node, xml.Node)
    assert root.node is root
    text = document.create_text("x")
    assert isinstance(text, xml.Node)
    assert not isinstance(text, xml.Element)
    assert isinstance(document.root, xml.Element)
    assert isinstance(document, xml.Document)
    assert isinstance(xml.Namespace("u"), xml.Namespace)
    assert isinstance(xml.QualifiedName("n"), xml.QualifiedName)
    assert isinstance(root.qualified_name, xml.QualifiedName)
    assert isinstance(root.namespace, type(None))  # no namespace on a plain name
    assert isinstance(xml.parse("<a/>"), xml.Document)
    assert isinstance(xml.write(document), str)


def test_exported_functions_have_the_documented_signatures() -> None:
    assert list(inspect.signature(xml.parse).parameters) == ["source"]
    assert list(inspect.signature(xml.parse_file).parameters) == ["path"]
    assert list(inspect.signature(xml.write).parameters) == ["document", "options"]
    assert list(inspect.signature(xml.write_file).parameters) == ["document", "path", "options"]
    # And they are all actually usable.
    import tempfile
    from pathlib import Path

    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "doc.xml"
        xml.write_file(xml.parse("<a/>"), path)
        assert xml.write(xml.parse_file(path)) == "<a/>"


def test_argument_coercion_is_part_of_the_surface() -> None:
    """The three name spellings promised by the documentation all work, everywhere a name is
    taken."""
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
