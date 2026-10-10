//! # Biodivine XML DOM
//!
//! A Rust library for building, parsing, and manipulating XML documents through a familiar
//! Document Object Model (DOM) API.
//!
//! The library is designed for **document manipulation** rather than high-performance streaming.
//! It trades raw throughput for ergonomics: you can freely create, modify, traverse, clone and
//! serialize XML trees with full namespace awareness, and a single document can be shared between
//! threads.
//!
//! ## The model
//!
//! A [`Document`] is an *arena*: every node payload of one document lives in one vector, and the
//! public handles — [`Node`] (any node) and [`Element`] (an element) — are a counted reference to
//! one arena slot plus a clone of the [`Document`] handle. Cloning a handle therefore refers to the
//! same node (and keeps it alive); the operations that actually duplicate data are
//! [`Node::shallow_clone`] and [`Node::deep_clone`], and [`Node::deep_clone_into`] copies a subtree
//! into another document. Nodes that nothing refers to are reclaimed by
//! [`Document::collect_garbage`].
//!
//! ```text
//! Document ── Arc ──▶ RwLock<Arena>          one lock per document
//!                        ├─ nodes: Vec<NodeData>   (all payloads)
//!                        └─ root:  Option<NodeId>
//! Node/Element = { Document, NodeId }          cheap, Send + Sync
//! ```
//!
//! ## Thread safety
//!
//! Each document owns exactly **one** reader-writer lock. Every method acquires it, does its work
//! and releases it; internal helpers work directly on the arena and cannot lock. Since there is
//! only a single lock and it is never re-acquired while held, there is no lock ordering that could
//! deadlock. Debug builds additionally assert the invariant with a re-entrancy guard.
//!
//! Operations that can fail for *logical* reasons (creating a cycle, attaching a node of another
//! document, …) come in two flavours: a `_checked` variant returning [`XmlResult`], and an
//! ergonomic variant that panics with a message describing the failure.
//!
//! ## Namespaces
//!
//! Element and attribute names are *expanded names* ([`QualifiedName`]: a local name plus an
//! optional [`Namespace`] holding both the prefix and the URI). Editing operations never add,
//! remove or rewrite namespace declarations; a document that becomes inconsistent this way is
//! reported by whole-document validation rather than repaired behind your back.
//!
//! ## Integrity
//!
//! * *Locally* decidable properties are enforced at construction: [`xml_spec::NCName`],
//!   [`xml_spec::Text`], [`xml_spec::Comment`], [`xml_spec::CData`], [`xml_spec::PiTarget`] and
//!   [`xml_spec::PiData`] can only be built from valid input, attributes are keyed by expanded
//!   name (so duplicates cannot be stored), and the parser turns violations into typed errors
//!   instead of producing a broken document.
//! * *Document-wide* properties are the subject of a separate validation pass; see
//!   [Validation](#validation) below.
//!
//! ## Scope
//!
//! XML 1.0 (fifth edition) and Namespaces in XML 1.0 (third edition), UTF-8 only. `DOCTYPE`
//! declarations are read but never processed, so no DTD-defined entity, attribute type or content
//! model is available.
//!
//! ## Reading and writing documents
//!
//! Parsing and serialization are methods of [`Document`], so the whole life cycle of a document —
//! create, parse, edit, validate, serialize — is reachable from one type:
//!
//! | task | entry point | options |
//! | --- | --- | --- |
//! | parse a string | [`Document::parse_string`] | — |
//! | parse bytes | [`Document::parse_bytes`] | — |
//! | parse a reader | [`Document::parse_reader`] | — |
//! | parse a file | [`Document::parse_file`] | — |
//! | serialize to a string | [`Document::write_string`], [`Document::write_string_with`] | [`WriteOptions`] |
//! | serialize to a file | [`Document::write_file`], [`Document::write_file_with`] | [`WriteOptions`] |
//! | serialize to a writer | [`Document::write_writer`], [`Document::write_writer_with`] | [`WriteOptions`] |
//!
//! Parsing is strict: the input has to be a well-formed XML 1.0 document with exactly one root
//! element, and every failure is a typed [`XmlError`] rather than a panic or a half-built document.
//! Since the data model is one root element plus the nodes below it, content that appears *outside*
//! the root element is handled as follows:
//!
//! | input outside the root | result |
//! | --- | --- |
//! | whitespace | ignored |
//! | a comment | **discarded** (the model has no place for it) |
//! | a processing instruction | **discarded** (the model has no place for it) |
//! | non-whitespace text | [`XmlError::ContentOutsideRoot`] |
//! | a second element | [`XmlError::MultipleRootElements`] |
//! | no element at all | [`XmlError::MissingRoot`] |
//!
//! Serialization is *lossless* for everything the data model can represent:
//!
//! * names are written with the prefix they carry, so `<html:body>` stays `<html:body>`;
//! * namespace declarations are written exactly as they are stored — the serializer never adds,
//!   removes or rewrites one, so an inconsistent document stays inconsistent until
//!   [`Document::validate`] reports it;
//! * text and attribute values are escaped so that `&`, `<`, `]]>` and a literal carriage return
//!   survive a round trip;
//! * adjacent text nodes are merged, so the output does not depend on how the tree was built; an
//!   *empty* text node is omitted, because XML has no representation for it;
//! * serialization is iterative, so a deeply nested document cannot overflow the stack.
//!
//! Because the serializer is not "smart", a document whose names use a prefix that no declaration
//! in scope provides is written as such, and re-parsing it fails. Run [`Document::validate`] before
//! serializing a document that was assembled by hand.
//!
//! ## Validation
//!
//! Editing operations deliberately do *not* keep the document consistent: removing a declaration
//! that a subtree relies on, or moving an element into a scope where its prefix means something
//! else, succeeds silently and quickly. [`Document::validate`] is the counterpart: it checks the
//! whole document in one pass and reports *every* problem it can find, so a caller can fix them all
//! at once instead of discovering them one failed operation at a time.
//!
//! What it checks:
//!
//! * **namespace scope and declarations** — every prefix that a name uses is declared in scope,
//!   the reserved `xml` and `xmlns` bindings are respected, and default-namespace rules hold
//!   (attributes never take the default namespace);
//! * **values** — `xml:id` syntax and uniqueness, `xml:lang` values are BCP 47 or empty, `xml:space`
//!   values are one of the two enumerated values;
//! * **structure** — the document has at most one root element and the stored tree is consistent.
//!
//! What it deliberately does not check: everything that would need DTD processing (attribute
//! *types*, content models, and the "must be declared" constraints on `xml:lang`/`xml:space`).
//! That is validity rather than well-formedness, and `DOCTYPE` support is out of scope for this
//! library.
//!
//! Detached nodes are normal, so being detached is never an error by itself. Node-local rules (name
//! resolution against the node's own scope, `xml:lang`/`xml:space` values, `xml:id` syntax) are
//! checked for detached nodes too, which means a subtree can be prepared and validated before it is
//! attached. Document-wide rules (`xml:id` uniqueness) are checked over the tree below the root
//! only: a detached copy of a subtree would otherwise be reported as a duplicate of its original.
//!
//! See [`ValidationErrorKind`] for the individual findings and [`XmlValidationErrors`] for the
//! collection a failed validation returns.
//!
//! # Examples
//!
//! ## Building and serializing a document
//!
//! ```rust
//! use biodivine_lib_xml_dom::{Document, Namespace, QualifiedName};
//!
//! let document = Document::empty();
//! let html = Namespace::prefixed("http://www.w3.org/1999/xhtml", "html").unwrap();
//! let root = document.create_element(QualifiedName::with_namespace("html", &html).unwrap());
//! root.declare_namespace(html.clone());
//! document.set_root(root.clone());
//!
//! let body = document.create_element(QualifiedName::with_namespace("body", &html).unwrap());
//! body.set_attribute(QualifiedName::without_namespace("class").unwrap(), "main");
//! body.append_child(document.create_text("Hello, World!").unwrap());
//! root.append_child(body);
//!
//! let xml = document.write_string().unwrap();
//! assert!(xml.contains("Hello, World!"));
//! assert_eq!(
//!     root.child_elements()[0].namespace().unwrap().uri(),
//!     "http://www.w3.org/1999/xhtml"
//! );
//! ```
//!
//! ## Parsing and traversing
//!
//! ```rust
//! use biodivine_lib_xml_dom::Document;
//!
//! let document = Document::parse_string(
//!     r#"<root xmlns:ex="http://example.com"><ex:item>one</ex:item></root>"#,
//! )
//! .unwrap();
//!
//! let root = document.root().unwrap();
//! assert_eq!(root.local_name(), "root");
//!
//! let item = &root.child_elements()[0];
//! assert_eq!(item.qualified_name().to_string(), "ex:item");
//! assert_eq!(item.namespace().unwrap().uri(), "http://example.com");
//!
//! assert!(document.write_string().unwrap().contains("one"));
//! ```
//!
//! ## Detached nodes and multiple documents
//!
//! ```rust
//! use biodivine_lib_xml_dom::{Document, QualifiedName};
//!
//! let first = Document::empty();
//! let second = Document::empty();
//!
//! let root = first.create_element(QualifiedName::without_namespace("root").unwrap());
//! first.set_root(root.clone());
//!
//! // A freshly created node is detached and can be prepared before it is attached.
//! let child = first.create_element(QualifiedName::without_namespace("child").unwrap());
//! assert!(!child.is_attached());
//! root.append_child(child.clone());
//! assert!(child.is_attached());
//!
//! // Nodes cannot travel between documents ...
//! let foreign_parent = second.create_element(QualifiedName::without_namespace("other").unwrap());
//! assert!(foreign_parent.append_child_checked(child.clone()).is_err());
//!
//! // ... but a subtree can be copied.
//! let copy = root.deep_clone_into(&second);
//! assert!(copy.belongs_to(&second));
//! assert_eq!(copy.children().len(), 1);
//! ```
//!
//! ## Sharing between threads
//!
//! ```rust
//! use biodivine_lib_xml_dom::{Document, QualifiedName};
//!
//! let document = Document::empty();
//! let root = document.create_element(QualifiedName::without_namespace("root").unwrap());
//! document.set_root(root.clone());
//!
//! let handles: Vec<_> = (0..4)
//!     .map(|index| {
//!         let document = document.clone();
//!         std::thread::spawn(move || {
//!             let node = document
//!                 .create_element(QualifiedName::without_namespace("item").unwrap());
//!             document.root().unwrap().append_child_checked(node).unwrap();
//!             index
//!         })
//!     })
//!     .collect();
//!
//! for handle in handles {
//!     handle.join().unwrap();
//! }
//! assert_eq!(root.children().len(), 4);
//! ```

