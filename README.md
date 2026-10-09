# biodivine-lib-xml-dom

An XML DOM library for Rust with an idiomatic Python package on top, built for **document
manipulation**: build, edit, traverse, validate and serialize XML trees.

The parts that are usually awkward are the parts this library gets right:

* **Namespaces as expanded names.** Every element and attribute name carries its local name plus the
  namespace (URI *and* prefix it was written with), so prefix juggling disappears and two spellings
  of the same expanded name compare equal.
* **Thread-safe documents.** A document owns exactly one reader-writer lock. Every operation takes
  it once and no operation ever holds two, so a tree can be shared between threads and no sequence
  of calls can deadlock. Handles are cheap (`Send + Sync`) views into the document.
* **Integrity by construction, and one validation pass.** Locally decidable properties are enforced
  by the types (`NCName`, `Text`, `Comment`, …), so nothing invalid can enter a document; the
  whole-document properties (namespace scope, unique `xml:id`s, structural invariants) are collected
  by `Document::validate()`, which reports *every* problem at once.
* **No namespace magic.** Editing never adds, removes or rewrites a namespace declaration; edits are
  silent and fast, and `validate()` is where you ask. The [documentation book](docs/book) states the
  full contract.

## Three layers

```text
biodivine-lib-xml-dom            the Rust library (no PyO3 code at all)
biodivine-lib-xml-dom-py-sys     a thin PyO3 mirror of the Rust API, imported as `..._sys`
biodivine_lib_xml_dom            the pure-Python package: coercion, Pythonic protocols, conveniences
```

The mapping between the Rust API and the Python one is itemised in
[`docs/design/BINDINGS.md`](docs/design/BINDINGS.md), including what is deliberately not mirrored.

## Examples

Rust:

```rust
use biodivine_lib_xml_dom::{Document, Namespace, QualifiedName, write_string};

let document = Document::empty();
let ex = Namespace::prefixed("http://example.com", "ex").unwrap();

let root = document.create_element(QualifiedName::with_namespace("root", &ex).unwrap());
root.declare_namespace(ex.clone());
document.set_root(root.clone());

let child = document.create_element(QualifiedName::with_namespace("child", &ex).unwrap());
child.append_child(document.create_text("Hello, World!").unwrap());
root.append_child(child);

assert!(document.is_valid());
assert_eq!(
    write_string(&document).unwrap(),
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
assert xml.write(document) == (
    '<ex:root xmlns:ex="http://example.com"><ex:child>Hello, World!</ex:child></ex:root>'
)
```

More, with a language switch on every example, in the [documentation book](docs/book).

## Building and testing

```sh
make build              # cargo build --workspace
make test               # cargo test --workspace (both crates)
make python-extension   # build the native extension into .venv (maturin develop --release)
make test-python        # pytest against the built extension
make examples           # run every Rust example the book includes
make lint               # cargo fmt --check, cargo clippy, pytest
make docs               # rustdoc + Python API reference + the book
```

By hand, without `make`:

```sh
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace

python3 -m venv .venv
.venv/bin/pip install maturin pytest sphinx sphinx-design myst-parser
cd biodivine-lib-xml-dom-py-sys && VIRTUAL_ENV=../.venv ../.venv/bin/maturin develop --release && cd ..
.venv/bin/python -m pytest biodivine-lib-xml-dom-py-sys/tests-python
docs/build_docs.sh
```

## Documentation

| where | what |
| --- | --- |
| `cargo doc --open` | the Rust API (rustdoc) |
| [`docs/book`](docs/book) | the book: tutorials with Rust/Python examples you can switch between |
| `biodivine-lib-xml-dom-py-sys/docs` | the Python API reference (Sphinx autodoc) |
| [`docs/design/REVIEW.md`](docs/design/REVIEW.md) | the audit of the original implementation: per-requirement status and eleven ranked defects |
| [`docs/design/PLAN.md`](docs/design/PLAN.md) | the architecture, the deadlock-freedom and panic-safety arguments, the API sketch, the risks, and §15 with every deviation and its reason |
| [`docs/design/BINDINGS.md`](docs/design/BINDINGS.md) | the Python bindings audit: mirroring verdicts, error mapping, GIL policy, `Send`/`Sync` justifications |
| [`docs/design/evidence`](docs/design/evidence) | reproducible evidence: baseline gates, defect probes with raw transcripts, the rule inventory, the rule-enforcement verdicts, the docs/bindings build transcripts |
| [`specification`](specification) | the XML 1.0 and Namespaces 1.0 specifications and one file per rule |

## Scope and limitations

XML 1.0 (fifth edition) and Namespaces in XML 1.0 (third edition), UTF-8 only. `DOCTYPE`
declarations are read and ignored, so there is no DTD processing: no DTD-defined entities, attribute
types or content models, and no validity checking. The book's
[Known limitations and gotchas](docs/book/limitations.md) chapter lists every deliberate
limitation, from top-level comments being discarded to arena slots never being reclaimed.

## License

MIT.
