//! # Biodivine XML DOM
//!
//! A Rust library for building, parsing, and manipulating XML documents through a
//! familiar Document Object Model (DOM) API.
//!
//! This library is designed for **document manipulation** rather than high-performance
//! parsing. If you need to process large XML files at scale, consider a streaming parser
//! instead. Biodivine XML DOM trades raw throughput for flexibility: you can freely create,
//! modify, traverse, and serialize XML trees with full namespace awareness and thread-safe
//! shared ownership via `Arc` and `RwLock`.
//!
//! ## Public API
//!
//! The library exposes a small, focused surface area:
//!
//! - **[`Document`]** — the root handle for an XML document. Create elements, set the root,
//!   and coordinate the document tree.
//! - **[`Element`]** — represents an XML node with children, attributes, text, and comments.
//! - **[`Namespace`]** and **[`QualifiedName`]** — types for working with XML namespaces.
//! - **I/O** — [`parse_string`], [`parse_file`], [`parse_reader`] for reading;
//!   [`write_string`], [`write_file`], [`write_writer`] for serialization.
//!
//! ## Limitations
//!
//! This library targets **XML 1.0** with **UTF-8 encoding** only. Other XML versions,
//! encodings, or DTD-based validation are not supported.
//!
//! # Features
//!
//! - **Thread Safety**: Uses `Arc` and `RwLock` for safe concurrent access
//! - **Namespace Support**: Full XML namespace support with scoped declarations
//! - **Memory Management**: Reference counting for automatic cleanup
//! - **Parsing & Writing**: Parse from files/strings and write back to XML
//! - **Element Manipulation**: Create, modify, and traverse XML elements
//! - **Comment Support**: Full support for XML comments in parsing and serialization
//!
//! # Examples
//!
//! ## Creating and manipulating XML
//!
//! ```rust
//! use biodivine_lib_xml_dom::{Document, Namespace, QualifiedName};
//! use biodivine_lib_xml_dom::xml_spec::nc_name;
//!
//! // Create a new document and elements in a single block
//! let doc = Document::empty();
//! let html_ns = Namespace::prefixed("http://www.w3.org/1999/xhtml", "html").unwrap();
//! let root = doc.create_element(QualifiedName::with_namespace("html", &html_ns).unwrap());
//! root.declare_namespace(html_ns.clone());
//! doc.set_root(root.clone()).unwrap();
//! let body = doc.create_element(QualifiedName::without_namespace("body").unwrap());
//! body.add_attribute(QualifiedName::without_namespace("class").unwrap(), "main".to_string());
//! body.add_text("Hello, World!".to_string());
//! root.add_child_element(body).unwrap();
//! ```
//!
//! ## Parsing XML
//!
//! ```rust
//! use biodivine_lib_xml_dom::parse_string;
//!
//! let xml = r#"<root xmlns:ex="http://example.com">
//!     <ex:element>Hello, World!</ex:element>
//! </root>"#;
//!
//! let doc = parse_string(xml).unwrap();
//! let root = doc.root().unwrap();
//! assert_eq!(root.qualified_name().local_name(), "root");
//! ```
//!
//! ## Working with Comments
//!
//! ```rust
//! use biodivine_lib_xml_dom::{Document, write_string, QualifiedName, parse_string};
//!
//! let doc = Document::empty();
//! let root = doc.create_element(QualifiedName::without_namespace("root").unwrap());
//! doc.set_root(root.clone()).unwrap();
//!
//! // Add comments to elements
//! root.add_comment(" This is a comment ".to_string());
//! root.add_text("Some content".to_string());
//!
//! // Parse XML with comments
//! let xml_with_comments = r#"<root><!-- Comment --><child>Content</child></root>"#;
//! let parsed_doc = parse_string(xml_with_comments).unwrap();
//! let parsed_root = parsed_doc.root().unwrap();
//! let comments = parsed_root.comment_children();
//! assert_eq!(comments.len(), 1);
//! ```
//!
//! ## Writing XML
//!
//! ```
//!
//! use biodivine_lib_xml_dom::{Document, write_string};
//! let doc = Document::empty();
//! // ... build document ...
//! let xml = write_string(&doc).unwrap();
//! ```

