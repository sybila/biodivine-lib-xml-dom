# Getting started

## Install

Rust:

```sh
cargo add biodivine-lib-xml-dom --git https://github.com/sybila/biodivine-lib-xml-dom
```

Python (the extension is built from the repository):

```sh
python3 -m venv .venv
.venv/bin/pip install maturin
cd biodivine-lib-xml-dom-py-sys
../.venv/bin/maturin develop --release
```

## Build a document

A document starts empty and gets a root element. Both languages then work with *expanded names*:
`("child", ex)` means the local name `child` in the namespace `ex`, and the prefix is part of that
namespace rather than part of the name.

::::{tab-set}
:::{tab-item} Rust
:sync: rust
```{literalinclude} ../../examples/book_getting_started.rs
:language: rust
:lines: 10-32
```
:::
:::{tab-item} Python
:sync: python
```{literalinclude} examples/python/getting_started.py
:language: python
:lines: 9-27
```
:::
::::

Two things are worth noticing:

* the root declares the namespace, so `write` produces `<ex:root xmlns:ex="…">…</ex:root>`;
* `is_valid` / `is_valid()` returns `true` because the document is consistent. Remove the
  declaration and it would return `false` — see [Validating a document](validation.md).

## Where values are checked

Names, text, comments, CDATA sections, processing instructions and namespace URIs are validated
*when they are constructed*. Nothing invalid can enter a document through the API:

::::{tab-set}
:::{tab-item} Rust
:sync: rust
```{literalinclude} ../../examples/book_parsing.rs
:language: rust
:lines: 30-45
```
:::
:::{tab-item} Python
:sync: python
```{literalinclude} examples/python/parsing.py
:language: python
:lines: 22-35
```
:::
::::

Continue with [Building documents](building-documents.md) or jump straight to
[Namespaces](namespaces.md) if that is what you came for.
