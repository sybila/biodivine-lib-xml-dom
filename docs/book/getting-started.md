# Getting started

Biodivine XML DOM builds, parses, edits, validates and writes XML documents. This chapter is a tour
of what the library does, in the order you are likely to need it; each section is a short program in
Rust and Python that you can copy and extend. Everything else in the book is a more detailed look at
one of these steps.

## Install

Rust:

```sh
cargo add biodivine-lib-xml-dom
```

Python (the package is built from this repository):

```sh
python3 -m venv .venv
.venv/bin/pip install maturin
cd biodivine-lib-xml-dom-py-sys
../.venv/bin/maturin develop --release
```

`documentation::build` — the build page of the crate documentation, also available in the repository
— lists the toolchains, every `make` target and how to build the documentation itself.

## Build a document

A document starts empty. You create nodes, give them a name, and attach them to a root; names are
*expanded* names, so a namespace travels with the name instead of being juggled as a prefix string.

::::{tab-set}
:::{tab-item} Rust
:sync: rust
```{literalinclude} ../../examples/book_getting_started.rs
:language: rust
:start-after: ANCHOR: document
:end-before: ANCHOR_END: document
:dedent:
```
:::
:::{tab-item} Python
:sync: python
```{literalinclude} examples/python/getting_started.py
:language: python
:start-after: ANCHOR: document
:end-before: ANCHOR_END: document
:dedent:
```
:::
::::

## Write it out, and read it back

Serialization is lossless for everything the data model represents: prefixes, namespace
declarations, escaping and the XML declaration all survive a round trip, which is why the parsed
form of the output is the same document again.

::::{tab-set}
:::{tab-item} Rust
:sync: rust
```{literalinclude} ../../examples/book_getting_started.rs
:language: rust
:start-after: ANCHOR: serialize
:end-before: ANCHOR_END: serialize
:dedent:
```
:::
:::{tab-item} Python
:sync: python
```{literalinclude} examples/python/getting_started.py
:language: python
:start-after: ANCHOR: serialize
:end-before: ANCHOR_END: serialize
:dedent:
```
:::
::::

## Ask a node what it is

A handle carries the full expanded name, so you can ask for the local name and the namespace
separately instead of splitting strings. Text, comments, CDATA sections and processing instructions
are nodes too, with their own kinds.

::::{tab-set}
:::{tab-item} Rust
:sync: rust
```{literalinclude} ../../examples/book_getting_started.rs
:language: rust
:start-after: ANCHOR: names
:end-before: ANCHOR_END: names
:dedent:
```
:::
:::{tab-item} Python
:sync: python
```{literalinclude} examples/python/getting_started.py
:language: python
:start-after: ANCHOR: names
:end-before: ANCHOR_END: names
:dedent:
```
:::
::::

## Validate the whole document

Editing is silent and fast: it never adds, removes or rewrites a namespace declaration behind your
back. `validate` is the counterpart — one call that checks the whole document and reports *every*
problem it can find, each with the node it belongs to and the rule it comes from.

::::{tab-set}
:::{tab-item} Rust
:sync: rust
```{literalinclude} ../../examples/book_validation.rs
:language: rust
:start-after: ANCHOR: problems
:end-before: ANCHOR_END: problems
:dedent:
```
:::
:::{tab-item} Python
:sync: python
```{literalinclude} examples/python/validation.py
:language: python
:start-after: ANCHOR: problems
:end-before: ANCHOR_END: problems
:dedent:
```
:::
::::

## Share it between threads

A document owns exactly one reader-writer lock, and no operation ever holds two. That is what makes
a document safe to share: every operation is one critical section, a read never sees a half-finished
edit, and no sequence of calls can deadlock.

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

## Let go of memory you no longer need

A node lives while something refers to it: a handle, or the document root. Dropping the last handle
of a detached subtree makes it collectable, and `collect_garbage` reclaims it on demand (the
collector also runs on its own when an arena fills up). `NodeId` is an arena slot number, not a
handle: it resolves through `node(id)`, and it stops resolving once the slot has been reclaimed.

::::{tab-set}
:::{tab-item} Rust
:sync: rust
```{literalinclude} ../../examples/book_gc.rs
:language: rust
:start-after: ANCHOR: collected
:end-before: ANCHOR_END: collected
:dedent:
```
:::
:::{tab-item} Python
:sync: python
```{literalinclude} examples/python/gc.py
:language: python
:start-after: ANCHOR: collected
:end-before: ANCHOR_END: collected
:dedent:
```
:::
::::

## Where to go next

* [Building documents](building-documents.md) and [Traversing and editing](traversing-and-editing.md)
  are the two chapters about the tree itself.
* [Namespaces](namespaces.md) explains why prefixes are decoration and expanded names are the truth.
* [Parsing and serializing](parsing-and-serializing.md) covers the I/O options and everything the
  parser rejects.
* [Validating a document](validation.md) lists what is checked and what is deliberately not.
* [Thread safety](thread-safety.md) explains the locking model, in both languages.
* [Using the Python package](python-usage.md) is the chapter to read if you are writing Python.
* The [Rust API reference](https://docs.rs/biodivine-lib-xml-dom) and the design notes that ship
  with it (`documentation::architecture`) are the reference for everything on this page.
