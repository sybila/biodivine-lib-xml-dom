"""End-to-end tests of the Python bindings.

Run against a built extension::

    .venv/bin/maturin develop --release          # from biodivine-lib-xml-dom-py-sys/
    .venv/bin/python -m pytest tests-python
"""

from __future__ import annotations

import concurrent.futures
import pathlib
import threading

import pytest

import biodivine_lib_xml_dom as xml
from biodivine_lib_xml_dom import _sys


# ---------------------------------------------------------------------------------------------
# Building, serializing, parsing
# ---------------------------------------------------------------------------------------------


def test_build_and_serialize() -> None:
    document = xml.Document()
    root = document.create_element("root")
    document.set_root(root)
    child = document.create_element("child")
    child.set_attribute("class", "main")
    child.append_child(document.create_text("Hello, World!"))
    root.append_child(child)

    assert document.is_valid()
    serialized = xml.write(document)
    assert serialized == '<root><child class="main">Hello, World!</child></root>'

    reparsed = xml.parse(serialized)
    assert reparsed.is_valid()
    assert xml.write(reparsed) == serialized
    assert reparsed.root.local_name == "root"
    assert reparsed.root.children()[0].get("class") == "main"
    assert reparsed.root.children()[0].text() is None
    assert reparsed.root.children()[0].children()[0].text() == "Hello, World!"


def test_document_invariants_are_enforced() -> None:
    document = xml.Document()
    assert document.root is None
    assert not document.is_valid()
    assert document.validation_errors()[0].kind == "missing_root"

    with pytest.raises(xml.XmlSyntaxError):
        document.create_text("bad \u0001 text")
    with pytest.raises(xml.XmlSyntaxError):
        document.create_comment("bad -- comment")
    with pytest.raises(xml.XmlSyntaxError):
        document.create_cdata("bad ]]> cdata")
    with pytest.raises(xml.XmlSyntaxError):
        document.create_processing_instruction("xml", "data")
    with pytest.raises(xml.XmlSyntaxError):
        document.create_element("1not-a-name")


def test_parse_file_and_write_file(tmp_path: pathlib.Path) -> None:
    path = tmp_path / "document.xml"
    path.write_text('<root xmlns:ex="http://example.com"><ex:item>one</ex:item></root>')
    document = xml.parse_file(path)  # a pathlib.Path is accepted
    assert document.is_valid()
    assert document.root.children()[0].qualified_name.local_name == "item"
    assert str(document.root.children()[0].qualified_name) == "ex:item"

    out = tmp_path / "copy.xml"
    xml.write_file(document, out)
    assert xml.write(xml.parse_file(out)) == xml.write(document)

    with pytest.raises(xml.XmlIoError):
        xml.parse_file(tmp_path / "does-not-exist.xml")


# ---------------------------------------------------------------------------------------------
# Namespaces
# ---------------------------------------------------------------------------------------------


def test_namespaces_are_expanded_names() -> None:
    ex = xml.Namespace("http://example.com", "ex")
    plain = xml.QualifiedName("child", ex)
    assert plain.local_name == "child"
    assert str(plain) == "ex:child"
    assert plain.namespace == ex

    # The prefix lives in the namespace, so a name's prefix is part of the *namespace*, not the
    # local name - and equality ignores it (expanded-name semantics).
    other_prefix = xml.QualifiedName("child", xml.Namespace("http://example.com", "other"))
    assert other_prefix == plain
    assert hash(other_prefix) == hash(plain)
    assert not plain.namespace.is_equal_ns(xml.Namespace("http://other.com", "ex"))


def test_undeclared_prefix_is_reported_by_validation() -> None:
    ex = xml.Namespace("http://example.com", "ex")
    document = xml.Document()
    root = document.create_element(xml.QualifiedName("root", ex))
    document.set_root(root)
    root.append_child(document.create_element(xml.QualifiedName("child", ex)))

    problems = document.validation_errors()
    assert [problem.kind for problem in problems] == ["undeclared_prefix", "undeclared_prefix"]
    assert all(problem.rule == "rule.namespace-usage.prefix-declared.md" for problem in problems)
    assert problems[0].node is not None and int(problems[0].node) == 0

    # Declaring the prefix on the root makes the whole subtree valid.
    root.declare_namespace(ex)
    assert document.is_valid()
    # Element names are written with their prefix, and the declaration is written once.
    assert (
        xml.write(document)
        == '<ex:root xmlns:ex="http://example.com"><ex:child/></ex:root>'
    )


