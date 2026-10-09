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
//! public handles — [`Node`] (any node) and [`Element`] (an element) — are nothing but an id into
//! that arena plus a clone of the [`Document`] handle. Cloning a handle therefore refers to the
//! same node; the operations that actually duplicate data are [`Node::shallow_clone`] and
//! [`Node::deep_clone`], and [`Node::deep_clone_into`] copies a subtree into another document.
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
//! * *Document-wide* properties are the subject of a separate validation pass.
//!
//! ## Scope
//!
//! XML 1.0 (fifth edition) and Namespaces in XML 1.0 (third edition), UTF-8 only. `DOCTYPE`
//! declarations are read but never processed, so no DTD-defined entity, attribute type or content
//! model is available.
//!
//! # Examples
//!
//! ## Building and serializing a document
//!
//! ```rust
//! use biodivine_lib_xml_dom::{Document, Namespace, QualifiedName, write_string};
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
//! let xml = write_string(&document).unwrap();
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
//! use biodivine_lib_xml_dom::{parse_string, write_string};
//!
//! let document = parse_string(
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
//! assert!(write_string(&document).unwrap().contains("one"));
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
mod element;
mod error;
mod interner;
pub mod io;
mod namespace;
mod node;
mod qualified_name;

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

pub use crate::arena::{MAX_NODES, NodeId};
pub use crate::document::Document;
pub use crate::element::Element;
pub use crate::error::{XmlError, XmlResult};
pub use crate::io::{
    DeclarationStyle, EmptyElementStyle, WriteOptions, parse_bytes, parse_file, parse_reader,
    parse_string, write_file, write_file_with, write_string, write_string_with, write_writer,
    write_writer_with,
};
pub use crate::namespace::Namespace;
pub use crate::node::{Node, NodeContent, NodeKind};
pub use crate::qualified_name::QualifiedName;

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
