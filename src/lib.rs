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
//!
//! # Examples
//!
//! ## Creating and manipulating XML
//!
//! ```rust
//! use biodivine_lib_xml_dom::{create_document, Attribute, Namespace};
//!
//! // Create a new document
//! let doc = create_document();
//!
//! // Create elements
//! let html_ns = Namespace::prefixed("http://www.w3.org/1999/xhtml".to_string(), "html".to_string());
//! let root = doc.create_element_with_namespace("html".to_string(), html_ns);
//! 
//! // Declare namespaces on elements
//! root.declare_namespace("html".to_string(), "http://www.w3.org/1999/xhtml".to_string());
//! doc.set_root(root.clone()).unwrap();
//!
//! // Add attributes and content
//! let body = doc.create_element("body".to_string());
//! body.add_attribute(Attribute::new("class".to_string(), "main".to_string()));
//! body.set_text_content("Hello, World!".to_string());
//! root.add_child(body).unwrap();
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
//! ## Writing XML
//!
//! ```rust
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

// Re-export public API
pub use document::Document;
pub use element::Element;
pub use error::{XmlError, XmlResult};
pub use io::{parse_file, parse_reader, parse_string, write_file, write_string, write_writer};
pub use namespace::{Attribute, Namespace};

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
        let element = doc.create_element("test".to_string());
        assert_eq!(element.name(), "test");
        assert!(element.namespace().is_none());
    }

    #[test]
    fn test_namespace_support() {
        let doc = create_document();
        let namespace = Namespace::prefixed("http://example.com".to_string(), "ex".to_string());
        let element = doc.create_element_with_namespace("test".to_string(), namespace.clone());

        assert_eq!(element.name(), "test");
        assert_eq!(element.namespace(), Some(&namespace));
        assert_eq!(element.qualified_name(), "ex:test");
    }

    #[test]
    fn test_add_attributes() {
        let doc = create_document();
        let element = doc.create_element("test".to_string());

        let attr = Attribute::new("id".to_string(), "123".to_string());
        element.add_attribute(attr);

        let attributes = element.attributes();
        assert_eq!(attributes.len(), 1);
        assert_eq!(attributes[0].name, "id");
        assert_eq!(attributes[0].value, "123");
    }

    #[test]
    fn test_add_children() {
        let doc = create_document();
        let parent = doc.create_element("parent".to_string());
        let child = doc.create_element("child".to_string());

        parent.add_child(child.clone()).unwrap();

        let children = parent.children();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name(), "child");
        assert!(child.is_attached());
    }

    #[test]
    fn test_namespace_declaration() {
        let doc = create_document();
        let root = doc.create_element("root".to_string());
        root.declare_namespace("ex".to_string(), "http://example.com".to_string());

        assert_eq!(
            root.get_namespace_uri("ex"),
            Some("http://example.com".to_string())
        );
    }

    #[test]
    fn test_qualified_name_resolution() {
        let doc = create_document();
        let root = doc.create_element("root".to_string());
        root.declare_namespace("ex".to_string(), "http://example.com".to_string());

        let (local_name, namespace) = root.resolve_qualified_name("ex:test").unwrap();
        assert_eq!(local_name, "test");
        assert_eq!(namespace.unwrap().uri, "http://example.com");
    }

    #[test]
    fn test_document_reference() {
        let doc = create_document();
        let element = doc.create_element("test".to_string());

        // Set as root should work
        doc.set_root(element).unwrap();
        assert!(doc.root().is_some());
    }
}