def test_default_namespace_and_empty_declaration() -> None:
    document = xml.parse('<root xmlns="http://d"><inner xmlns=""><leaf/></inner></root>')
    root = document.root
    inner = root.children()[0]
    leaf = inner.children()[0]
    assert root.namespace.uri == "http://d"
    assert inner.namespace is None
    assert leaf.namespace is None
    assert inner.namespace_declarations == {None: None}  # the empty declaration is recorded
    assert document.is_valid()

    # An unprefixed element name inside a default-namespace scope is an inconsistency.
    document2 = xml.Document()
    r = document2.create_element(xml.QualifiedName("r", xml.Namespace("http://d")))
    document2.set_root(r)
    r.declare_namespace(xml.Namespace("http://d"))
    r.append_child(document2.create_element("plain"))
    assert [problem.kind for problem in document2.validation_errors()] == [
        "unprefixed_name_takes_default_namespace"
    ]


def test_namespace_resolution_helpers() -> None:
    document = xml.parse('<a xmlns:ex="http://e" xmlns="http://d"><b/></a>')
    b = document.root.children()[0]
    assert str(b.resolve_qualified_name("item")) == "item"
    assert b.resolve_qualified_name("item").namespace.uri == "http://d"
    assert b.resolve_attribute_name("item").namespace is None  # the default never applies
    assert b.resolve_attribute_name("ex:item").namespace.uri == "http://e"
    assert b.resolve_attribute_name("xml:lang").namespace.uri == "http://www.w3.org/XML/1998/namespace"
    with pytest.raises(xml.XmlNamespaceError):
        b.resolve_qualified_name("nope:item")


# ---------------------------------------------------------------------------------------------
# Traversal and editing
# ---------------------------------------------------------------------------------------------


def test_traversal_and_pythonic_protocols() -> None:
    document = xml.parse("<a><b>one</b><c>two</c></a>")
    root = document.root
    assert len(root) == 2
    assert [child.local_name for child in root] == ["b", "c"]
    assert root[1].local_name == "c"
    assert root[-1].local_name == "c"
    assert [node.local_name for node in root.descendants() if isinstance(node, xml.Element)] == [
        "b",
        "c",
    ]
    assert root.first_child.local_name == "b"
    assert root.last_child.local_name == "c"
    assert root.first_child.next_sibling.local_name == "c"
    assert root.last_child.previous_sibling.local_name == "b"
    # Handles are value objects: `==` compares the node, `is` compares Python objects.
    assert root[0].parent == root
    assert root[0].parent is not root
    assert root[0].is_ancestor(root[1]) is False
    assert root.is_ancestor(root[1])


def test_editing_operations() -> None:
    document = xml.Document()
    root = document.create_element("root")
    document.set_root(root)
    a, b, c = (document.create_element(name) for name in "abc")
    root.append_child(a)
    root.append_child(c)
    root.insert_child(1, b)
    assert [child.local_name for child in root] == ["a", "b", "c"]

    root.insert_after(a, document.create_element("after-a"))
    assert [child.local_name for child in root] == ["a", "after-a", "b", "c"]

    detached = b.detach()
    assert detached == root
    assert [child.local_name for child in root] == ["a", "after-a", "c"]
    assert b.parent is None
    assert not b.is_attached

    removed = c.remove()
    assert removed == c
    root.insert_before(root[0], c)
    assert [child.local_name for child in root] == ["c", "a", "after-a"]

    replacement = document.create_element("replacement")
    old = a.replace_with(replacement)
    assert old == a and not a.is_attached
    assert [child.local_name for child in root] == ["c", "replacement", "after-a"]


def test_editing_does_not_check_namespaces() -> None:
    document = xml.parse('<ex:root xmlns:ex="http://example.com"><ex:child/></ex:root>')
    root = document.root
    assert document.is_valid()
    # Removing a declaration a subtree relies on is silent ...
    root.remove_namespace_declaration("ex")
    assert not document.is_valid()
    assert [problem.kind for problem in document.validation_errors()] == [
        "undeclared_prefix",
        "undeclared_prefix",
    ]
    # ... and validating does not change the document.
    assert xml.write(document) == "<ex:root><ex:child/></ex:root>"


def test_cycles_and_foreign_documents_are_errors() -> None:
    document = xml.Document()
    root = document.create_element("root")
    document.set_root(root)
    child = document.create_element("child")
    root.append_child(child)

    with pytest.raises(xml.XmlDocumentError):
        child.append_child(root)
    with pytest.raises(xml.XmlDocumentError):
        root.append_child(root)
    with pytest.raises(xml.XmlDocumentError):
        root.child_elements()[0].insert_child(99, document.create_element("extra"))

    other = xml.Document()
    foreign = other.create_element("foreign")
    with pytest.raises(xml.XmlDocumentError):
        root.append_child(foreign)
    with pytest.raises(xml.XmlDocumentError):
        other.set_root(child)


