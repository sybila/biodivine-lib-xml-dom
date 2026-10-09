# Traversing and editing

## Walking the tree

Parents, children, siblings and descendants are all available; the descendants traversal is
iterative, so a deeply nested document cannot overflow the stack. Python additionally offers
`len(node)`, `node[i]` and `for child in node`.

::::{tab-set}
:::{tab-item} Rust
:sync: rust
```{literalinclude} ../../examples/book_editing.rs
:language: rust
:lines: 17-24
```
:::
:::{tab-item} Python
:sync: python
```{literalinclude} examples/python/editing.py
:language: python
:lines: 14-21
```
:::
::::

## Changing the tree

Inserting, moving, replacing and removing are all one critical section on the document lock, which
is what makes them atomic: another thread can never observe a half-finished edit, and the
cycle check cannot be raced against (a bug the previous per-element locking design had).

::::{tab-set}
:::{tab-item} Rust
:sync: rust
```{literalinclude} ../../examples/book_editing.rs
:language: rust
:lines: 26-40
```
:::
:::{tab-item} Python
:sync: python
```{literalinclude} examples/python/editing.py
:language: python
:lines: 22-39
```
:::
::::

| operation | Rust | Python |
| --- | --- | --- |
| append | `append_child` / `append_child_checked` | `append_child` (raises) |
| insert at a position | `insert_child` / `insert_child_checked` | `insert_child` |
| insert next to a sibling | `insert_before`, `insert_after` | same |
| remove from the parent | `detach` (returns the old parent) | `detach` |
| remove and keep the node | `remove` | `remove` |
| swap a node out | `replace_with` | `replace_with` |

## Text, comments, CDATA and processing instructions

Non-element nodes are children like any other; they are created by the document and appended. Their
content is validated on creation, and their kind is discoverable from a handle:

::::{tab-set}
:::{tab-item} Rust
:sync: rust
```{literalinclude} ../../examples/book_parsing.rs
:language: rust
:lines: 8-12
```
:::
:::{tab-item} Python
:sync: python
```{literalinclude} examples/python/parsing.py
:language: python
:lines: 6-11
```
:::
::::

An element's children keep their order, and mixed content (text, elements, comments, CDATA and
processing instructions interleaved) round-trips exactly. The serialized output is escaped so that
a literal carriage return and a `]]>` in text, and whitespace inside attribute values, survive a
re-parse.
