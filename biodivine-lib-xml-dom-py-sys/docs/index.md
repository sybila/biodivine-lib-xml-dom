# Biodivine XML DOM for Python

Biodivine XML DOM builds, parses, edits, validates and writes XML documents, with full namespace
support and documents that can be shared between threads. The implementation is the Rust crate
`biodivine-lib-xml-dom`; this package is the Pythonic layer on top of it, so the behaviour — down to
the error messages — is the same in both languages.

The package is deliberately three layers, and the import path shows the split:

| layer | import path | what it is |
| --- | --- | --- |
| the Rust library | — | the implementation, with no Python code in it |
| the native extension | `biodivine_lib_xml_dom._sys` | a thin, one-to-one mirror of the Rust API |
| this package | `biodivine_lib_xml_dom` | argument coercion, Pythonic protocols, conveniences |

Everything documented here is the third layer. The mirror is importable but not part of the
documented surface.

## Installation

The package is built from this repository with [maturin](https://maturin.rs), which needs a Rust
toolchain and a development version of Python:

```sh
python3 -m venv .venv
.venv/bin/pip install --upgrade pip
.venv/bin/pip install maturin pytest
cd biodivine-lib-xml-dom-py-sys
VIRTUAL_ENV="$PWD/../.venv" ../.venv/bin/maturin develop --release
```

Building the extension is a prerequisite for these documents as well, because `autodoc` imports the
package; if the import fails, the build fails with `ModuleNotFoundError:
biodivine_lib_xml_dom._sys`, which means the extension was not built. The full setup, including the
documentation toolchain, is described in the repository's build instructions.

## Quick start

```python
import biodivine_lib_xml_dom as xml

document = xml.Document()
root = document.create_element("root")
document.set_root(root)

ex = xml.Namespace("http://example.com", "ex")
child = document.create_element(xml.QualifiedName("child", ex))
root.append_child(child)
child.append_child(document.create_text("Hello"))

document.is_valid()                  # False: nothing declares `ex` yet
root.declare_namespace(ex)
document.is_valid()                  # True
xml.write(document)
# '<root xmlns:ex="http://example.com"><ex:child>Hello</ex:child></root>'
```

## Thread safety

A document owns exactly one reader-writer lock. Every method acquires and releases it, and no
operation ever holds two document locks, so a document can be shared freely between threads and no
sequence of calls can deadlock. Parsing, serializing, validating and cross-document copies release
the interpreter lock while they work, so a long operation in one thread does not freeze the others.
A document handle is safe to pass to another thread; nodes and elements refer to their document, so
they stay valid for as long as the document is alive.

## Cancellation

The operations that walk the whole document — parsing, writing, validating, cross-document copies
and `Document.collect_garbage()` — can be interrupted. A `Ctrl+C` (`KeyboardInterrupt`) delivered
while one of them runs cancels it and is re-raised as a `KeyboardInterrupt`, instead of being
ignored until the call returns. The underlying Rust API can likewise be cancelled by any
[cancel-this](https://crates.io/crates/cancel-this) trigger registered on the calling thread.

## Garbage collection and node ids

A node lives while something refers to it: a `Node`/`Element` handle, or the document root, which the
document keeps alive on its own. Detaching a node, dropping a subtree or discarding a copy therefore
does not leak — the collector reclaims anything that no handle and no root can reach, and its slot is
handed out again later. The collector runs automatically when an arena is full, and
`Document.collect_garbage()` runs it on demand and reports what it did.

`NodeId` is an arena slot number, and it is deliberately not a usable reference on its own: a
reclaimed slot is reused, so an id means something only while the node it was created for is alive.
`Document.node(id)` resolves an id to a node (or `None`), and it accepts a `NodeId` or a plain
integer.

```python
document = xml.Document()
node = document.create_text("temporary")
node_id = node.id
assert document.node(node_id) is not None
del node
document.collect_garbage()
assert document.node(node_id) is None      # the slot was reclaimed
```

## Where to look next

* {doc}`api` — the generated reference for every public module.
* The tutorial book in the repository covers the same ground with runnable examples in both Python
  and Rust.
* The Rust documentation of `biodivine-lib-xml-dom` describes the same behaviour in more depth,
  including the design notes, and its `documentation` module contains the item-by-item mapping of
  this package onto the Rust API and what is deliberately not mirrored.

```{toctree}
:hidden:
api
```
