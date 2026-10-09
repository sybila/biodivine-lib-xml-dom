# biodivine-lib-xml-dom — Python API

An XML DOM library with full namespace support and thread-safe documents, exposed as an idiomatic
Python package. The implementation is the Rust crate `biodivine-lib-xml-dom`; this package is a thin
Pythonic layer over the native `biodivine_lib_xml_dom._sys` extension.

## Installation

```sh
python3 -m venv .venv
.venv/bin/pip install maturin
cd biodivine-lib-xml-dom-py-sys
../.venv/bin/maturin develop --release
```

Building the extension is a prerequisite for these documents as well, because `autodoc` imports the
package; if the import fails, the build fails with `ModuleNotFoundError:
biodivine_lib_xml_dom._sys`, which means the extension was not built.

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

## Where to look next

* {doc}`api` — the generated reference for every public module.
* The tutorial book (`docs/book`) covers the same ground with runnable examples in both Python and
  Rust, and has a chapter on migration from the 0.1 API.
* `docs/design/BINDINGS.md` in the repository explains how this package maps onto the Rust API and
  what is deliberately not mirrored.

```{toctree}
:hidden:
api
```
