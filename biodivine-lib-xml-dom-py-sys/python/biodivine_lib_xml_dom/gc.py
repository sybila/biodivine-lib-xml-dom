"""Pythonic wrapper for the result of a garbage collection."""

from __future__ import annotations

from . import _sys


class GcStats:
    """What one run of the garbage collector did.

    A snapshot of the arena at the moment the collection finished. :meth:`Document.collect_garbage`
    returns one of these; ``collected`` is what the run reclaimed, ``live`` and ``free`` describe the
    arena afterwards, and ``capacity`` is how many nodes the arena may hold before it is considered
    full.
    """

    __slots__ = ("_stats",)

    def __init__(self, stats: "_sys.GcStats") -> None:
        self._stats = stats

    @classmethod
    def _wrap(cls, stats: "_sys.GcStats") -> "GcStats":
        wrapper = cls.__new__(cls)
        wrapper._stats = stats
        return wrapper

    @property
    def capacity(self) -> int:
        """The arena capacity after the collection, in slots."""
        return self._stats.capacity

    @property
    def live(self) -> int:
        """The number of nodes the document holds after the collection."""
        return self._stats.live

    @property
    def free(self) -> int:
        """The number of nodes that can be created without growing the arena."""
        return self._stats.free

    @property
    def collected(self) -> int:
        """The number of nodes this collection reclaimed."""
        return self._stats.collected

    def free_fraction(self) -> float:
        """The fraction of the capacity that is free, in ``[0.0, 1.0]``."""
        return self._stats.free_fraction()

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, GcStats):
            return NotImplemented
        return self._stats == other._stats

    def __repr__(self) -> str:
        return (
            f"GcStats(capacity={self.capacity}, live={self.live}, "
            f"free={self.free}, collected={self.collected})"
        )
