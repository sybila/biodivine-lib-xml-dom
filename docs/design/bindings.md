# The Rust and Python bindings

The Python side is three layers, and the import path shows the split:

<!-- diagram: layers.svg -->
<svg xmlns="http://www.w3.org/2000/svg" width="720" height="220" viewBox="0 0 720 220" role="img" aria-label="Three layers: the Rust library, the native PyO3 mirror imported as _sys, and the pure Python package that wraps it.">
  <style>
    .box { fill: #f8fafc; stroke: #475569; stroke-width: 1.2; }
    .label { font-family: system-ui, -apple-system, "Segoe UI", sans-serif; font-size: 13px; fill: #0f172a; }
    .small { font-family: system-ui, -apple-system, "Segoe UI", sans-serif; font-size: 11px; fill: #475569; }
    .title { font-family: system-ui, -apple-system, "Segoe UI", sans-serif; font-size: 14px; font-weight: 600; fill: #0f172a; }
    .edge { stroke: #4b5563; stroke-width: 1.4; fill: none; }
  </style>
  <text class="title" x="24" y="26">Your program</text>
  <rect class="box" x="24" y="40" width="320" height="70" rx="8"/>
  <text class="label" x="40" y="66">biodivine_lib_xml_dom</text>
  <text class="small" x="40" y="86">the Python package: coercion, protocols, conveniences</text>
  <rect class="box" x="24" y="130" width="320" height="70" rx="8"/>
  <text class="label" x="40" y="156">biodivine_lib_xml_dom._sys</text>
  <text class="small" x="40" y="176">the native extension: a one-to-one mirror of the Rust API</text>
  <rect class="box" x="380" y="85" width="316" height="70" rx="8"/>
  <text class="label" x="396" y="111">biodivine-lib-xml-dom</text>
  <text class="small" x="396" y="131">the Rust library: the implementation, with no Python in it</text>
  <path class="edge" d="M 184 110 L 184 130"/>
  <path class="edge" d="M 184 165 L 184 180"/>
  <path class="edge" d="M 344 165 L 378 140"/>
  <path class="edge" d="M 344 75 L 378 105"/>
  <text class="small" x="24" y="216">Only the top box is the documented API; the two below it are implementation layers.</text>
</svg>
<!-- /diagram -->

`biodivine_lib_xml_dom._sys` is the native module; everything documented lives in
`biodivine_lib_xml_dom`.

## 1. What is mirrored, item by item

The wrapper exposes each Rust item where it makes sense to do so. This table is the audit of that
rule: it names the binding and the Python name for every public Rust item group and gives a verdict,
so the "does not make sense" set is visible rather than implicit.

| Rust item group | `_sys` binding | Python name | verdict |
| --- | --- | --- | --- |
| `Document` (root, `set_root`, `clear_root`, `create_element`, `create_text`, `create_comment`, `create_cdata`, `create_processing_instruction`, `xml_declaration`, `set_xml_declaration`, `validate`, `is_valid`, `node_count`, `nodes`, `node`, `ptr_eq`) | `Document` | `Document` | **mirrored**, same names (documents are cheap `Send + Sync` handles) |
| `Document::set_root` vs `set_root_checked` | `Document.set_root` | `Document.set_root` | **mirrored, panicking twin dropped.** Rust's `set_root` panics; Python has no panics, so the binding is the `_checked` behaviour under the plain name and failures raise `XmlDocumentError`. Same rule for every `_checked`/panicking pair below. |
| `Document::validate` / `is_valid` | `Document.validate` / `is_valid` | same | **mirrored**; `validate` raises `XmlValidationError` carrying all issues, and Python adds `validation_errors()` for the non-raising form (the Rust API returns them as the error value, which Python expresses as an exception) |
| `Node` (`kind`, `as_element`, `is_attached`, `parent`, `children`, `child_elements`, `first_child`, `last_child`, `next_sibling`, `previous_sibling`, `index_in_parent`, `descendants`, `text`, `comment`, `cdata`, `processing_instruction`, `detach`, `remove`, `append_child`, `insert_child`, `insert_before`, `insert_after`, `replace_with`, `shallow_clone`, `deep_clone`, `shallow_clone_into`, `deep_clone_into`, `is_ancestor`, `ptr_eq`, `belongs_to`, `id`) | `Node` | `Node` | **mirrored**, same names |
| `Node` Rust `Clone` = handle copy; `deep_clone`/`shallow_clone` = data copy | — | — | **mirrored as documented**: Python has no `Clone`, so `Document.copy()`/`Node`/`Element` equality is by node identity and cloning data is spelled out (`deep_clone`, …) |
| `Element` (`qualified_name`, `local_name`, `namespace`, `set_qualified_name`, `attributes`, `attribute`, `attribute_local`, `has_attribute`, `set_attribute`, `remove_attribute`, `clear_attributes`, `namespace_declarations`, `namespaces_in_scope`, `declare_namespace`, `declare_namespace_checked`, `undeclare_default_namespace`, `remove_namespace_declaration`, `get_namespace`, `resolve_qualified_name`, `resolve_attribute_name`) | `Element` | `Element` | **mirrored**, same names |
| `Element`'s `Deref<Target = Node>` (so `element.children()` works in Rust) | `Element.node()` | `Element` subclasses `Node` | **mirrored differently in each layer, on purpose**: Rust expresses this with `Deref`, which Python cannot do for a native type, so `_sys` keeps `Element` separate and offers `.node()` (the Rust method), while the pure-Python `Element` inherits from the pure-Python `Node` so a user never has to think about it |
| `Namespace` (`new`, `without_prefix`, `prefixed`, `uri`, `prefix`, `prefix_str`, `is_equal_ns`) | `Namespace` | `Namespace` | **mirrored**, same names |
| `QualifiedName` (`new`, `without_namespace`, `with_namespace`, `local_name`, `namespace`, `resolve_element`, `resolve_attribute`) | `QualifiedName` | `QualifiedName` | **mirrored**; Python additionally accepts a plain URI string wherever a namespace is expected |
| `NodeId` (`From<u32>`, `index`, `Eq`, `Hash`, `Display`) | `NodeId` (constructible from an `int`) | `NodeId` | **mirrored**; an id can be built from an integer on both sides, and it resolves only through the document that handed it out (`Document::node` / `Document.node`) |
| `NodeKind` | `NodeKind` | `NodeKind` | **mirrored** |
| `NodeContent` (enum) | — | — | **not mirrored.** A native enum with five payload variants would force a discriminator dance in Python; `Node.kind()` plus the typed accessors (`as_element`, `text`, `comment`, `cdata`, `processing_instruction`) express exactly the same information and are what Python code wants |
| `WriteOptions`, `DeclarationStyle`, `EmptyElementStyle` | same | same | **mirrored** (Python adds `options=None` defaults where Rust has `write_*_with`) |
| `XmlDeclaration` | `XmlDeclaration` | `XmlDeclaration` | **mirrored** |
| `XmlError` (+ all variants) | `XmlError` and five subclasses | same | **mirrored as an exception hierarchy** (see §2) |
| `XmlValidationError`, `XmlValidationErrors`, `ValidationErrorKind` | `ValidationError`, `ValidationErrors` | same | **mirrored**; the kind is exposed both as a stable snake_case string (`kind()`) and as the rule file it comes from (`rule()`) |
| `Document::parse_string`, `Document::parse_bytes`, `Document::parse_file`, `Document::write_string`, `Document::write_string_with`, `Document::write_file`, `Document::write_file_with` | `Document` static/instance methods with the same names | module-level `parse`, `parse_file`, `write`, `write_file` (+ `options=`) | **mirrored**; Python puts the entry points at module level (its idiom) and delegates to the mirrored `Document` methods; `parse` accepts `str` or `bytes` and the file variants accept any `os.PathLike` |
| `MAX_NODES` | — | — | **not mirrored**: reachable only with more than four billion nodes in one document; Rust documents it where it is checked |
| `xml_spec::{NCName, Text, Comment, CData, PiTarget, PiData}` | — | — | **not mirrored.** These types *are* the Rust construction mechanism: validity is enforced by the type. Python's equivalents are the raising constructors (`Document.create_text`, `QualifiedName(...)`, `Namespace(...)`), so mirroring the wrapper types would create a second, parallel way to validate names with no benefit |
| `xml_spec::{rules, validation, declaration}`, `xml_spec::rule_set`, `documentation` | — | — | **not mirrored**: these are for Rust developers and for the Rust documentation (`rules::assert_rule_exists` anchors the test suite, `NamespaceScope`/`check_*` are the internal shape of validation, `rule_set` reproduces the specification summaries, `documentation` holds the design notes) |
| `Namespace::shares_data_with`, `Arena`, `Interner`, the whole-document validation traversal | — | — | crate-internal in Rust, so nothing to mirror |

### Python-only additions

These exist in the Python layer only, and each one is a convenience rather than behaviour:

* `Node.__iter__`/`__len__`/`__getitem__` over children; `Element.attributes` returns a `dict`;
  `Element.get(name, default=None)`; `Element.local_name = "x"` as a setter.
* `Document.create_element_tree(name, *children)`, `Document.copy()`/`__copy__`/`__deepcopy__`.
* `parse`/`parse_file`/`write`/`write_file` module functions with `options=None` defaults, which
  delegate to the mirrored `Document.parse_string`/`parse_file`/`write_string`/`write_file` methods.
* `ValidationErrors` is a `list` subclass, so `if document.validation_errors():` reads naturally.
* Argument coercion: `str` where a `QualifiedName` is expected (meaning "no namespace"), `Namespace`
  or URI `str` where a namespace is expected, `Node` or `Element` wherever a node is expected
  (`append_child`, `insert_before`, …), and any `os.PathLike` for file paths.

## 2. Error mapping

| Rust error | Python exception |
| --- | --- |
| `XmlError::Io` | `XmlIoError` |
| names, text, comments, CDATA, PI, duplicate attributes, malformed markup, UTF-8, encodings, XML version, entity/character references, missing/multiple roots, content outside the root | `XmlSyntaxError` |
| invalid/reserved/undeclared namespace prefixes, illegal namespace declarations | `XmlNamespaceError` |
| foreign document, cycle, not an element, no parent, not a child, index out of bounds, root has a parent, cannot attach root | `XmlDocumentError` |
| `Document::validate` failures | `XmlValidationError` (with `args[1]` a `ValidationErrors` sequence) |
| anything else | `XmlError` |

All of them derive from `XmlError`, so `except XmlError:` catches everything this library raises and
the message text is the Rust `Display`. The mapping lives in exactly one function
(`error::to_py_err`), and each subclass is exercised by a pytest case.

## 3. GIL policy

Every operation acquires one document lock. The four that hold it for a time proportional to the
document release the GIL, so a long operation in one Python thread does not freeze the others:

| operation | GIL | why |
| --- | --- | --- |
| `Document.parse_string`, `Document.parse_bytes`, `Document.parse_file` | released | parses a whole document |
| `Document.write_string`, `Document.write_file` | released | serializes a whole document |
| `Document.validate`, `is_valid`, `validation_errors` | released | walks the whole arena |
| `Node.deep_clone_into`, `Element.deep_clone_into` | released | snapshots and rebuilds a subtree |
| every other binding | held | one lock acquisition and a bounded read or write, where releasing would cost more than it saves |

The closures passed to `Python::detach` capture only owned Rust values (every handle type is
`Send + Sync`), never a `Bound`/`Py` reference, which is what PyO3 requires. The corresponding
objects are cloned *before* detaching.

## 4. Deadlock freedom carries over

No binding acquires two document locks at once. The one operation that concerns two documents — the
cross-document copy — goes through `Node::deep_clone_into`, which snapshots the source under a read
lock, releases it, and only then writes to the target. That is the same argument the Rust core
makes, and it means the Python layer inherits it rather than re-establishing it. `tests-python`
shares one document between four Python threads (reads, writes and validation) under a 30-second
join timeout, so a hang fails the test instead of blocking the run.

## 5. `#[pyclass]` thread-safety audit

| class | wraps | `Send`/`Sync` because | `unsendable` needed? |
| --- | --- | --- | --- |
| `Document` | `biodivine_lib_xml_dom::Document` | `Arc<DocumentInner>` where `DocumentInner` is a `RwLock<Arena>`; all payloads are `Send + Sync` | no |
| `Node` | `Node` (a `Document` handle + a `u32` index) | both fields are `Send + Sync` | no |
| `Element` | `Element` (a `Node` + the element invariant) | as above | no |
| `Namespace`, `QualifiedName` | `Arc`-shared immutable values | immutable data behind `Arc` | no |
| `NodeId` | `u32` | — | no |
| `WriteOptions`, `XmlDeclaration`, the style enums, `NodeKind` | plain `Copy`/`Clone` data | — | no |
| `ValidationError`, `ValidationErrors` | owned strings and ids | — | no |

A compile-time assertion for all of them lives in `src/tests.rs::handles_are_send_and_sync`, and
`tests-python` moves a `Document` and a `Node` between Python threads.

## 6. Boundary hygiene

* **No panics on input.** `grep -n 'unwrap()\|expect(\|panic!\|unreachable!\|todo!\|unimplemented!'`
  over `biodivine-lib-xml-dom-py-sys/src/` matches nothing outside `src/tests.rs`: every failure
  path goes through `error::to_py_err`. The only `expect` calls in the whole crate are in the
  test module.
* **No `panic = "abort"`.** No profile in the workspace sets it (the root `Cargo.toml` says so
  explicitly), so a panic in the bindings would surface as PyO3's `PanicException` rather than
  aborting the interpreter — a bug would be reportable instead of fatal.
* **`_checked` semantics only.** Every Rust operation that has a panicking twin and a `_checked`
  twin is exposed once, with the `_checked` behaviour, under the plain name.
* **The mirror is callable from Rust too.** The `#[pymethods]` bodies are `pub`, so this crate's own
  tests (`src/tests.rs`) drive the exact same code paths Python does, without a built wheel.

## 7. Building and testing

```sh
python3 -m venv .venv
.venv/bin/pip install maturin pytest

cd biodivine-lib-xml-dom-py-sys
../.venv/bin/maturin develop --release      # builds and installs the extension
../.venv/bin/python -m pytest tests-python  # runs the Python test-suite

cd ..
cargo test --workspace                      # also runs the in-process binding tests
```

`extension-module` is enabled by maturin through `[tool.maturin] features`, not by default, because
`cargo test` links the crate into a test binary that needs libpython while a wheel must not link it.
The crate's `dev-dependencies` enable `pyo3/auto-initialize` for the same reason.

## 8. What building the Python layer found in the Rust core

Writing the wrapper tests surfaced a real defect in the core, which is worth recording because it
is exactly the kind of thing a second layer notices and a single layer does not.

The interner deduplicates the qualified names of a document to avoid duplicate allocations. It must
be keyed by the *structural* name (local name plus namespace including the prefix), not by
`QualifiedName`'s own equality: that equality deliberately ignores the prefix, because for attribute
maps only the expanded name matters. Keying the interner by it meant that interning *rewrote* the
prefix of a name to whatever prefix was stored first: a user could ask for `a` in the default
namespace and get `ex:a`. Since the prefix is what the serializer writes, the document's output (and
possibly its validity) changed without any operation reporting anything.
`src/interner.rs::interning_never_rewrites_a_prefix` pins the behaviour, and
`tests-python/test_public_surface.py::test_argument_coercion_is_part_of_the_surface` covers it from
Python.

## 9. Known gaps

* None of the *functional* API is missing: the gaps listed in §1 (`NodeContent`, the `xml_spec`
  newtypes, `MAX_NODES`) are deliberate and each has a reason.
* The extension is built for the interpreter that maturin finds; `abi3` is not enabled, so a wheel is
  specific to one CPython version. Enabling it is a packaging decision for a release rather than a
  correctness one, and is recorded here instead of being taken silently.
* The Python API reference and the book are generated from the docstrings and the `py.typed` marker
  this package provides; `docs/design/build.md` explains how to build them.