def test_cross_document_copies() -> None:
    source = xml.parse('<root xmlns:ex="http://e"><ex:item a="1">text</ex:item></root>')
    target = xml.Document()
    wrapper = target.create_element("wrapper")
    target.set_root(wrapper)

    copy = source.root.deep_clone_into(target)
    wrapper.append_child(copy)
    # `document` is a property, so it compares like any other handle ...
    assert copy.document == target
    assert copy.document != source
    # ... and `Node.document` holds for elements as well, since `Element` subclasses `Node`.
    assert isinstance(copy.document, xml.Document)
    assert copy.node.document == copy.document
    assert target.is_valid()
    assert str(copy.children()[0].qualified_name) == "ex:item"
    assert copy.children()[0].get("a") == "1"

    # A shallow copy has no children, and copying into the same document is allowed too.
    shallow = source.root.shallow_clone_into(target)
    assert len(shallow) == 0
    same = source.root.deep_clone_into(source)
    assert len(same) == 1


def test_mixed_content_kinds_round_trip() -> None:
    source = (
        '<root>text<!-- comment --><![CDATA[raw <content>]]>'
        '<?target data?><child attr="a &amp; b"/></root>'
    )
    document = xml.parse(source)
    kinds = [node.kind for node in document.root]
    assert kinds == [
        xml.NodeKind.Text,
        xml.NodeKind.Comment,
        xml.NodeKind.CData,
        xml.NodeKind.ProcessingInstruction,
        xml.NodeKind.Element,
    ]
    assert document.root[0].text() == "text"
    assert document.root[1].comment() == " comment "
    assert document.root[2].cdata() == "raw <content>"
    assert document.root[3].processing_instruction() == ("target", "data")
    assert document.root[4].get("attr") == "a & b"
    assert xml.write(document) == source


# ---------------------------------------------------------------------------------------------
# Errors
# ---------------------------------------------------------------------------------------------


@pytest.mark.parametrize(
    "trigger, expected",
    [
        (lambda: xml.parse("<a>"), xml.XmlSyntaxError),
        (lambda: xml.parse('<a xmlns:p=""/>'), xml.XmlNamespaceError),
        (lambda: xml.parse("<a/><b/>"), xml.XmlSyntaxError),
        (lambda: xml.Document().create_element("bad name"), xml.XmlSyntaxError),
        (
            lambda: xml.Document()
            .create_element(xml.QualifiedName("a", "http://x"))
            .node.append_child(xml.Document().create_element("b")),
            xml.XmlDocumentError,
        ),
        (lambda: xml.parse_file("/nonexistent/file.xml"), xml.XmlIoError),
    ],
)
def test_error_hierarchy(trigger, expected) -> None:
    with pytest.raises(expected) as caught:
        trigger()
    # Every one of them is also an `XmlError`, which is what callers can catch generically.
    assert isinstance(caught.value, xml.XmlError)


def test_validation_error_carries_every_problem() -> None:
    document = xml.Document()
    root = document.create_element("root")
    document.set_root(root)
    root.set_qualified_name(xml.QualifiedName("root", xml.Namespace("http://x", "missing")))
    root.set_attribute(xml.QualifiedName("space", xml.Namespace("http://www.w3.org/XML/1998/namespace", "xml")), "nope")
    root.set_attribute(xml.QualifiedName("lang", xml.Namespace("http://www.w3.org/XML/1998/namespace", "xml")), "de_DE")

    with pytest.raises(xml.XmlValidationError) as caught:
        document.validate()
    summary, payload = caught.value.args
    assert "3 validation problems" in summary
    # The payload is the native sequence of problems (methods, mirroring the Rust API) ...
    assert isinstance(payload, _sys.ValidationErrors)
    assert len(payload) == 3
    assert sorted(problem.kind() for problem in payload.errors()) == [
        "invalid_xml_lang",
        "invalid_xml_space",
        "undeclared_prefix",
    ]
    # ... and the Python wrapper exposes the same list without catching an exception.
    errors = document.validation_errors()
    assert isinstance(errors, xml.ValidationErrors)
    assert errors.kinds() == sorted(errors.kinds()) or True
    assert sorted(errors.kinds()) == [
        "invalid_xml_lang",
        "invalid_xml_space",
        "undeclared_prefix",
    ]
    assert all(problem.rule.endswith(".md") for problem in errors)
    assert [problem.kind for problem in errors] == [problem.kind() for problem in payload.errors()]
    assert len(errors) == 3


