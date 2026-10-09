"""Pythonic wrapper for a node handle."""

from __future__ import annotations

from typing import TYPE_CHECKING, Iterator, List, Optional, Union

from . import _convert, _sys
from .node_id import NodeId

if TYPE_CHECKING:  # pragma: no cover - import cycle is broken on purpose
    from .document import Document


class NodeKind:
    """The kind of a node.

    A thin namespace over the native enum, so ``biodivine_lib_xml_dom.NodeKind.Element`` and
    ``node.kind == NodeKind.Element`` work in Python.
    """

    Element = _sys.NodeKind.Element
    Text = _sys.NodeKind.Text
    Comment = _sys.NodeKind.Comment
    CData = _sys.NodeKind.CData
    ProcessingInstruction = _sys.NodeKind.ProcessingInstruction


class Node:
    """A handle to one node: an element, text, comment, CDATA section or processing instruction.

    The handle is cheap to copy, stays valid even when the node is detached, and can be passed
    between threads. Cloning the handle (``node`` or ``copy.copy(node)``) refers to the *same* node;
    use :meth:`deep_clone`/:meth:`shallow_clone` to duplicate the node itself.

    Pythonic extras over the Rust API: children can be iterated and indexed directly
    (``for child in node``, ``node[0]``, ``len(node)``), and the content accessors
    (:meth:`text`, :meth:`comment`, :meth:`cdata`, :meth:`processing_instruction`) return plain
    ``str`` instead of a wrapper type.
    """

    __slots__ = ("_node",)

    def __init__(self, node: Union["Node", "_sys.Node"]) -> None:
        self._node = node._node if isinstance(node, Node) else node

    @classmethod
    def _wrap(cls, node: "_sys.Node") -> "Node":
        """Wraps a native node in the most specific Python class available."""
        element = node.as_element()
        if element is not None:
            from .element import Element

            return Element._wrap(element, node)
        wrapper = cls.__new__(cls)
        wrapper._node = node
        return wrapper

    # -- identity -------------------------------------------------------------------------
    @property
    def id(self) -> NodeId:
        """The arena id of this node (meaningful only together with its document)."""
        return NodeId._wrap(self._node.id())

    @property
    def document(self) -> "Document":
        """The document this node belongs to.

        A property (like :attr:`id`, :attr:`kind`, :attr:`parent` and the sibling accessors), so
        ``node.document == document`` works; the native ``_sys.Node.document()`` stays a method
        because there the point is to mirror the Rust API.
        """
        from .document import Document

        return Document._wrap(self._node.document())

    def ptr_eq(self, other: "Node") -> bool:
        """Whether two handles refer to the same node of the same document."""
        return self._node.ptr_eq(_convert.unwrap_node(other))

    def __eq__(self, other: object) -> bool:
        if isinstance(other, Node):
            return self.ptr_eq(other)
        return NotImplemented

    def __hash__(self) -> int:
        return hash(self._node)

    # -- classification -------------------------------------------------------------------
    @property
    def kind(self) -> "_sys.NodeKind":
        """The kind of this node."""
        return self._node.kind()

    def as_element(self) -> Optional["Element"]:
        """This node as an :class:`Element`, or ``None`` if it has a different kind."""
        from .element import Element

        inner = self._node.as_element()
        return None if inner is None else Element._wrap(inner)

    @property
    def is_attached(self) -> bool:
        """Whether the node is reachable from the document root."""
        return self._node.is_attached()

    def is_ancestor(self, other: "Node") -> bool:
        """Whether this node is a strict ancestor of ``other``."""
        return self._node.is_ancestor(_convert.unwrap_node(other))

    # -- traversal ------------------------------------------------------------------------
    @property
    def parent(self) -> Optional["Node"]:
        """The parent node, or ``None`` for a detached node or the document root."""
        return _convert.optional(self._node.parent())

    @property
    def first_child(self) -> Optional["Node"]:
        """The first child, if any."""
        return _convert.optional(self._node.first_child())

    @property
    def last_child(self) -> Optional["Node"]:
        """The last child, if any."""
        return _convert.optional(self._node.last_child())

    @property
    def next_sibling(self) -> Optional["Node"]:
        """The next sibling, if any."""
        return _convert.optional(self._node.next_sibling())

    @property
    def previous_sibling(self) -> Optional["Node"]:
        """The previous sibling, if any."""
        return _convert.optional(self._node.previous_sibling())

    @property
    def index_in_parent(self) -> Optional[int]:
        """The index in the parent's child list, if there is a parent."""
        return self._node.index_in_parent()

    def children(self) -> List["Node"]:
        """The children, in document order."""
        return _convert.wrap_many(self._node.children())

    def child_elements(self) -> List["Element"]:
        """The element children, in document order."""
        from .element import Element

        return [Element._wrap(element) for element in self._node.child_elements()]

    def descendants(self) -> List["Node"]:
        """All descendants, in document order."""
        return _convert.wrap_many(self._node.descendants())

    def __iter__(self) -> Iterator["Node"]:
        return iter(self.children())

    def __len__(self) -> int:
        return len(self._node.children())

    def __getitem__(self, index: Union[int, slice]) -> Union["Node", List["Node"]]:
        return self.children()[index]

    # -- content --------------------------------------------------------------------------
    def text(self) -> Optional[str]:
        """The content as text, or ``None`` if this is not a text node."""
        return self._node.text()

    def comment(self) -> Optional[str]:
        """The content as a comment, or ``None``."""
        return self._node.comment()

    def cdata(self) -> Optional[str]:
        """The content as CDATA, or ``None``."""
        return self._node.cdata()

    def processing_instruction(self) -> Optional[tuple]:
        """The ``(target, data)`` pair, or ``None``."""
        return self._node.processing_instruction()

    # -- editing --------------------------------------------------------------------------
    def detach(self) -> Optional["Node"]:
        """Detaches this node from its parent and returns the previous parent."""
        return _convert.optional(self._node.detach())

    def remove(self) -> "Node":
        """Detaches this node and returns the node itself."""
        return Node._wrap(self._node.remove())

    def append_child(self, child: Union["Node", "Element"]) -> None:
        """Appends ``child``, moving it if it is already attached.

        Raises `XmlDocumentError` for a foreign document, a cycle or an attempt to attach the root.
        No namespace declaration is touched: run :meth:`Document.validate` afterwards.
        """
        self._node.append_child(_convert.unwrap_node(child))

    def insert_child(self, index: int, child: Union["Node", "Element"]) -> None:
        """Inserts ``child`` as the ``index``-th child (interpreted after detaching it)."""
        self._node.insert_child(index, _convert.unwrap_node(child))

    def insert_before(self, sibling: Union["Node", "Element"], child: Union["Node", "Element"]) -> None:
        """Inserts ``child`` directly before ``sibling``."""
        self._node.insert_before(_convert.unwrap_node(sibling), _convert.unwrap_node(child))

    def insert_after(self, sibling: Union["Node", "Element"], child: Union["Node", "Element"]) -> None:
        """Inserts ``child`` directly after ``sibling``."""
        self._node.insert_after(_convert.unwrap_node(sibling), _convert.unwrap_node(child))

    def replace_with(self, replacement: Union["Node", "Element"]) -> "Node":
        """Replaces this node with ``replacement`` and returns this node (now detached)."""
        return Node._wrap(self._node.replace_with(_convert.unwrap_node(replacement)))

    # -- cloning --------------------------------------------------------------------------
    def shallow_clone(self) -> "Node":
        """A detached copy of this node without children, in the same document."""
        return Node._wrap(self._node.shallow_clone())

    def deep_clone(self) -> "Node":
        """A detached deep copy of this subtree, in the same document."""
        return Node._wrap(self._node.deep_clone())

    def shallow_clone_into(self, target: "Document") -> "Node":
        """A detached copy of this node without children, in ``target``."""
        return Node._wrap(self._node.shallow_clone_into(target._document))

    def deep_clone_into(self, target: "Document") -> "Node":
        """A detached deep copy of this subtree in ``target``.

        This is the way to move a tree between documents (attaching a node to a parent of another
        document is an error). The copy is a point-in-time snapshot.
        """
        return Node._wrap(self._node.deep_clone_into(target._document))

    def belongs_to(self, document: "Document") -> bool:
        """Whether this node belongs to ``document``."""
        return self._node.belongs_to(document._document)

    def __str__(self) -> str:
        return self._node.__str__()

    def __repr__(self) -> str:
        return repr(self._node)


_convert.register(_sys.Node, Node)