mod arena;
mod document;
pub mod documentation;
mod element;
mod error;
mod interner;
mod io;
mod namespace;
mod node;
mod node_id;
mod qualified_name;
mod ref_count;
mod validation;

/// XML specification types and checks.
///
/// This module holds everything that exists because of the XML 1.0 and Namespaces in XML 1.0
/// specifications: the validated string newtypes ([`NCName`](xml_spec::NCName),
/// [`Text`](xml_spec::Text), [`Comment`](xml_spec::Comment), [`CData`](xml_spec::CData),
/// [`PiTarget`](xml_spec::PiTarget), [`PiData`](xml_spec::PiData)), reserved prefix and URI
/// constants, and the name/namespace predicates the rest of the crate builds on.
///
/// Keeping specification logic here (rather than next to the code that happens to use it) is the
/// convention this project follows, together with annotating rules with the identifier of the
/// corresponding file in `specification/rules/`.
pub mod xml_spec;

pub use crate::arena::{GcConfig, GcStats};
pub use crate::document::Document;
pub use crate::element::Element;
pub use crate::error::{XmlError, XmlResult};
pub use crate::io::{DeclarationStyle, EmptyElementStyle, WriteOptions};
pub use crate::namespace::Namespace;
pub use crate::node::{Node, NodeContent, NodeKind};
pub use crate::node_id::{MAX_NODES, NodeId};
pub use crate::qualified_name::QualifiedName;
pub use crate::validation::{ValidationErrorKind, XmlValidationError, XmlValidationErrors};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml_spec::nc_name;

    #[test]
    fn test_create_document() {
        let document = Document::empty();
        assert!(document.root().is_none());
    }

    #[test]
    fn test_create_element() {
        let document = Document::empty();
        let element = document.create_element(QualifiedName::without_namespace("test").unwrap());
        assert_eq!(element.local_name(), "test");
        assert!(element.namespace().is_none());
        assert!(!element.is_attached());
    }

    #[test]
    fn test_add_children() {
        let document = Document::empty();
        let parent = document.create_element(QualifiedName::without_namespace("parent").unwrap());
        let child = document.create_element(QualifiedName::without_namespace("child").unwrap());

        parent.append_child(child.clone());
        document.set_root(parent.clone());

        let children = parent.children();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].kind(), NodeKind::Element);
        assert_eq!(children[0].as_element().unwrap().local_name(), "child");
        assert!(child.is_attached());
    }

    #[test]
    fn test_namespace_declaration() {
        let document = Document::empty();
        let root = document.create_element(QualifiedName::without_namespace("root").unwrap());
        root.declare_namespace(Namespace::prefixed("http://example.com", "ex").unwrap());

        assert_eq!(
            root.get_namespace(Some(&nc_name("ex"))),
            Some(Namespace::prefixed("http://example.com", "ex").unwrap())
        );
    }

    #[test]
    fn test_qualified_name_resolution() {
        let document = Document::empty();
        let root = document.create_element(QualifiedName::without_namespace("root").unwrap());
        root.declare_namespace(Namespace::prefixed("http://example.com", "ex").unwrap());

        let resolved = root.resolve_qualified_name("ex:test").unwrap();
        assert_eq!(resolved.local_name(), "test");
        assert_eq!(resolved.namespace().unwrap().uri(), "http://example.com");
    }

    #[test]
    fn test_document_reference() {
        let document = Document::empty();
        let element = document.create_element(QualifiedName::without_namespace("test").unwrap());
        document.set_root(element);
        assert!(document.root().is_some());
    }

    #[test]
    fn test_dropping_a_document_frees_its_nodes() {
        use std::sync::{Arc as StdArc, Weak};

        // Regression test for REVIEW D4: with the previous per-node `Arc` design, the
        // parent <-> child links formed a reference cycle and the whole tree stayed alive.
        // The arena stores indices instead, so dropping every handle must free the document.
        let weak: Weak<crate::document::DocumentInner> = {
            let document = Document::empty();
            let root = document.create_element(QualifiedName::without_namespace("root").unwrap());
            let child = document.create_element(QualifiedName::without_namespace("child").unwrap());
            root.append_child(child.clone());
            document.set_root(root.clone());
            root.clone()
                .declare_namespace(Namespace::without_prefix("http://example.com").unwrap());

            // One clone per live handle: the original `document` plus `root` and `child`.
            assert_eq!(StdArc::strong_count(&document.inner), 3);
            StdArc::downgrade(&document.inner)
        };

        assert!(
            weak.upgrade().is_none(),
            "the document is still alive after every handle was dropped: nodes leak"
        );
    }
}
