"""The result of a garbage collection."""

from __future__ import annotations

from dataclasses import dataclass

from . import _sys


@dataclass(frozen=True)
class GcStats:
    """What one run of the garbage collector did.

    A snapshot of the arena at the moment the collection finished. :meth:`Document.collect_garbage`
    returns one of these; ``collected`` is what the run reclaimed, ``live`` and ``free`` describe the
    arena afterwards, and ``capacity`` is how many nodes the arena may hold before it is considered
    full.
    """

    capacity: int
    """The arena capacity after the collection, in slots."""
    live: int
    """The number of nodes the document holds after the collection."""
    free: int
    """The number of nodes that can be created without growing the arena."""
    collected: int
    """The number of nodes this collection reclaimed."""

    @classmethod
    def _wrap(cls, stats: "_sys.GcStats") -> "GcStats":
        return cls(stats.capacity, stats.live, stats.free, stats.collected)

    def free_fraction(self) -> float:
        """The fraction of the capacity that is free, in ``[0.0, 1.0]``."""
        return 0.0 if self.capacity == 0 else self.free / self.capacity
