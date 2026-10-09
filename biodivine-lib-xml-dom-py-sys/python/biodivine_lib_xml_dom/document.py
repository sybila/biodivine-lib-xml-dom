"""Pythonic wrapper for a document handle."""

from __future__ import annotations

from typing import TYPE_CHECKING, List, Optional, Union

from . import _convert, _sys
from .element import Element
from .name import QualifiedName
from .node import Node, NodeKind
from .node_id import NodeId
from .validation import ValidationErrors
from .write_options import WriteOptions, XmlDeclaration

if TYPE_CHECKING:  # pragma: no cover
    from ._convert import PathLike


class Document:
    """An XML document.

    A document is a cheap, thread-safe handle to shared state: copying it or passing it to another
    thread does not copy the tree, and all node data lives behind a single lock that every method
    acquires and releases. Because there is exactly one lock and it is never re-acquired while held,
    no operation can deadlock - from Python just as from Rust.

    Names created through the API are validated at construction, so an invalid name, text, comment,
    CDATA section or processing instruction raises immediately instead of producing a broken
    document. Namespace consistency is *not* checked while editing (that would make every move pay
    for a whole-document scan); call :meth:`validate` when you want to know.
    """

    __slots__ = ("_document",)

    def __init__(self, document: Optional["_sys.Document"] = None) -> None:
        self._document = _sys.Document() if document is None else document

    @classmethod
    def _wrap(cls, document: "_sys.Document") -> "Document":
        wrapper = cls.__new__(cls)
        wrapper._document = document
        return wrapper

    @classmethod
    def empty(cls) -> "Document":
        """Creates a new, empty document (no root element)."""
        return cls()

    # -- root -----------------------------------------------------------------------------
    @property
    def root(self) -> Optional[Element]:
        """The root element, or ``None``."""
        inner = self._document.root()
        return None if inner is None else Element._wrap(inner)

    def set_root(self, root: Union[Element, Node]) -> Optional[Element]:
        """Sets the root element and returns the previous root (if any).

        The previous root is kept in the arena as a detached subtree. Raises `XmlDocumentError` if
        ``root`` belongs to another document or is already attached to a parent.
        """
        element = _convert.unwrap_element(root)
        previous = self._document.set_root(element)
        return None if previous is None else Element._wrap(previous)

    def clear_root(self) -> Optional[Element]:
        """Removes the root element and returns it (the subtree is preserved, detached)."""
        inner = self._document.clear_root()
        return None if inner is None else Element._wrap(inner)

    # -- node creation --------------------------------------------------------------------
    def create_element(self, name: Union[str, QualifiedName, tuple]) -> Element:
        """Creates a new, detached element with the given expanded name.

        The name may be given in three ways::

            document.create_element("child")                              # no namespace
            document.create_element(("child", "http://example.com"))      # default namespace
            document.create_element(("child", Namespace(uri, "ex")))      # prefix + URI
            document.create_element(QualifiedName("child", Namespace(...)))

        The same three spellings are accepted by every other argument that takes a name
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

    def create_element_tree(self, name: Union[str, QualifiedName], *children: Union[Node, Element]) -> Element:
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
        """The ``<?xml ...?>`` declaration of the document, or ``None``."""
        return self._document.xml_declaration()

    @xml_declaration.setter
    def xml_declaration(self, declaration: Optional[XmlDeclaration]) -> None:
        self._document.set_xml_declaration(declaration)

    def set_xml_declaration(self, declaration: Optional[XmlDeclaration]) -> None:
        """Replaces the XML declaration of the document."""
        self._document.set_xml_declaration(declaration)

    # -- validation -----------------------------------------------------------------------
    def validate(self) -> None:
        """Checks the whole document and raises `XmlValidationError` with **every** problem found.

        The exception carries a :class:`ValidationErrors` sequence in its first argument::

            try:
                document.validate()
            except XmlValidationError as error:
                for problem in error.args[1]:
                    print(problem.kind, problem.message)

        Use :meth:`validation_errors` if you would rather get the list without an exception.
        """
        self._document.validate()

    def validation_errors(self) -> ValidationErrors:
        """Every problem found by whole-document validation, as a list (empty when valid)."""
        return ValidationErrors(
            [
                __import__("biodivine_lib_xml_dom.validation", fromlist=["ValidationError"]).ValidationError(
                    error
                )
                for error in self._document.validation_errors()
            ]
        )

    def is_valid(self) -> bool:
        """Whether the document passes :meth:`validate`."""
        return self._document.is_valid()

    # -- nodes ----------------------------------------------------------------------------
    @property
    def node_count(self) -> int:
        """The number of arena slots used (attached and detached nodes alike)."""
        return self._document.node_count()

    def nodes(self) -> List[Node]:
        """All nodes of this document, attached and detached alike, in creation order."""
        return _convert.wrap_many(self._document.nodes())

    def node(self, id: Union[NodeId, int]) -> Optional[Node]:
        """Looks up a node by id, or returns ``None`` if the id does not belong to this document.

        Accepts a :class:`NodeId` (the precise form) or a plain arena index, which is handy when
        reading an id out of a validation problem: ``document.node(int(problem.node))``.
        """
        if isinstance(id, NodeId):
            return _convert.optional(self._document.node(id._id))
        index = int(id)
        nodes = self.nodes()
        return nodes[index] if 0 <= index < len(nodes) else None

    # -- copying and identity --------------------------------------------------------------
    def copy(self) -> "Document":
        """Returns another handle to the *same* document (cheap; no tree is copied)."""
        return Document._wrap(self._document)

    def ptr_eq(self, other: "Document") -> bool:
        """Whether two handles refer to the same document."""
        return self._document.ptr_eq(other._document)

    def __copy__(self) -> "Document":
        return self.copy()

    def __deepcopy__(self, memo: dict) -> "Document":
        return self.copy()

    def __eq__(self, other: object) -> bool:
        if isinstance(other, Document):
            return self.ptr_eq(other)
        return NotImplemented

    def __hash__(self) -> int:
        return hash(self._document)

    def __repr__(self) -> str:
        return f"Document(nodes={self.node_count}, root={self.root!r})"


_convert.register(_sys.Document, Document)
