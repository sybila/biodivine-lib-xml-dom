use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

/// Error types for XML DOM operations
#[derive(Error, Debug)]
pub enum XmlError {
    #[error("Invalid XML: {0}")]
    InvalidXml(String),
    #[error("Namespace error: {0}")]
    NamespaceError(String),
    #[error("Element not found")]
    ElementNotFound,
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

/// Result type for XML DOM operations
pub type XmlResult<T> = Result<T, XmlError>;

/// Represents an XML namespace with URI and optional prefix
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Namespace {
    pub uri: String,
    pub prefix: Option<String>,
}

impl Namespace {
    /// Create a new namespace with URI and optional prefix
    pub fn new(uri: String, prefix: Option<String>) -> Self {
        Self { uri, prefix }
    }

    /// Create a default namespace (no prefix)
    pub fn default(uri: String) -> Self {
        Self { uri, prefix: None }
    }

    /// Create a prefixed namespace
    pub fn prefixed(uri: String, prefix: String) -> Self {
        Self {
            uri,
            prefix: Some(prefix),
        }
    }
}

/// Represents an XML attribute with optional namespace
#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub value: String,
    pub namespace: Option<Namespace>,
}

impl Attribute {
    /// Create a new attribute without namespace
    pub fn new(name: String, value: String) -> Self {
        Self {
            name,
            value,
            namespace: None,
        }
    }

    /// Create a new namespaced attribute
    pub fn with_namespace(name: String, value: String, namespace: Namespace) -> Self {
        Self {
            name,
            value,
            namespace: Some(namespace),
        }
    }
}

/// Represents an XML element node
#[derive(Debug)]
pub struct Element {
    /// The document this element belongs to
    document: Arc<Document>,
    /// Element name (local name)
    name: String,
    /// Element namespace
    namespace: Option<Namespace>,
    /// Element attributes
    attributes: RwLock<Vec<Attribute>>,
    /// Child elements
    children: RwLock<Vec<Arc<Element>>>,
    /// Text content (if this element contains only text)
    text_content: RwLock<Option<String>>,
    /// Parent element (None if root or detached)
    parent: RwLock<Option<Arc<Element>>>,
}

impl Element {
    /// Create a new element in the given document
    pub fn new(document: Arc<Document>, name: String) -> Self {
        Self {
            document,
            name,
            namespace: None,
            attributes: RwLock::new(Vec::new()),
            children: RwLock::new(Vec::new()),
            text_content: RwLock::new(None),
            parent: RwLock::new(None),
        }
    }

    /// Create a new namespaced element
    pub fn with_namespace(document: Arc<Document>, name: String, namespace: Namespace) -> Self {
        Self {
            document,
            name,
            namespace: Some(namespace),
            attributes: RwLock::new(Vec::new()),
            children: RwLock::new(Vec::new()),
            text_content: RwLock::new(None),
            parent: RwLock::new(None),
        }
    }

    /// Get the element name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the element namespace
    pub fn namespace(&self) -> Option<&Namespace> {
        self.namespace.as_ref()
    }

    /// Get the qualified name (prefix:local_name or just local_name)
    pub fn qualified_name(&self) -> String {
        if let Some(ref ns) = self.namespace {
            if let Some(ref prefix) = ns.prefix {
                format!("{}:{}", prefix, self.name)
            } else {
                self.name.clone()
            }
        } else {
            self.name.clone()
        }
    }

    /// Add an attribute to this element
    pub fn add_attribute(&self, attribute: Attribute) {
        self.attributes.write().push(attribute);
    }

    /// Get all attributes
    pub fn attributes(&self) -> Vec<Attribute> {
        self.attributes.read().clone()
    }

    /// Get attribute by name (local name only)
    pub fn get_attribute(&self, name: &str) -> Option<Attribute> {
        self.attributes
            .read()
            .iter()
            .find(|attr| attr.name == name)
            .cloned()
    }