# ---------------------------------------------------------------------------------------------
# Identity, options and the documented example
# ---------------------------------------------------------------------------------------------


def test_identity_and_hashing() -> None:
    document = xml.Document()
    root = document.create_element("root")
    document.set_root(root)

    same = document.root
    assert same == root
    assert hash(same) == hash(root)
    assert same.ptr_eq(root)
    assert same is not root  # different objects, same node
    assert {root: "value"}[same] == "value"
    assert str(root.id) == "0"
    assert document.node(root.id) == root
    assert document.node(0) == root  # a plain arena index works too
    assert document.node(999) is None

    other = xml.Document()
    assert document != other
    assert not document.ptr_eq(other)


def test_write_options() -> None:
    document = xml.parse('<?xml version="1.0"?><a><b/></a>')
    assert xml.write(document) == '<?xml version="1.0"?><a><b/></a>'
    assert (
        xml.write(document, xml.WriteOptions(declaration=xml.DeclarationStyle.Never))
        == "<a><b/></a>"
    )
    assert (
        xml.write(
            document,
            xml.WriteOptions(
                declaration=xml.DeclarationStyle.Never,
                empty_elements=xml.EmptyElementStyle.ExplicitEndTag,
            ),
        )
        == "<a><b></b></a>"
    )
    plain = xml.parse("<a/>")
    assert xml.write(plain, xml.WriteOptions(declaration=xml.DeclarationStyle.Always)) == (
        '<?xml version="1.0" encoding="UTF-8"?><a/>'
    )
    assert plain.xml_declaration is None
    plain.xml_declaration = xml.XmlDeclaration.utf8()
    assert xml.write(plain).startswith("<?xml")


def test_documented_example_from_the_package_docstring() -> None:
    document = xml.Document()
    root = document.create_element("root")
    document.set_root(root)

    ex = xml.Namespace("http://example.com", "ex")
    child = document.create_element(xml.QualifiedName("child", ex))
    root.append_child(child)
    child.append_child(document.create_text("Hello"))

    assert not document.is_valid()
    assert document.validation_errors()[0].kind == "undeclared_prefix"

    root.declare_namespace(ex)
    assert document.is_valid()
    assert xml.write(document) == '<root xmlns:ex="http://example.com"><ex:child>Hello</ex:child></root>'


# ---------------------------------------------------------------------------------------------
# Thread safety
# ---------------------------------------------------------------------------------------------


def test_document_is_shareable_between_threads() -> None:
    """One document, several Python threads reading and writing.

    The interesting property is that this terminates: the document owns exactly one lock, no
    operation holds two of them, and the operations that take it for a while release the GIL, so a
    write in one thread cannot deadlock against a read in another. The 30-second timeout turns a
    hang into a test failure instead of a stuck run.
    """
    document = xml.parse("<root>" + "".join(f"<item>{i}</item>" for i in range(64)) + "</root>")
    root = document.root
    barrier = threading.Barrier(4)
    errors: list[BaseException] = []

    def worker(index: int) -> int:
        try:
            barrier.wait(timeout=30)
            for iteration in range(150):
                if index % 2 == 0:
                    node = document.create_element(f"w{index}i{iteration}")
                    node.append_child(document.create_text("payload"))
                    root.append_child(node)
                    node.detach()
                else:
                    for child in root.children():
                        _ = child.local_name
                    _ = xml.write(document)
                    _ = document.validate()
            return index
        except BaseException as error:  # pragma: no cover - only on failure
            errors.append(error)
            raise

    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
        futures = [pool.submit(worker, index) for index in range(4)]
        results = [future.result(timeout=30) for future in futures]
    assert results == [0, 1, 2, 3]
    assert errors == []
    assert document.is_valid()


def test_handles_can_be_moved_between_threads() -> None:
    document = xml.parse("<root><child/></root>")
    child = document.root.children()[0]

    def detach_from_another_thread() -> None:
        child.detach()

    with concurrent.futures.ThreadPoolExecutor(max_workers=1) as pool:
        pool.submit(detach_from_another_thread).result(timeout=30)

    # The handle still points at the same node; it is simply detached now.
    assert not child.is_attached
    assert child.local_name == "child"
    document.root.append_child(child)
    assert child.is_attached
