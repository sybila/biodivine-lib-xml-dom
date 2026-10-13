"""The identifier of a node's arena slot."""

from __future__ import annotations

from . import _sys

#: The largest slot number a document can use (ids are unsigned 32-bit numbers).
_MAX_INDEX = 2**32 - 1


class NodeId:
    """The arena slot of a node, as an opaque identifier.

    An id on its own is *not* a usable reference. It can be compared, hashed, shown as an integer,
    and turned back into a node with :meth:`Document.node` - which returns ``None`` once the node it
    referred to has been reclaimed by the garbage collector and the slot has been reused. Node
    handles (see :class:`Node`) keep their node alive instead.

    An id can also be built from an integer, mirroring Rust's ``NodeId::from(u32)``::

        document.node(NodeId(7))   # the node in slot 7, if there is one

    Raises ``ValueError`` for an integer outside ``0 <= index < 2**32``.
    """

    __slots__ = ("_index",)

    def __init__(self, index: int) -> None:
        index = int(index)
        if not 0 <= index <= _MAX_INDEX:
            raise ValueError(f"a node id is between 0 and {_MAX_INDEX}, got {index}")
        self._index = index

    @classmethod
    def _wrap(cls, id: "_sys.NodeId") -> "NodeId":
        return cls(id.index())

    def _unwrap(self) -> "_sys.NodeId":
        return _sys.NodeId(self._index)

    @property
    def index(self) -> int:
        """The raw integer behind the id (its arena slot).

        Useful for logging and for round-tripping through :class:`NodeId`; it does not by itself
        guarantee that the slot still holds the same node.
        """
        return self._index

    def __index__(self) -> int:
        return self._index

    def __int__(self) -> int:
        return self._index

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, NodeId):
            return NotImplemented
        return self._index == other._index

    def __hash__(self) -> int:
        return hash(self._index)

    def __str__(self) -> str:
        return str(self._index)

    def __repr__(self) -> str:
        return f"NodeId({self._index})"