    /// Get attribute by qualified name (prefix:name)
    pub fn get_attribute_by_qualified_name(&self, qualified_name: &str) -> Option<Attribute> {
        self.attributes
            .read()
            .iter()
            .find(|attr| {
                if let Some(ref ns) = attr.namespace {
                    if let Some(ref prefix) = ns.prefix {
                        format!("{}:{}", prefix, attr.name) == qualified_name
                    } else {
                        attr.name == qualified_name
                    }
                } else {
                    attr.name == qualified_name
                }
            })
            .cloned()
    }

    /// Add a child element
    pub fn add_child(&self, child: Arc<Element>) -> XmlResult<()> {
        // Set this element as the parent of the child
        *child.parent.write() = Some(Arc::new(self.clone()));
        
        // Add to children list
        self.children.write().push(child);
        Ok(())
    }

    /// Remove a child element
    pub fn remove_child(&self, child: &Arc<Element>) -> XmlResult<()> {
        let mut children = self.children.write();
        if let Some(pos) = children.iter().position(|c| Arc::ptr_eq(c, child)) {
            children.remove(pos);
            // Clear the parent reference
            *child.parent.write() = None;
            Ok(())
        } else {
            Err(XmlError::ElementNotFound)
        }
    }

    /// Get all child elements
    pub fn children(&self) -> Vec<Arc<Element>> {
        self.children.read().clone()
    }

    /// Get child elements by name
    pub fn get_children_by_name(&self, name: &str) -> Vec<Arc<Element>> {
        self.children
            .read()
            .iter()
            .filter(|child| child.name() == name)
            .cloned()
            .collect()
    }

    /// Set text content (clears children if any)
    pub fn set_text_content(&self, text: String) {
        let mut text_content = self.text_content.write();
        let mut children = self.children.write();
        
        *text_content = Some(text);
        children.clear();
    }

    /// Get text content
    pub fn text_content(&self) -> Option<String> {
        self.text_content.read().clone()
    }

    /// Get parent element
    pub fn parent(&self) -> Option<Arc<Element>> {
        self.parent.read().clone()
    }

    /// Check if element is attached to a document tree
    pub fn is_attached(&self) -> bool {
        self.parent.read().is_some()
    }

    /// Get the document this element belongs to
    pub fn document(&self) -> Arc<Document> {
        self.document.clone()
    }

    /// Check if this element belongs to the given document
    pub fn belongs_to_document(&self, doc: &Document) -> bool {
        // Since we're comparing with a Document reference, we need to create a temporary Arc
        // and compare the underlying pointers
        let temp_doc = Arc::new(doc.clone());
        Arc::ptr_eq(&self.document, &temp_doc)
    }

    /// Check if this element belongs to the given document
    pub fn belongs_to_document_arc(&self, doc: &Arc<Document>) -> bool {
        Arc::ptr_eq(&self.document, doc)
    }
}

impl Clone for Element {
    fn clone(&self) -> Self {
        Self {
            document: self.document.clone(),
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            attributes: RwLock::new(self.attributes.read().clone()),
            children: RwLock::new(self.children.read().clone()),
            text_content: RwLock::new(self.text_content.read().clone()),
            parent: RwLock::new(None), // Don't clone parent to avoid cycles
        }
    }
}

/// Represents an XML document with root element and namespace management
#[derive(Debug)]
pub struct Document {
    /// Root element of the document
    root: RwLock<Option<Arc<Element>>>,
    /// Default namespace declarations
    default_namespaces: RwLock<HashMap<String, String>>,
    /// Prefixed namespace declarations
    prefixed_namespaces: RwLock<HashMap<String, String>>,
    /// Next available prefix for auto-generated prefixes
    next_prefix_id: RwLock<u32>,
}

impl Document {
    /// Create a new empty XML document
    pub fn new() -> Self {
        Self {
            root: RwLock::new(None),
            default_namespaces: RwLock::new(HashMap::new()),
            prefixed_namespaces: RwLock::new(HashMap::new()),
            next_prefix_id: RwLock::new(0),
        }
    }

