//! XML DOM Library
//!
//! A thread-safe XML DOM library with full namespace support, reference counting,
//! and read-write locks for memory management.
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
//! use biodivine_lib_xml_dom::{create_document, Namespace, QualifiedName};
//!
//! // Create a new document and elements in a single block
//! let doc = create_document();
//! let html_ns = Namespace::prefixed("http://www.w3.org/1999/xhtml", "html").unwrap();
//! let root = doc.create_element(QualifiedName::with_namespace("html", &html_ns).unwrap());
//! root.declare_namespace("html".to_string(), html_ns.clone());
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
//! assert_eq!(root.name(), "root");
//! ```
//!
//! ## Working with Comments
//!
//! ```rust
//! use biodivine_lib_xml_dom::{create_document, write_string, QualifiedName, parse_string};
//!
//! let doc = create_document();
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
//! use biodivine_lib_xml_dom::{create_document, write_string};
//! let doc = create_document();
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

// Re-export public API
pub use document::Document;
pub use element::Element;
pub use error::{XmlError, XmlResult};
pub use io::{parse_file, parse_reader, parse_string, write_file, write_string, write_writer};
pub use namespace::Namespace;
pub use qualified_name::QualifiedName;

/// Main entry point for the library
///
/// Creates a new empty XML document with thread-safe operations.
///
/// # Returns
///
/// A new `Document` instance ready for use.
///
/// # Example
///
/// ```rust
/// use biodivine_lib_xml_dom::create_document;
///
/// let doc = create_document();
/// assert!(doc.root().is_none());
/// ```
pub fn create_document() -> Document {
    Document::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_document() {
        let doc = create_document();
        assert!(doc.root().is_none());
    }

    #[test]
    fn test_create_element() {
        let doc = create_document();
        let element = doc.create_element(QualifiedName::without_namespace("test").unwrap());
        assert_eq!(element.name(), "test");
        assert!(element.namespace().is_none());
    }

    #[test]
    fn test_add_children() {
        let doc = create_document();
        let parent = doc.create_element(QualifiedName::without_namespace("parent").unwrap());
        let child = doc.create_element(QualifiedName::without_namespace("child").unwrap());

        parent.add_child_element(child.clone()).unwrap();

        let children = parent.children();
        assert_eq!(children.len(), 1);
        match &children[0] {
            crate::element::XmlNode::Element(e) => assert_eq!(e.name(), "child"),
            crate::element::XmlNode::Text(_) => panic!("Expected element child, got text"),
            crate::element::XmlNode::Comment(_) => panic!("Expected element child, got comment"),
            crate::element::XmlNode::CData(_) => panic!("Expected element child, got cdata"),
            crate::element::XmlNode::ProcessingInstruction(_, _) => {
                panic!("Expected element child, got processing instruction")
            }
        }
        assert!(child.is_attached());
    }

    #[test]
    fn test_namespace_declaration() {
        let doc = create_document();
        let root = doc.create_element(QualifiedName::without_namespace("root").unwrap());
        root.declare_namespace(
            "ex".to_string(),
            Namespace::prefixed("http://example.com", "ex").unwrap(),
        );

        assert_eq!(
            root.get_namespace("ex"),
            Some(Namespace::prefixed("http://example.com", "ex").unwrap())
        );
    }

    #[test]
    fn test_qualified_name_resolution() {
        let doc = create_document();
        let root = doc.create_element(QualifiedName::without_namespace("root").unwrap());
        root.declare_namespace(
            "ex".to_string(),
            Namespace::prefixed("http://example.com", "ex").unwrap(),
        );

        let (local_name, namespace) = root.resolve_qualified_name("ex:test").unwrap();
        assert_eq!(local_name, "test");
        assert_eq!(namespace.unwrap().uri(), "http://example.com");
    }

    #[test]
    fn test_document_reference() {
        let doc = create_document();
        let element = doc.create_element(QualifiedName::without_namespace("test").unwrap());

        // Set as root should work
        doc.set_root(element).unwrap();
        assert!(doc.root().is_some());
    }
}
