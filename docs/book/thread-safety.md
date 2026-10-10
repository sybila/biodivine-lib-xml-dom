# Thread safety

Why a document can be shared between threads, what that means for the operations you write, and
which parts of the locking behaviour you can rely on. The model it rests on is described in
[Building documents](building-documents.md), and the Python side of it in
[Using the Python package](python-usage.md).

## One lock per document

Every document owns exactly one reader-writer lock, and every operation acquires it once, does its
work and releases it. Internal code never acquires a second lock — not even for the operations that
touch two documents, which snapshot one document before locking the other. That is the whole
deadlock argument: there is only one lock, and it is never re-entered or held while another is
taken.

In debug builds a re-entrancy guard turns that claim into a panic rather than a hang if it is ever
violated.

## Sharing a document

Handles are cheap and thread-safe: pass the document, or a node, to another thread and work there.
Reads run concurrently with each other; a read never sees a half-finished edit, because each
operation is a single critical section.

::::{tab-set}
:::{tab-item} Rust
:sync: rust
```{literalinclude} ../../examples/book_threads.rs
:language: rust
:start-after: ANCHOR: threads
:end-before: ANCHOR_END: threads
:dedent:
```
:::
:::{tab-item} Python
:sync: python
```{literalinclude} examples/python/threads.py
:language: python
:start-after: ANCHOR: threads
:end-before: ANCHOR_END: threads
:dedent:
```
:::
::::

## Readers in parallel

Reading never blocks another reader: the lock is a reader-writer lock, and every operation is
one critical section, so a read never sees a half-finished edit.

::::{tab-set}
:::{tab-item} Rust
:sync: rust
```{literalinclude} ../../examples/book_threads.rs
:language: rust
:start-after: ANCHOR: readers
:end-before: ANCHOR_END: readers
:dedent:
```
:::
:::{tab-item} Python
:sync: python
```{literalinclude} examples/python/threads.py
:language: python
:start-after: ANCHOR: readers
:end-before: ANCHOR_END: readers
:dedent:
```
:::
::::

## The GIL, for Python users

Python's global interpreter lock is released around the operations that hold the document lock for a
while — parsing, serializing, validating and cross-document copies — so one thread parsing a large
document does not freeze the others. Everything else holds the GIL, because those operations are a
lock acquisition plus a small read.

Both facts are checked by tests: the Rust suite has a multi-threaded stress test with a watchdog,
and the Python suite shares one document between four threads with a join timeout.

## What "thread-safe" does not mean

It does not mean that your *edits* are transactional across calls: `if node.parent() == … { node.detach() }`
is two operations, and another thread may get in between. Each individual operation is atomic, and
there is no way to corrupt the tree, but a compound invariant of your own needs your own
synchronisation.

Next: [Using the Python package](python-usage.md) for the Python side of the same guarantees.
