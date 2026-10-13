#![deny(rustdoc::broken_intra_doc_links, rustdoc::private_intra_doc_links)]
//! # Biodivine XML DOM
//!
//! Biodivine XML DOM is a Rust library for building, parsing, editing, validating and serializing
//! XML documents through a familiar Document Object Model (DOM) API, with full namespace awareness
//! and documents that can be shared between threads.
//!
//! It is built for **document manipulation** rather than high-throughput streaming: a document is a
//! tree in memory that you can take apart and put together again, and the library's job is to make
//! that pleasant and hard to get wrong. Names are expanded names rather than prefix strings, the
//! properties that can be decided locally are enforced while a document is built, everything that
//! needs a whole-document view is reported by [one validation pass](#validation), and unreachable
//! nodes are reclaimed by a [garbage collector](#garbage-collection-and-node-ids).
//!
//! ## A quick tour
//!
//! Building a document and writing it out:
//!
//! ```rust
//! use biodivine_lib_xml_dom::{Document, Namespace, QualifiedName};
//!
//! let document = Document::new();
//! let html = Namespace::prefixed("http://www.w3.org/1999/xhtml", "html").unwrap();
//!
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
//! assert!(xml.contains(r#"<html:body class="main">Hello, World!</html:body>"#));
//! assert!(document.is_valid().unwrap());
//! ```
//!
//! Parsing a document and walking it:
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
//! let item = &root.child_elements()[0];
//!
//! assert_eq!(root.local_name(), "root");
//! assert_eq!(item.qualified_name().to_string(), "ex:item");
//! assert_eq!(item.namespace().unwrap().uri(), "http://example.com");
//! assert_eq!(item.children()[0].text().unwrap().as_str(), "one");
//! ```
//!
//! ## The model
//!
//! A [`Document`] is a cheap handle onto shared state, and that state owns one *arena*: a vector of
//! node slots behind a single reader-writer lock. Nodes refer to each other by slot index, so
//! internal links are free and cannot form a reference cycle. [`Node`] (any node) and [`Element`]
//! (an element) are a document handle plus a counted reference to one slot; cloning a handle refers
//! to the same node and keeps it alive, and the operations that duplicate *data* are
//! [`Node::shallow_clone`], [`Node::deep_clone`] and [`Node::deep_clone_into`] (which copies a
//! subtree into another document).
//!
#![doc = include_str!("../docs/assets/model.svg")]
//!
//! ## Garbage collection and node ids
//!
//! A node lives while something refers to it from outside the document: a [`Node`]/[`Element`]
//! handle, or the document root itself, which the document keeps alive on its own. Detaching a node,
//! dropping a subtree or discarding a copy therefore does not leak: a mark-and-sweep collection
//! reclaims every node that no handle and no root can reach, and its slot is handed out again by a
//! later allocation. Collection runs automatically when an arena is full and can be requested with
//! [`Document::collect_garbage`]; the capacity policy is [`GcConfig`].
//!
//! [`NodeId`] is an arena slot number. It is deliberately *not* a usable reference on its own: the
//! arena never moves a slot, but a reclaimed slot is reused, so an id only means something while the
//! node it was created for is alive. [`Document::node`] resolves an id to a [`Node`] — returning
//! `None` when the slot holds nothing — and a handle, unlike an id, keeps its node alive.
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
//! | serialize to a writer | [`Document::write_to`], [`Document::write_to_with`] | [`WriteOptions`] |
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
//! * **structure** — the document has a root element. (That it has *only* one, and that the tree
//!   is a tree, is guaranteed by construction.)
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
//! A failed validation returns [`XmlError::Validation`], which carries the [`ValidationErrors`];
//! [`Document::validation_errors`] returns the same findings as a plain list, and
//! [`ValidationErrorKind`] describes each of them.
//!
//! ## Cancellation
//!
//! The operations that take time proportional to the document (parsing, serialization,
//! validation, deep copies and explicit garbage collection) cooperate with the [`cancel_this`]
//! crate. A trigger registered on the current thread with [`cancel_this::on_trigger`] (or one of
//! its helpers, such as [`cancel_this::on_timeout`] or [`cancel_this::on_atomic`]) is consulted
//! periodically, and a cancelled operation returns [`XmlError::Cancelled`] instead of running to
//! completion. Triggers are thread-local: a trigger registered on one thread never cancels an
//! operation running on another.
//!
//! A cancelled operation leaves the document unchanged, with one exception that is in the nature
//! of streaming output: a cancelled [`Document::write_file`] or [`Document::write_to`] has already
//! written part of the document to its destination. In detail:
//!
//! | operation | what a cancellation leaves behind |
//! | --- | --- |
//! | parsing | nothing: no document is returned |
//! | validation | nothing: no report is returned |
//! | [`Node::deep_clone`], [`Node::deep_clone_into`] | nothing: the part of the copy built so far is freed again |
//! | [`Document::collect_garbage`] | nothing: only the read-only search for reachable nodes is cancellable, never the reclaiming |
//! | serialization | the output written so far |
//!
//! Cheap operations are never cancelled: single-node edits, [`Node::shallow_clone`], and the
//! `Display` implementations, which cannot report an error.
//!
//! ```rust
//! use biodivine_lib_xml_dom::{cancel_this, Document, XmlError};
//!
//! let document = Document::parse_string("<root><a/></root>").unwrap();
//!
//! // A trigger that is already cancelled aborts the walk immediately.
//! let trigger = cancel_this::CancelAtomic::new();
//! trigger.cancel();
//! let result = cancel_this::on_atomic(trigger, || document.validate());
//! assert!(matches!(result, Err(XmlError::Cancelled(_))));
//! ```
//!
//! The Python bindings use the same mechanism to let Ctrl+C (or any other Python signal handler
//! that raises) interrupt a long operation.
//!
//! ## More examples
//!
//! ### Detached nodes and multiple documents
//!
//! ```rust
//! use biodivine_lib_xml_dom::{Document, QualifiedName};
//!
//! let first = Document::new();
//! let second = Document::new();
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
//! let copy = root.deep_clone_into(&second).unwrap();
//! assert!(copy.belongs_to(&second));
//! assert_eq!(copy.children().len(), 1);
//! ```
//!
//! ### Sharing between threads
//!
//! ```rust
//! use biodivine_lib_xml_dom::{Document, QualifiedName};
//!
//! let document = Document::new();
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
//!
//! ## Where to look next
//!
//! | resource | what it gives you |
//! | --- | --- |
//! | [`Document`] | the entry point for the whole life cycle: create, parse, edit, validate, serialize |
//! | [`Node`], [`Element`], [`QualifiedName`], [`Namespace`] | the handles and names you work with |
//! | [`Document::validate`] and [`ValidationErrorKind`] | whole-document validation and its findings |
//! | [`xml_spec`] | the specification types, the predicates, and [`xml_spec::rule_set`] with one module per XML rule |
//! | [`documentation`] | the design notes, the Rust/Python mapping and the [rule coverage](documentation::rule_coverage) |
//! | the documentation book that ships with the repository | tutorials with Rust and Python examples side by side |
//! | the `biodivine_lib_xml_dom` Python package | the same API with Pythonic wrappers; see [the bindings map](documentation::bindings) |
//! | the `examples` directory | runnable programs, including the ones the book embeds |

