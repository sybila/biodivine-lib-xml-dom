# biodivine-lib-xml-dom

An XML DOM library with full namespace support, whole-document validation and thread-safe
documents, in Rust with an idiomatic Python package on top.

The library is designed for **document manipulation**: you build, edit, traverse, validate and
serialize XML trees. It is not a streaming parser, but it gets the awkward parts right —
expanded names instead of prefix juggling, one lock per document so a tree can be shared between
threads, and a validation pass that reports everything at once instead of failing on the first
problem.

## What it looks like

::::{tab-set}
:::{tab-item} Rust
:sync: rust
```{literalinclude} ../../examples/book_getting_started.rs
:language: rust
:lines: 10-30
```
:::
:::{tab-item} Python
:sync: python
```{literalinclude} examples/python/getting_started.py
:language: python
:lines: 9-22
```
:::
::::

## The three layers

| layer | what it is | who uses it |
| --- | --- | --- |
| `biodivine-lib-xml-dom` | the Rust library; no PyO3 anywhere | Rust programs |
| `biodivine-lib-xml-dom-py-sys` | a thin PyO3 mirror of the Rust API, imported as `biodivine_lib_xml_dom._sys` | the layer below |
| `biodivine_lib_xml_dom` | the pure-Python package: argument coercion, Pythonic protocols, conveniences | Python programs |

The mapping between the Rust API and the Python one is itemised in `docs/design/BINDINGS.md`, and
the Rust API itself is documented in its rustdoc.

## Running the examples

Every example in this book is a real file: the Rust ones are `examples/book_*.rs` targets of the
crate, and the Python ones are scripts under `docs/book/examples/python/`. They are executed by the
test suites, so nothing here can quietly rot.

```sh
cargo run --example book_getting_started        # Rust
.venv/bin/python docs/book/examples/python/getting_started.py   # Python
```

## Chapters

```{toctree}
:maxdepth: 1
getting-started
building-documents
traversing-and-editing
namespaces
parsing-and-serializing
validation
thread-safety
python-usage
migration
limitations
design-notes
```
