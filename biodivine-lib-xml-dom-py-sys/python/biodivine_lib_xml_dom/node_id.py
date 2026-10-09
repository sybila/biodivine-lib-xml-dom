"""Pythonic wrapper for a node id."""

from __future__ import annotations

from . import _sys


class NodeId:
    """The index of a node in its document's arena.

    Ids are opaque, exactly as in Rust: the only things you can do with one are compare it, hash it,
    use it as an integer index for display, and look the node up again with
    :meth:`Document.node`.
    """

    __slots__ = ("_id",)

    def __init__(self, id: "_sys.NodeId") -> None:
        self._id = id

    @classmethod
    def _wrap(cls, id: "_sys.NodeId") -> "NodeId":
        wrapper = cls.__new__(cls)
        wrapper._id = id
        return wrapper

    @property
    def index(self) -> int:
        """The zero-based index of the node in its document's arena."""
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
