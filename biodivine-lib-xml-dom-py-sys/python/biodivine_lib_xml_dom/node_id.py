"""Pythonic wrapper for a node id."""

from __future__ import annotations

from typing import Union

from . import _sys


class NodeId:
    """The arena slot of a node, as an opaque identifier.

    An id on its own is *not* a usable reference. It can be compared, hashed, shown as an integer,
    and turned back into a node with :meth:`Document.node` - which returns ``None`` once the node it
    referred to has been reclaimed by the garbage collector and the slot has been reused. Node
    handles (see :class:`Node`) keep their node alive instead.

    An id can also be built from an integer, mirroring Rust's ``NodeId::from(u32)``::

        document.node(NodeId(7))   # the node in slot 7, if there is one
    """

    __slots__ = ("_id",)

    def __init__(self, id: Union[int, "_sys.NodeId"]) -> None:
        if isinstance(id, _sys.NodeId):
            self._id = id
        else:
            self._id = _sys.NodeId(int(id))

    @classmethod
    def _wrap(cls, id: "_sys.NodeId") -> "NodeId":
        wrapper = cls.__new__(cls)
        wrapper._id = id
        return wrapper

    @property
    def index(self) -> int:
        """The raw integer behind the id (its arena slot).

        Useful for logging and for round-tripping through :class:`NodeId`; it does not by itself
        guarantee that the slot still holds the same node.
        """
        return self._id.index()

    def __index__(self) -> int:
        return self.index

    def __int__(self) -> int:
        return self.index

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, NodeId):
            return NotImplemented
        return self._id == other._id

    def __hash__(self) -> int:
        return hash(self._id)

    def __str__(self) -> str:
        return str(self.index)

    def __repr__(self) -> str:
        return f"NodeId({self.index})"