    /// Set the root element
    pub fn set_root(&self, root: Arc<Element>) -> XmlResult<()> {
        // Ensure the element belongs to this document
        if !root.belongs_to_document_arc(&Arc::new(self.clone())) {
            return Err(XmlError::InvalidOperation(
                "Element belongs to a different document".to_string(),
            ));
        }

        *self.root.write() = Some(root);
        Ok(())
    }

    /// Get the root element
    pub fn root(&self) -> Option<Arc<Element>> {
        self.root.read().clone()
    }

    /// Create a new element in this document
    pub fn create_element(&self, name: String) -> Arc<Element> {
        Arc::new(Element::new(Arc::new(self.clone()), name))
    }

    /// Create a new namespaced element in this document
    pub fn create_element_with_namespace(&self, name: String, namespace: Namespace) -> Arc<Element> {
        Arc::new(Element::with_namespace(Arc::new(self.clone()), name, namespace))
    }

    /// Declare a default namespace
    pub fn declare_default_namespace(&self, uri: String) {
        self.default_namespaces.write().insert("".to_string(), uri);
    }

    /// Declare a prefixed namespace
    pub fn declare_namespace(&self, prefix: String, uri: String) {
        self.prefixed_namespaces.write().insert(prefix, uri);
    }

    /// Get namespace URI by prefix
    pub fn get_namespace_uri(&self, prefix: &str) -> Option<String> {
        if prefix.is_empty() {
            // Return default namespace
            self.default_namespaces.read().get("").cloned()
        } else {
            self.prefixed_namespaces.read().get(prefix).cloned()
        }
    }

    /// Generate a unique prefix for a namespace
    pub fn generate_prefix(&self) -> String {
        let mut id = self.next_prefix_id.write();
        *id += 1;
        format!("ns{}", id)
    }

    /// Resolve a qualified name to local name and namespace
    pub fn resolve_qualified_name(&self, qualified_name: &str) -> XmlResult<(String, Option<Namespace>)> {
        if let Some(colon_pos) = qualified_name.find(':') {
            let prefix = &qualified_name[..colon_pos];
            let local_name = &qualified_name[colon_pos + 1..];
            
            if let Some(uri) = self.get_namespace_uri(prefix) {
                let namespace = Namespace::prefixed(uri, prefix.to_string());
                Ok((local_name.to_string(), Some(namespace)))
            } else {
                Err(XmlError::NamespaceError(
                    format!("Undefined namespace prefix: {}", prefix)
                ))
            }
        } else {
            // No prefix, use default namespace if available
            if let Some(uri) = self.get_namespace_uri("") {
                let namespace = Namespace::default(uri);
                Ok((qualified_name.to_string(), Some(namespace)))
            } else {
                Ok((qualified_name.to_string(), None))
            }
        }
    }
}

impl Clone for Document {
    fn clone(&self) -> Self {
        Self {
            root: RwLock::new(self.root.read().clone()),
            default_namespaces: RwLock::new(self.default_namespaces.read().clone()),
            prefixed_namespaces: RwLock::new(self.prefixed_namespaces.read().clone()),
            next_prefix_id: RwLock::new(*self.next_prefix_id.read()),
        }
    }
}

/// Main entry point for the library
pub fn create_document() -> Arc<Document> {
    Arc::new(Document::new())
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
        doc.declare_namespace("ex".to_string(), "http://example.com".to_string());
        
        assert_eq!(
            doc.get_namespace_uri("ex"),
            Some("http://example.com".to_string())
        );
    }

    #[test]
    fn test_qualified_name_resolution() {
        let doc = create_document();
        doc.declare_namespace("ex".to_string(), "http://example.com".to_string());
        
        let (local_name, namespace) = doc.resolve_qualified_name("ex:test").unwrap();
        assert_eq!(local_name, "test");
        assert_eq!(namespace.unwrap().uri, "http://example.com");
    }

    #[test]
    fn test_document_reference() {
        let doc = create_document();
        let element = doc.create_element("test".to_string());
        
        // This should work
        assert!(element.belongs_to_document_arc(&doc));
        
        // Set as root should work
        doc.set_root(element).unwrap();
        assert!(doc.root().is_some());
    }
} 