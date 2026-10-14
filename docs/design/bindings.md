# The Rust and Python bindings

The Python bindings have two layers on top of the Rust library, and the import path shows the split.
The native layer is the `biodivine-lib-xml-dom-py-sys` crate of this repository; it is compiled
into the `biodivine-sys` extension (the `biodivine-py` repository) together with the native layers
of the other Biodivine libraries, which is what lets their bindings share the XML types. The Python
layer is the `biodivine.xml_dom` module of the `biodivine` package, in the same repository as the
extension:

<!-- diagram: layers.svg -->
<svg xmlns="http://www.w3.org/2000/svg" width="720" height="220" viewBox="0 0 720 220" role="img" aria-label="Three layers: the Rust library, its native PyO3 mirror in the biodivine_sys extension, and the biodivine.xml_dom Python module that wraps it.">
  <style>
    .box { fill: #f8fafc; stroke: #475569; stroke-width: 1.2; }
    .label { font-family: system-ui, -apple-system, "Segoe UI", sans-serif; font-size: 13px; fill: #0f172a; }
    .small { font-family: system-ui, -apple-system, "Segoe UI", sans-serif; font-size: 11px; fill: #475569; }
    .title { font-family: system-ui, -apple-system, "Segoe UI", sans-serif; font-size: 14px; font-weight: 600; fill: #0f172a; }
    .edge { stroke: #4b5563; stroke-width: 1.4; fill: none; }
  </style>
  <text class="title" x="24" y="26">Your program</text>
  <rect class="box" x="24" y="40" width="320" height="70" rx="8"/>
  <text class="label" x="40" y="66">biodivine.xml_dom</text>
  <text class="small" x="40" y="86">the Python module: coercion, protocols, conveniences</text>
  <rect class="box" x="24" y="130" width="320" height="70" rx="8"/>
  <text class="label" x="40" y="156">biodivine_sys.xml_dom</text>
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

`biodivine_sys.xml_dom` is the native module; everything documented lives in `biodivine.xml_dom`.

## 1. What is mirrored, item by item

The Python layer exposes each Rust item where it makes sense to do so. The table names the `_sys`
binding and the Python name for every public Rust item group, so the items that are deliberately
not mirrored are visible rather than implicit. Two conventions apply throughout:

* **Methods that read a value become properties** in the Python layer (`root`, `parent`, `kind`,
  `id`, `document`, `qualified_name`, `local_name`, `namespace`, `attributes`, `node_count`,
  `capacity`, `xml_declaration`, the sibling and child accessors, ...); methods that compute or
  change something stay methods. `_sys` keeps the Rust shape (everything is a method).
* **Panicking/`_checked` pairs are exposed once**, with the `_checked` behaviour under the plain
  name, because Python has no panics: a failing operation raises.

| Rust item group | `_sys` binding | Python | verdict |
| --- | --- | --- | --- |
| `Document` (`new`, `root`, `set_root`, `clear_root`, `create_element`, `create_text`, `create_comment`, `create_cdata`, `create_processing_instruction`, `xml_declaration`, `set_xml_declaration`, `node_count`, `nodes`, `node`, `ptr_eq`) | `Document` | `Document` | **mirrored**; `Document()` is `Document::new`, and the declaration is the assignable `xml_declaration` property |
| `Document::validate`, `is_valid`, `validation_errors` | same | same | **mirrored**; `validate` raises `XmlValidationError`, whose `errors` attribute is the list of problems |
| `Document::collect_garbage`, `capacity` | same | same | **mirrored**; `collect_garbage` returns a `GcStats` |
| `Document::with_gc_config`, `gc_config`, `set_gc_config`, `GcConfig` | — | — | **not mirrored**: the Python API uses the default capacity policy and only reads the statistics |
| `Document::parse_string`, `parse_bytes`, `parse_file`, `write_string(_with)`, `write_file(_with)` | `Document` static and instance methods | `Document.parse` (`str` or bytes-like), `Document.parse_file`, `Document.write`, `Document.write_file` | **mirrored**; one Python method per Rust pair, with `options=None` instead of a `_with` twin, and any `os.PathLike` for paths |
| `Document::parse_reader`, `write_to(_with)` | — | — | **not mirrored**: Python passes strings, bytes or paths |
| `Node` (`document`, `id`, `kind`, `as_element`, `is_attached`, `is_ancestor_of`, `parent`, `children`, `child_elements`, `first_child`, `last_child`, `next_sibling`, `previous_sibling`, `index_in_parent`, `descendants`, `text`, `comment`, `cdata`, `processing_instruction`, `detach`, `append_child`, `insert_child`, `insert_before`, `insert_after`, `replace_with`, `shallow_clone`, `deep_clone`, `shallow_clone_into`, `deep_clone_into`, `ptr_eq`, `belongs_to`) | `Node` | `Node` | **mirrored**, same names |
| `Node`'s `Clone` (a handle copy) | — | `copy.copy` | **mirrored**: `copy.copy` of a handle returns the same handle; data is copied with `deep_clone`/`shallow_clone` |
| `Element` (`qualified_name`, `local_name`, `namespace`, `set_qualified_name`, `attributes`, `attribute`, `has_attribute`, `set_attribute`, `remove_attribute`, `clear_attributes`, `namespace_declarations`, `namespaces_in_scope`, `declare_namespace`, `declare_namespace_checked`, `undeclare_default_namespace`, `remove_namespace_declaration`, `lookup_namespace`, `resolve_element_name`, `resolve_attribute_name`) | `Element` | `Element` | **mirrored**, same names; `remove_namespace_declaration` raises `KeyError` for a missing declaration, like `dict.pop` |
| `Element::attribute_without_namespace` | same | `Element.get(name)` | **mirrored differently**: in Python every name argument accepts a plain `str` meaning "no namespace" |
| `Element::node`, `into_node`, and `Deref<Target = Node>` | `Element.node()` | `Element` subclasses `Node` | **mirrored differently in each layer, on purpose**: `_sys` cannot express `Deref`, so it offers `.node()`; the Python `Element` inherits from `Node`, so no conversion is ever needed |
| `Namespace` (`new`, `without_prefix`, `prefixed`, `uri`, `prefix`, `has_same_uri`) | `Namespace` | `Namespace` | **mirrored**; `has_same_uri` also accepts a URI string |
| `Namespace::prefix_str`, `QualifiedName::local_name_str` | — | — | **not mirrored**: Python strings need no second accessor |
| `QualifiedName` (`new`, `without_namespace`, `with_namespace`, `local_name`, `namespace`) | `QualifiedName` | `QualifiedName` | **mirrored**; Python also accepts a URI string wherever a namespace is expected |
| `NodeId` (`From<u32>`, `Eq`, `Hash`, `Display`) | `NodeId` | `NodeId` | **mirrored**; `NodeId(int)` on both sides, resolved only through `Document.node` |
| `NodeKind` | `NodeKind` | `NodeKind` (an `enum.Enum`) | **mirrored** |
| `NodeContent` | — | — | **not mirrored**: `kind` plus the typed accessors (`as_element`, `text`, `comment`, `cdata`, `processing_instruction`) carry the same information in the form Python code wants |
| `WriteOptions`, `DeclarationStyle`, `EmptyElementStyle` | same | a frozen dataclass and two `enum.Enum` classes | **mirrored** |
| `xml_spec::XmlDeclaration` | `XmlDeclaration` | `XmlDeclaration` | **mirrored**; `version`, `encoding` and `standalone` are properties |
| `XmlError` | `XmlError` and five subclasses | same | **mirrored as an exception hierarchy** (see §2) |
| `ValidationError`, `ValidationErrors`, `ValidationErrorKind` | `ValidationError` | `ValidationError` (a frozen dataclass), `ValidationErrors` (a `list`) | **mirrored**; the kind is a stable snake_case string, and `rule` is the identifier of the rule it comes from |
| `GcStats` | `GcStats` | `GcStats` (a frozen dataclass) | **mirrored** |
| `cancel_this`, `XmlError::Cancelled` | — | — | **mirrored differently**: Python signal handlers interrupt long operations (see §3) |
| `MAX_NODES` | — | — | **not mirrored**: reachable only with more than four billion nodes in one document |
| `xml_spec::{NCName, Text, Comment, CData, PiTarget, PiData}` and the predicates | — | — | **not mirrored**: in Rust these types are how validity is enforced; Python's equivalent is that every constructor and `create_*` method validates and raises |
| `xml_spec::rule_set`, `documentation` | — | — | **not mirrored**: documentation of the Rust crate |

### Python-only additions

These exist in the Python layer only, and each one is a convenience rather than behaviour:

* `Node.__iter__`/`__len__`/`__getitem__` over children; `Element.attributes` returns a `dict`;
  `Element.get(name, default=None)`; `Element.local_name` and `Element.qualified_name` are
  assignable.
* `Document.create_element_tree(name, *children)`; `copy.deepcopy(document)` creates an independent
  document with a copy of the tree.
* `ValidationErrors` is a `list` subclass with `kinds()` and `messages()`, so
  `if document.validation_errors():` reads naturally.
* Argument coercion: a `str` where a name is expected (meaning "no namespace"), a
  `(local_name, namespace_or_uri)` tuple as a name, a URI `str` where a namespace is expected, and
  any `os.PathLike` for file paths.

## 2. Error mapping

| Rust error | Python exception |
| --- | --- |
| `XmlError::Io` | `XmlIoError` |
| names, text, comments, CDATA, PI, duplicate attributes, malformed markup, UTF-8, encodings, XML version, entity/character references, missing/multiple roots, content outside the root | `XmlSyntaxError` |
| invalid, reserved or undeclared namespace prefixes | `XmlNamespaceError` |
| foreign document, cycle, not an element, no parent, not a child, index out of bounds, root has a parent, cannot attach root | `XmlDocumentError` |
| `XmlError::Validation` | `XmlValidationError`, whose `errors` attribute (and `args[1]`) is a `ValidationErrors` list |
| anything else | `XmlError` |

All of these derive from `XmlError`, and the message text is the Rust `Display`. The dispatch lives
in exactly one function (`error::to_py_err`); the one case with a payload, `XmlValidationError`,
gets its problem list from a helper (`validation::validation_failure`). Each subclass is exercised
by a pytest case. An interrupted operation is not an error of this library: the exception the
signal handler raised (`KeyboardInterrupt` for Ctrl+C) is propagated unchanged.

## 3. GIL policy and interruption

Every operation acquires one document lock. The operations that hold it for a time proportional to
the document release the GIL, so a long operation in one Python thread does not freeze the others:

| operation | GIL | why |
| --- | --- | --- |
| `Document.parse`, `Document.parse_file` | released | parses a whole document |
| `Document.write`, `Document.write_file` | released | serializes a whole document |
| `Document.validate`, `is_valid`, `validation_errors` | released | walks the whole arena |
| `Document.collect_garbage` | released | walks the whole arena |
| `deep_clone`, `deep_clone_into` | released | copies a subtree |
| everything else | held | one lock acquisition and a bounded read or write, where releasing would cost more than it saves |

A Python signal handler only runs when the main thread calls into the interpreter, so a long
operation without the GIL would postpone Ctrl+C until it returns. Running the handlers from inside
the operation is not an option either: they would run while the document lock is held, and a
handler that touches the same document would deadlock.

The binding therefore runs a long operation (one on a large input or document) that is called from
the main thread on a scoped worker thread, under a `cancel_this` atomic trigger. The calling thread
waits without the GIL and checks for signals every few milliseconds. A handler therefore runs on a
thread that holds no document lock; if it raises, the trigger is cancelled, the worker stops at its
next cancellation check (leaving the document as the Rust core documents for a cancelled
operation), and the handler's own exception is propagated unchanged. Short operations, and
operations called from other threads (which never run signal handlers), simply run without the GIL.

## 4. Deadlock freedom carries over

No binding acquires two document locks at once. The one operation that concerns two documents (the
cross-document copy) goes through `Node::deep_clone_into`, which snapshots the source under a read
lock, releases it, and only then writes to the target. That is the same argument the Rust core
makes, so the Python layer inherits it rather than re-establishing it, and signal handlers never run
under a document lock (§3). The Python test suite of `biodivine.xml_dom` shares one document between four Python threads (reads,
writes and validation) under a join timeout, and runs a signal handler that edits the document an
interrupted operation is working on.

## 5. Thread safety of the native classes

| class | wraps | `Send`/`Sync` because | `unsendable` needed? |
| --- | --- | --- | --- |
| `Document` | `biodivine_lib_xml_dom::Document` | `Arc<DocumentInner>` where `DocumentInner` is a `RwLock<Arena>`; all payloads are `Send + Sync` | no |
| `Node` | `Node` (a `Document` handle + a `u32` index) | both fields are `Send + Sync` | no |
| `Element` | `Element` (a `Node` + the element invariant) | as above | no |
| `Namespace`, `QualifiedName` | `Arc`-shared immutable values | immutable data behind `Arc` | no |
| `NodeId`, `NodeKind`, `GcStats`, `WriteOptions`, the style enums | plain `Copy` data | — | no |
| `XmlDeclaration`, `ValidationError` | owned strings and ids | — | no |

A compile-time assertion for all of them lives in `src/tests.rs::handles_are_send_and_sync`, and
the Python test suite moves a `Document` and a `Node` between Python threads.

## 6. Boundary hygiene

* **No panics on input.** Every failure path goes through `error::to_py_err`; the one `expect`
  call in `biodivine-lib-xml-dom-py-sys/src/` is on a conversion that cannot fail.
* **No `panic = "abort"`.** No profile in the workspace sets it, so a panic in the bindings surfaces
  as PyO3's `PanicException` rather than aborting the interpreter.
* **The mirror is callable from Rust too.** The `#[pymethods]` bodies are `pub`, so this crate's own
  tests (`src/tests.rs`) drive the exact same code paths Python does, without a built wheel.

## 7. Building and testing

The binding crate is a plain Rust library: its classes declare `module = "biodivine_sys.xml_dom"`,
and its `register` function adds them (and the exception classes) to a module created by the
`biodivine-sys` extension. It is never built as an extension on its own, so it has no
`extension-module` feature and no Python packaging; `cargo test` links it into a test binary that
starts an interpreter in-process (the `pyo3/auto-initialize` dev-dependency), which is how
`src/tests.rs` exercises it. See `docs/design/build.md` for the setup, and the `biodivine-py`
repository for building the extension and running the Python test suite.

## 8. Known gaps

* The gaps listed in §1 are deliberate, and each has a reason.
* Python can only interrupt long operations through signal handlers; there is no cancellation token
  and no `timeout=` argument yet, and operations started on a thread other than the main thread
  cannot be interrupted.
* `abi3` is not enabled, so a wheel is specific to one CPython version.
