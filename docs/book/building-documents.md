# Building documents

## Documents, nodes and handles

A document owns all of its nodes in a single arena and protects them with a single reader-writer
lock. What you hold in your hand is a **handle**: a document reference plus an index into that
arena.

* copying a handle is cheap and refers to the *same* node;
* a handle stays valid even if the node is detached;
* handles with the same identity compare equal and can be used as dictionary keys;
* handles can be sent to other threads.

::::{tab-set}
:::{tab-item} Rust
:sync: rust
```{literalinclude} ../../examples/book_editing.rs
:language: rust
:lines: 10-15
```
:::
:::{tab-item} Python
:sync: python
```{literalinclude} examples/python/editing.py
:language: python
:lines: 8-12
```
:::
::::

## Attached and detached

A freshly created node is **detached**: it belongs to a document, but it has no parent yet. You can
edit it, clone it, validate it, and attach it later. Attaching a node that is already attached
*moves* it, and the two things that are never allowed are cycles and cross-document attachments:

::::{tab-set}
:::{tab-item} Rust
:sync: rust
```{literalinclude} ../../examples/book_editing.rs
:language: rust
:lines: 34-55
```
:::
:::{tab-item} Python
:sync: python
```{literalinclude} examples/python/editing.py
:language: python
:lines: 30-52
```
:::
::::

The fallible operations come in two flavours: `append_child` panics with a clear message, and
`append_child_checked` returns an error you can handle. In Python only the checked behaviour exists,
as an exception — Python has no panics.

## Copying

There are three different kinds of "copy", and they are spelled differently on purpose:

| what you want | Rust | Python |
| --- | --- | --- |
| another handle to the same node | `clone()` | the object itself / `copy.copy` |
| a detached copy of one node | `shallow_clone()` | `shallow_clone()` |
| a detached copy of a subtree | `deep_clone()` | `deep_clone()` |
| a copy of a subtree in *another* document | `deep_clone_into(&target)` | `deep_clone_into(target)` |

::::{tab-set}
:::{tab-item} Rust
:sync: rust
```{literalinclude} ../../examples/book_editing.rs
:language: rust
:lines: 57-67
```
:::
:::{tab-item} Python
:sync: python
```{literalinclude} examples/python/editing.py
:language: python
:lines: 54-66
```
:::
::::

The cross-document copy is the sanctioned way to move a tree between documents: attaching a handle
to a parent in another document is an error, while copying works and rebuilds the subtree with the
target document's own names. It takes a point-in-time snapshot of the source, so a concurrent edit
in the source cannot produce a half-copied tree.
