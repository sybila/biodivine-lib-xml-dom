# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project follows semantic
versioning.

## 0.2.0 — unreleased (branch `rewrite`)

A rewrite of the storage model, the I/O layer, validation and the public API, plus a Python package.
The user-facing summary of the changes is the book's "Migrating from 0.1" chapter; the reasoning, the
deviations and the per-requirement audit are in `docs/design/REVIEW.md` and `docs/design/PLAN.md`.
`docs/design/VERIFICATION.md` is the evidence that the whole thing works.

### Changed

* **The node storage model is now an arena with one lock per document.** Nodes live in a vector and
  refer to each other by index; `Node`/`Element` are cheap handles (document reference plus index)
  instead of per-node `Arc<RwLock<..>>` values. This removes a memory leak (every 0.1 tree was kept
  alive by a parent↔child `Arc` cycle) and makes edits atomic, so the cross-thread race that could
  create a cycle is gone.
* **The parser no longer panics.** `&amp;` and the other predefined entities are expanded, character
  references are validated, and any other entity reference is a typed error instead of
  `unimplemented!()`.
* **The serializer preserves element prefixes** (`<html:body>` stays `<html:body>`, where 0.1 wrote
  `<body>`), escapes text so that a literal carriage return and `]]>` survive a round trip, writes
  whitespace inside attribute values as character references, and merges adjacent text nodes.
* **Strictness**: the parser now rejects unclosed tags, more than one root element, content outside
  the root, `<` in an attribute value and `xmlns:p=""`. The XML declaration is interpreted (version
  1.0, UTF-8) instead of discarded. Comments and processing instructions outside the root element are
  discarded rather than silently dropped mid-tree.
* `XmlError` is a typed enum; `XmlError::InvalidXml`/`InvalidOperation`/`NamespaceError`,
  `QuickXmlError` and `ElementNotFound` are gone.
* `rust-version = "1.88"` is declared and verified.

### Added

* `Document::validate()` / `is_valid()`: whole-document validation that reports *every* problem at
  once (namespace scope, structural invariants, `xml:id` uniqueness and syntax, `xml:lang` and
  `xml:space` values).
* Structural editing: `detach`, `remove`, `insert_child`, `insert_before`, `insert_after`,
  `replace_with`, `clear_root`, and `_checked` twins for every fallible operation.
* Cloning: `deep_clone`, `shallow_clone`, `deep_clone_into`, `shallow_clone_into`.
* `Document::xml_declaration()`/`set_xml_declaration()`, `WriteOptions`, `write_*_with`.
* `NodeId`, `Document::node(id)`, `Document::nodes()`, `Document::node_count()`.
* `Element::namespaces_in_scope`, `remove_namespace_declaration`, `resolve_attribute_name` and the
  `xml_spec::rules`/`xml_spec::validation` modules.
* **The Python bindings**: `biodivine-lib-xml-dom-py-sys` (a thin PyO3 mirror) and the idiomatic
  `biodivine_lib_xml_dom` package, with the mapping audited in `docs/design/BINDINGS.md`.
* The documentation book, the Python API reference, `make verify` and the rule-enforcement inventory.

### Removed

* `parse_string_into`/`parse_bytes_into` (they never existed here; use `deep_clone_into` to import a
  tree into another document).
* The demo binary that lived in `src/main.rs` (now `examples/tour.rs`) and the stray `output.xml`.
