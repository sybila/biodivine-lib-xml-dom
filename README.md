[![Crates.io](https://img.shields.io/crates/v/biodivine-lib-xml-dom?style=flat-square)](https://crates.io/crates/biodivine-lib-xml-dom)
[![PyPI](https://img.shields.io/pypi/v/biodivine?style=flat-square)](https://pypi.org/project/biodivine/)
[![Api Docs](https://img.shields.io/badge/docs-api-yellowgreen?style=flat-square)](https://docs.rs/biodivine-lib-xml-dom/)
[![Continuous integration](https://img.shields.io/github/actions/workflow/status/sybila/biodivine-lib-xml-dom/build.yml?branch=master&style=flat-square)](https://github.com/sybila/biodivine-lib-xml-dom/actions?query=workflow%3Abuild)
[![Coverage](https://img.shields.io/codecov/c/github/sybila/biodivine-lib-xml-dom?style=flat-square)](https://codecov.io/gh/sybila/biodivine-lib-xml-dom)
[![MSRV](https://img.shields.io/badge/MSRV-1.95-blue?style=flat-square)](https://github.com/sybila/biodivine-lib-xml-dom/blob/master/Cargo.toml)
[![GitHub issues](https://img.shields.io/github/issues/sybila/biodivine-lib-xml-dom?style=flat-square)](https://github.com/sybila/biodivine-lib-xml-dom/issues)
[![GitHub last commit](https://img.shields.io/github/last-commit/sybila/biodivine-lib-xml-dom?style=flat-square)](https://github.com/sybila/biodivine-lib-xml-dom/commits/master)
[![License](https://img.shields.io/crates/l/biodivine-lib-xml-dom?style=flat-square)](https://github.com/sybila/biodivine-lib-xml-dom/blob/master/LICENSE)

# Biodivine XML DOM

 > Notice: This project is being maintained with extensive use of large language models. However, all major features 
 > and releases have been audited by a human contributor.

Biodivine XML DOM is an XML DOM library for Rust with an idiomatic Python interface on top, built
for **document manipulation**: build, edit, traverse, validate and serialize XML trees.

The Rust crate is called `biodivine-lib-xml-dom`. In Python, the library is the `biodivine.xml_dom`
module of the [`biodivine`](https://github.com/sybila/biodivine-py) package, which contains the
Python interface of every Biodivine library.

The parts that are usually awkward are the parts this library gets right:

* **Namespaces as expanded names.** Every element and attribute name carries its local name plus the
  namespace (URI *and* prefix it was written with), so prefix juggling disappears and two spellings
  of the same expanded name compare equal.
* **Thread-safe documents.** A document owns exactly one reader-writer lock. Every operation takes
  it once and no operation ever holds two, so a tree can be shared between threads and no sequence
  of calls can deadlock. Handles are cheap (`Send + Sync`) views into the document.
* **Integrity by construction, and one validation pass.** Locally decidable properties are enforced
  by the types (`NCName`, `Text`, `Comment`, …), so nothing invalid can enter a document; the
  whole-document properties (namespace scope, unique `xml:id`s, `xml:lang` and `xml:space` values)
  are collected by `Document::validate()`, which reports *every* problem at once.
* **No namespace magic.** Editing never adds, removes or rewrites a namespace declaration; edits are
  silent and fast, and `validate()` is where you ask.
* **Cancellable long operations.** Parsing, serialization, validation, deep copies and explicit
  garbage collection honor [cancel-this](https://crates.io/crates/cancel-this) triggers, and a
  canceled operation leaves the document unchanged. In Python, Ctrl+C interrupts them.

## Installation

Rust:

```sh
cargo add biodivine-lib-xml-dom
```

Python:

```sh
pip install biodivine
```

## Examples

Rust:

```rust
use biodivine_lib_xml_dom::{Document, Namespace, QualifiedName};

let document = Document::new();
let ex = Namespace::prefixed("http://example.com", "ex").unwrap();

let root = document.create_element(QualifiedName::with_namespace("root", &ex).unwrap());
root.declare_namespace(ex.clone());
document.set_root(root.clone());

let child = document.create_element(QualifiedName::with_namespace("child", &ex).unwrap());
child.append_child(document.create_text("Hello, World!").unwrap());
root.append_child(child);

assert!(document.is_valid().unwrap());
assert_eq!(
    document.write_string().unwrap(),
    r#"<ex:root xmlns:ex="http://example.com"><ex:child>Hello, World!</ex:child></ex:root>"#
);
```

Python:

```python
import biodivine.xml_dom as xml

document = xml.Document()
ex = xml.Namespace("http://example.com", "ex")

root = document.create_element(("root", ex))
root.declare_namespace(ex)
document.set_root(root)

child = document.create_element(("child", ex))
child.append_child(document.create_text("Hello, World!"))
root.append_child(child)

assert document.is_valid()
assert document.write() == (
    '<ex:root xmlns:ex="http://example.com"><ex:child>Hello, World!</ex:child></ex:root>'
)
```

Both examples are tested: the Rust one as a documentation test, the Python one by the checks of the
Biodivine book.

## Documentation

| where | what |
| --- | --- |
| `cargo doc --open` | the Rust API, including the `documentation` module with the design notes |
| the [Biodivine book](https://github.com/sybila/biodivine) | tutorials with Rust and Python examples you can switch between; this library is its "XML DOM" part |
| [`biodivine-py`](https://github.com/sybila/biodivine-py) | the Python package and its API reference (`biodivine.xml_dom`) |
| [`docs/design/architecture.md`](docs/design/architecture.md) | how the library is built and why: the arena model, the collector, the concurrency argument, the trade-offs |
| [`docs/design/bindings.md`](docs/design/bindings.md) | the Rust ↔ Python mapping, item by item |
| [`docs/design/rule-coverage.md`](docs/design/rule-coverage.md) | which XML rules this library acts on, in which layer, and what is out of scope |
| [`docs/design/build.md`](docs/design/build.md) | setting the project up, running the checks and building the documentation |

## Scope and limitations

XML 1.0 (fifth edition) and Namespaces in XML 1.0 (third edition), UTF-8 only. `DOCTYPE`
declarations are read and ignored, so there is no DTD processing: no DTD-defined entities, attribute
types or content models, and no validity checking. The book's "Known limitations and gotchas"
chapter lists every deliberate limitation, from top-level comments being discarded to the name
interner never shrinking.

## License

The code is available under the [MIT license](LICENSE). The `specification` directory contains
material from W3C Recommendations, which is used under the W3C Document License; see
[`specification/NOTICE.md`](specification/NOTICE.md).
