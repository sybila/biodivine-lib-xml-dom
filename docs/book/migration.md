# Migrating from 0.1

The 0.2 rewrite kept the names you know (`Document`, `Element`, `Namespace`, `QualifiedName`, the
`xml_spec` types, `parse_*`/`write_*`) but replaced the way nodes are stored, and that changes some
APIs. This chapter is the complete list; it is derived from the review and the design notes in
`docs/design/REVIEW.md` and `PLAN.md` §15, so it can be checked against them.

## The architecture changed

| 0.1 | 0.2 | why |
| --- | --- | --- |
| each node was an `Arc<RwLock<ElementData>>`, with `Arc` links up *and* down | one arena per document behind a single lock; handles are a document reference plus an index | the old design leaked every tree (parent ↔ child reference cycle) and its per-node locks made an edit non-atomic |
| `XmlNode` enum for children | `Node` handle plus `NodeKind`/`NodeContent` and typed accessors | one type for the tree API instead of a match on every access |
| a cycle could be created by two concurrent edits | impossible: the whole check-then-act is one critical section | see above |
| `Document::new` | `Document::empty` | says what it does (0.1 already had both) |

## Renamed and changed operations

| 0.1 | 0.2 |
| --- | --- |
| `doc.create_element(name)` | same (still the only way to make an element) |
| `parent.add_child_element(child)` → error if `child` had a parent | `parent.append_child(child)` → **moves** an attached child; `append_child_checked` for the fallible form |
| `element.add_attribute(name, value)` | `element.set_attribute(name, value)` (documented overwrite) + `set_attribute_checked` |
| `element.add_text(s)`, `add_comment(s)`, `add_cdata(s)`, `add_processing_instruction(t, d)` | `doc.create_text(s)` and friends, then `append_child` — node creation belongs to the document |
| `element.text_children()` / `comment_children()` / … | `element.children()` filtered through `Node::text()` / `comment()` / … , or `NodeKind` |
| `doc.set_root(root)` → `Result` | `doc.set_root(root)` → the *previous* root (panics on failure), `set_root_checked` for the fallible form |
| `element.declare_namespace(ns)` → error on a conflicting re-declaration | `declare_namespace(ns)` **overwrites** (documented default behaviour); `declare_namespace_checked` errors |
| `element.get_namespace(prefix)` (recursive) | same name, iterative; plus `namespaces_in_scope`, `remove_namespace_declaration`, `resolve_attribute_name` |
| `element.qualified_name()`, `local_name()`, `namespace()` | same |
| `element.is_ancestor(other)` | same (now also on `Node`) |
| `Element::new` (crate-internal) | gone; `Element` values come from `create_element`/`as_element` |

New in 0.2, and therefore worth knowing about:

* `detach`, `remove`, `insert_child`, `insert_before`, `insert_after`, `replace_with`, `clear_root`;
* `deep_clone`, `shallow_clone`, `deep_clone_into`, `shallow_clone_into` (element handles get
  element-typed results);
* `Document::validate()` / `is_valid()` — see [Validating a document](validation.md);
* `Document::xml_declaration()` / `set_xml_declaration()`, `WriteOptions`, `write_*_with`;
* `NodeId`, `Document::node(id)`, `Document::nodes()`, `Document::node_count()`;
* `XmlError` is a typed enum: matching on `XmlError::InvalidXml`/`InvalidOperation`/`NamespaceError`
  is replaced by the specific variants, and `QuickXmlError`/`ElementNotFound` are gone;
* `xml_spec::rules`, `xml_spec::validation` and the validated newtypes are public.

## Behaviour changes you may *rely* on

* the parser no longer panics: `&amp;` is expanded, and any other entity reference is a typed error;
* the serializer writes element prefixes (`<html:body>` stays `<html:body>`) and escapes text so
  that a literal carriage return and `]]>` survive;
* the XML declaration is interpreted (version 1.0, UTF-8) rather than discarded, and written back;
* the parser now rejects unclosed tags, more than one root, content outside the root, `<` in an
  attribute value and `xmlns:p=""`;
* comments and processing instructions outside the root element are discarded rather than kept (they
  have nowhere to live in the model) — see [Known limitations](limitations.md).

## Behaviour changes that may *break* you

* attaching a node that already has a parent moves it instead of returning an error, so check with
  `child.parent()` first if you relied on the error;
* `set_root` returns the previous root instead of `()`;
* `declare_namespace` overwrites instead of failing;
* `attribute_local(name)` and `resolve_attribute_name(name)` are the ways to reach
  no-namespace-attribute lookup and separate attribute resolution;
* `parse_string_into`/`parse_bytes_into` do not exist: import a tree with `deep_clone_into` instead;
* `element.clone()` still copies the handle (it always did), but there are now three *different*
  copy operations — see [Building documents](building-documents.md#copying).

## Python

The Python package is new in this version; there is no 0.1 Python API to migrate from. Its mapping
onto the Rust API, including what is deliberately not mirrored, is in `docs/design/BINDINGS.md`.