mod arena;
mod cancellation;
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
/// [`PiTarget`](xml_spec::PiTarget), [`PiData`](xml_spec::PiData)), the
/// [`XmlDeclaration`](xml_spec::XmlDeclaration), the reserved namespace URIs, the predicates for
/// attribute values that only the specification defines, and [`rule_set`](xml_spec::rule_set),
/// the summaries of the specification rules that the rest of the documentation links to.
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
pub use crate::validation::{ValidationError, ValidationErrorKind, ValidationErrors};

/// The cancellation crate this library cooperates with, re-exported so that a caller can register
/// triggers with exactly the version this library checks.
///
/// See [cancellation](crate#cancellation) for which operations consult the triggers.
pub use cancel_this;

// The examples of the README are compiled and run as documentation tests.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dropping_a_document_frees_its_nodes() {
        use std::sync::{Arc as StdArc, Weak};

        // The arena stores indices rather than reference-counted links between nodes, so there is
        // no parent <-> child reference cycle: dropping every handle must free the document.
        let weak: Weak<crate::document::DocumentInner> = {
            let document = Document::new();
            let root = document
                .create_element(QualifiedName::without_namespace("root").unwrap());
            let child = document
                .create_element(QualifiedName::without_namespace("child").unwrap());
            root.append_child(child.clone());
            document.set_root(root.clone());
            root.clone().declare_namespace(
                Namespace::without_prefix("http://example.com").unwrap(),
            );

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