// Module declarations
mod document;
mod element;
mod error;
mod io;
mod namespace;
mod qualified_name;

/// Module containing XML specification-enforced validation logic and constants.
///
/// This module separates rules mandated by XML 1.0 and XML Namespaces specifications
/// (NCName validation, reserved prefixes/URIs, namespace binding rules) from library
/// design choices like `Arc`-backed immutability or convenience constructors. Tests here
/// verify compliance against the specification rule files in `specification/rules/`.
///
/// Most of the validation rules are only internal, but we do export some utility types (e.g.,
/// [`xml_spec::NCName`], [`xml_spec::Text`], [`xml_spec::Comment`], [`xml_spec::CData`],
/// [`xml_spec::PiTarget`], [`xml_spec::PiData`]) that are part of the public API.
///
// Note: Types from this module are not re-exported because we want this to be explicitly separate
// from the main library implementation.
pub mod xml_spec;

// Re-export public API
pub use document::Document;
pub use element::{Element, XmlNode};
pub use error::{XmlError, XmlResult};
pub use io::{parse_file, parse_reader, parse_string, write_file, write_string, write_writer};
pub use namespace::Namespace;
pub use qualified_name::QualifiedName;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml_spec::nc_name;

    #[test]
    fn test_create_document() {
        let doc = Document::empty();
        assert!(doc.root().is_none());
    }

    #[test]
    fn test_create_element() {
        let doc = Document::empty();
        let element = doc.create_element(QualifiedName::without_namespace("test").unwrap());
        assert_eq!(element.qualified_name().local_name(), "test");
        assert!(element.qualified_name().namespace().is_none());
    }

    #[test]
    fn test_add_children() {
        let doc = Document::empty();
        let parent = doc.create_element(QualifiedName::without_namespace("parent").unwrap());
        let child = doc.create_element(QualifiedName::without_namespace("child").unwrap());

        parent.add_child_element(child.clone()).unwrap();
        doc.set_root(parent.clone()).unwrap();

        let children = parent.children();
        assert_eq!(children.len(), 1);
        match &children[0] {
            XmlNode::Element(e) => assert_eq!(e.qualified_name().local_name(), "child"),
            XmlNode::Text(_) => panic!("Expected element child, got text"),
            XmlNode::Comment(_) => panic!("Expected element child, got comment"),
            XmlNode::CData(_) => panic!("Expected element child, got cdata"),
            XmlNode::ProcessingInstruction(_, _) => {
                panic!("Expected element child, got processing instruction")
            }
        }
        assert!(child.is_attached());
    }

    #[test]
    fn test_namespace_declaration() {
        let doc = Document::empty();
        let root = doc.create_element(QualifiedName::without_namespace("root").unwrap());
        root.declare_namespace(Namespace::prefixed("http://example.com", "ex").unwrap())
            .unwrap();

        assert_eq!(
            root.get_namespace(Some(&nc_name("ex"))),
            Some(Namespace::prefixed("http://example.com", "ex").unwrap())
        );
    }

    #[test]
    fn test_qualified_name_resolution() {
        let doc = Document::empty();
        let root = doc.create_element(QualifiedName::without_namespace("root").unwrap());
        root.declare_namespace(Namespace::prefixed("http://example.com", "ex").unwrap())
            .unwrap();

        let resolved = root.resolve_qualified_name("ex:test").unwrap();
        assert_eq!(resolved.local_name(), "test");
        assert_eq!(resolved.namespace().unwrap().uri(), "http://example.com");
    }

    #[test]
    fn test_document_reference() {
        let doc = Document::empty();
        let element = doc.create_element(QualifiedName::without_namespace("test").unwrap());

        // Set as root should work
        doc.set_root(element).unwrap();
        assert!(doc.root().is_some());
    }
}
