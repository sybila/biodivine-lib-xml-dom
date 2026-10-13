# Biodivine XML DOM

Biodivine XML DOM is an XML DOM library for Rust with an idiomatic Python package on top, built for
**document manipulation**: build, edit, traverse, validate and serialize XML trees.

The Rust crate is called `biodivine-lib-xml-dom`, the Python package `biodivine_lib_xml_dom`.

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
  garbage collection honour [cancel-this](https://crates.io/crates/cancel-this) triggers, and a
  cancelled operation leaves the document unchanged. In Python, Ctrl+C interrupts them.

## Installation

Rust:

```sh
cargo add biodivine-lib-xml-dom
```

Python (built from this repository, see [`docs/design/build.md`](docs/design/build.md)):

```sh
python3 -m venv .venv
.venv/bin/pip install maturin
make python-extension
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
import biodivine_lib_xml_dom as xml

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

Both examples are tested: the Rust one as a documentation test, the Python one by the Python test
suite.

## Documentation

| where | what |
| --- | --- |
| `cargo doc --open` | the Rust API, including the `documentation` module with the design notes |
| `docs/book` | the book: tutorials with Rust and Python examples you can switch between (`make docs` builds it) |
| `biodivine-lib-xml-dom-py-sys/docs` | the Python API reference (`make docs` builds it) |
| [`docs/design/architecture.md`](docs/design/architecture.md) | how the library is built and why: the arena model, the collector, the concurrency argument, the trade-offs |
| [`docs/design/bindings.md`](docs/design/bindings.md) | the Rust ↔ Python mapping, item by item |
| [`docs/design/rule-coverage.md`](docs/design/rule-coverage.md) | which XML rules this library acts on, in which layer, and what is out of scope |
| [`docs/design/build.md`](docs/design/build.md) | setting the project up, running the checks and building the documentation |

## Scope and limitations

XML 1.0 (fifth edition) and Namespaces in XML 1.0 (third edition), UTF-8 only. `DOCTYPE`
declarations are read and ignored, so there is no DTD processing: no DTD-defined entities, attribute
types or content models, and no validity checking. The book's "Known limitations and gotchas"
chapter (`docs/book/limitations.md`) lists every deliberate limitation, from top-level comments
being discarded to the name interner never shrinking.

## License

The code is available under the [MIT license](LICENSE). The `specification` directory contains
material from W3C Recommendations, which is used under the W3C Document License; see
[`specification/NOTICE.md`](specification/NOTICE.md).
