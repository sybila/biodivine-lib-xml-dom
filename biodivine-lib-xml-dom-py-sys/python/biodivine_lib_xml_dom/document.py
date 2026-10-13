"""Pythonic wrapper for a document handle."""

from __future__ import annotations

import os
from typing import Any, List, Optional, Union

from . import _convert, _sys
from .element import Element, NameLike
from .gc import GcStats
from .node import Node
from .node_id import NodeId
from .validation import ValidationError, ValidationErrors
from .write_options import WriteOptions, XmlDeclaration


class Document:
    """An XML document.

    ``Document()`` creates an empty document; :meth:`parse` and :meth:`parse_file` read one.

    A document is a cheap, thread-safe handle to shared state: copying it with ``copy.copy`` or
    passing it to another thread does not copy the tree, and all node data lives behind a single
    lock that every method acquires and releases. Because there is exactly one lock and it is never
    re-acquired while held, no operation can deadlock - from Python just as from Rust.
    ``copy.deepcopy`` creates an independent document with a copy of the tree.

    Names created through the API are validated at construction, so an invalid name, text, comment,
    CDATA section or processing instruction raises immediately instead of producing a broken
    document. Namespace consistency is *not* checked while editing (that would make every move pay
    for a whole-document scan); call :meth:`validate` when you want to know.
    """

    __slots__ = ("_document",)

    def __init__(self) -> None:
        self._document = _sys.Document()

    @classmethod
    def _wrap(cls, document: "_sys.Document") -> "Document":
        wrapper = object.__new__(cls)
        wrapper._document = document
        return wrapper

    # -- parsing and serialization ---------------------------------------------------------
    @classmethod
    def parse(cls, source: Union[str, bytes, bytearray, memoryview]) -> "Document":
        """Parses an XML document from a string, or from bytes that must be valid UTF-8.

        Raises `XmlSyntaxError` for malformed input and `XmlNamespaceError` for an illegal use of a
        namespace.
        """
        if isinstance(source, str):
            return cls._wrap(_sys.Document.parse_string(source))
        if isinstance(source, (bytes, bytearray, memoryview)):
            return cls._wrap(_sys.Document.parse_bytes(bytes(source)))
        raise TypeError(f"expected str or bytes, got {type(source).__name__}")

    @classmethod
    def parse_file(cls, path: Union[str, "os.PathLike[str]"]) -> "Document":
        """Parses an XML document from a file (``str``, ``pathlib.Path``, or any ``os.PathLike``).

        Raises `XmlIoError` if the file cannot be read, and the errors of :meth:`parse`.
        """
        return cls._wrap(_sys.Document.parse_file(_convert.to_path(path)))

    def write(self, options: Optional[WriteOptions] = None) -> str:
        """Serializes this document into a string.

        ``options`` selects the declaration and empty-element style; the defaults reproduce a parsed
        document as closely as the data model allows. The output of a valid document parses back
        into an equal tree; an invalid one is written as it is.
        """
        return self._document.write_string(None if options is None else options._unwrap())

    def write_file(
        self,
        path: Union[str, "os.PathLike[str]"],
        options: Optional[WriteOptions] = None,
    ) -> None:
        """Writes this document to a file (created or truncated).

        Raises `XmlIoError` if the file cannot be written. If the write fails or is interrupted,
        the file may already contain part of the document.
        """
        self._document.write_file(
            _convert.to_path(path), None if options is None else options._unwrap()
        )

    # -- root -----------------------------------------------------------------------------
    @property
    def root(self) -> Optional[Element]:
        """The root element, or ``None``."""
        inner = self._document.root()
        return None if inner is None else Element._wrap(inner)

    def set_root(self, root: Element) -> Optional[Element]:
        """Sets the root element and returns the previous root (if any).

        The previous root is kept in the arena as a detached subtree. Raises `XmlDocumentError` if
        ``root`` belongs to another document or is already attached to a parent.
        """
        previous = self._document.set_root(_convert.unwrap_element(root))
        return None if previous is None else Element._wrap(previous)

    def clear_root(self) -> Optional[Element]:
        """Removes the root element and returns it (the subtree is preserved, detached)."""
        inner = self._document.clear_root()
        return None if inner is None else Element._wrap(inner)

    # -- node creation --------------------------------------------------------------------
    def create_element(self, name: NameLike) -> Element:
        """Creates a new, detached element with the given expanded name.

        The name may be given in four ways::

            document.create_element("child")                              # no namespace
            document.create_element(("child", "http://example.com"))      # default namespace
            document.create_element(("child", Namespace(uri, "ex")))      # prefix + URI
            document.create_element(QualifiedName("child", Namespace(...)))

        The same spellings are accepted by every other argument that takes a name
        (:meth:`Element.set_attribute`, :meth:`Element.attribute`, :meth:`Element.get`, ...).
        """
        return Element._wrap(self._document.create_element(_convert.unwrap_name(name)))

    def create_text(self, text: str) -> Node:
        """Creates a new, detached text node.

        Raises `XmlSyntaxError` if the content is not legal XML text.
        """
        return Node._wrap(self._document.create_text(text))

    def create_comment(self, text: str) -> Node:
        """Creates a new, detached comment node.

        Raises `XmlSyntaxError` if the content contains ``--`` or ends with ``-``.
        """
        return Node._wrap(self._document.create_comment(text))

    def create_cdata(self, text: str) -> Node:
        """Creates a new, detached CDATA section.

        Raises `XmlSyntaxError` if the content contains ``]]>``.
        """
        return Node._wrap(self._document.create_cdata(text))

    def create_processing_instruction(self, target: str, data: str) -> Node:
        """Creates a new, detached processing instruction.

        Raises `XmlSyntaxError` if the target is not a valid XML name or matches ``xml``
        case-insensitively, or if the content contains ``?>``.
        """
        return Node._wrap(self._document.create_processing_instruction(target, data))

    def create_element_tree(self, name: NameLike, *children: Node) -> Element:
        """Creates an element and appends ``children`` to it, returning the element.

        A small convenience for the very common "build a parent with children" step.
        """
        element = self.create_element(name)
        for child in children:
            element.append_child(child)
        return element

    # -- declaration ----------------------------------------------------------------------
    @property
    def xml_declaration(self) -> Optional[XmlDeclaration]:
        """The ``<?xml ...?>`` declaration of the document, or ``None``. Assignable."""
        inner = self._document.xml_declaration()
        return None if inner is None else XmlDeclaration._wrap(inner)

    @xml_declaration.setter
    def xml_declaration(self, declaration: Optional[XmlDeclaration]) -> None:
        self._document.set_xml_declaration(
            None if declaration is None else declaration._declaration
        )

    # -- validation -----------------------------------------------------------------------
    def validate(self) -> None:
        """Checks the whole document and raises :class:`XmlValidationError` with **every** problem.

        The exception's ``errors`` attribute is a :class:`ValidationErrors` list::

            try:
                document.validate()
            except XmlValidationError as error:
                for problem in error.errors:
                    print(problem.kind, problem.message)

        Use :meth:`validation_errors` if you would rather get the list without an exception.
        """
        try:
            self._document.validate()
        except _sys.XmlValidationError as native:
            problems = ValidationErrors(ValidationError._wrap(problem) for problem in native.args[1])
            error = _sys.XmlValidationError(native.args[0], problems)
            error.errors = problems
            raise error from None

    def validation_errors(self) -> ValidationErrors:
        """Every problem found by whole-document validation, as a list (empty when valid)."""
        return ValidationErrors(
            ValidationError._wrap(problem) for problem in self._document.validation_errors()
        )

    def is_valid(self) -> bool:
        """Whether the document passes :meth:`validate`."""
        return self._document.is_valid()

    # -- nodes ----------------------------------------------------------------------------
    @property
    def node_count(self) -> int:
        """The number of nodes the document currently holds (attached and detached alike).

        Nodes that the garbage collector has reclaimed are not counted, so this number can go down.
        """
        return self._document.node_count()

    def nodes(self) -> List[Node]:
        """All nodes the document currently holds, in slot order.

        The returned handles keep their nodes alive for as long as the list exists; prefer walking
        from :attr:`root` when you only need the tree.
        """
        return _convert.wrap_many(self._document.nodes())

    def node(self, id: Union[NodeId, int]) -> Optional[Node]:
        """Looks up a node by id, or returns ``None`` if no node lives there.

        Accepts a :class:`NodeId` or a plain integer slot number. ``None`` means that the id was
        never handed out by this document or that the node it referred to has since been reclaimed
        by the garbage collector, which is handy when reading an id out of a validation problem:
        ``document.node(problem.node)``.
        """
        node_id = id if isinstance(id, NodeId) else NodeId(id)
        return _convert.optional(self._document.node(node_id._unwrap()))

    def collect_garbage(self) -> GcStats:
        """Reclaims every node that no handle and no root can reach, and reports what happened.

        The collector also runs automatically when the arena is full, so this is for bounding memory
        earlier or for observing what is still alive::

            document = Document()
            temporary = document.create_text("scratch")
            assert document.collect_garbage().collected == 0   # the handle keeps it alive
            del temporary
            assert document.collect_garbage().collected == 1   # now it is gone
        """
        return GcStats._wrap(self._document.collect_garbage())

    @property
    def capacity(self) -> int:
        """The number of nodes the document may hold before its arena is considered full.

        The arena grows geometrically and never shrinks, so this is a high-water mark rather than the
        current memory usage; use :attr:`node_count` for the number of nodes.
        """
        return self._document.capacity()

    # -- copying and identity --------------------------------------------------------------
    def ptr_eq(self, other: "Document") -> bool:
        """Whether two handles refer to the same document."""
        return self._document.ptr_eq(_convert.unwrap_document(other))

    def __copy__(self) -> "Document":
        """Another handle to the *same* document (no tree is copied)."""
        return self

    def __deepcopy__(self, memo: Any) -> "Document":
        """An independent document with a copy of the declaration and of the tree.

        Detached nodes are not part of the tree and are not copied.
        """
        copy = Document()
        copy.xml_declaration = self.xml_declaration
        root = self.root
        if root is not None:
            copy.set_root(root.deep_clone_into(copy))
        return copy

    def __eq__(self, other: object) -> bool:
        if isinstance(other, Document):
            return self.ptr_eq(other)
        return NotImplemented

    def __hash__(self) -> int:
        return hash(self._document)

    def __repr__(self) -> str:
        return f"Document(nodes={self.node_count}, root={self.root!r})"


_convert.register(_sys.Document, Document)
