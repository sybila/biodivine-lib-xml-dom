use parking_lot::RwLock;
use std::sync::Arc;

use crate::error::{XmlError, XmlResult};
use crate::namespace::{Attribute, Namespace};
use crate::document::InternalDocument;

/// Represents an XML element node
#[derive(Debug)]
pub struct Element {
    /// The ID of the internal document this element belongs to
    document_id: u64,
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
    pub(crate) fn new(document: Arc<InternalDocument>, name: String) -> Self {
        Self {
            document_id: document.id(),
            name,
            namespace: None,
            attributes: RwLock::new(Vec::new()),
            children: RwLock::new(Vec::new()),
            text_content: RwLock::new(None),
            parent: RwLock::new(None),
        }
    }

    /// Create a new namespaced element
    pub(crate) fn with_namespace(document: Arc<InternalDocument>, name: String, namespace: Namespace) -> Self {
        Self {
            document_id: document.id(),
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

    /// Check if this element belongs to the given internal document
    pub(crate) fn belongs_to_document(&self, doc: &InternalDocument) -> bool {
        self.document_id == doc.id()
    }
}

impl Clone for Element {
    fn clone(&self) -> Self {
        Self {
            document_id: self.document_id,
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            attributes: RwLock::new(self.attributes.read().clone()),
            children: RwLock::new(self.children.read().clone()),
            text_content: RwLock::new(self.text_content.read().clone()),
            parent: RwLock::new(None), // Don't clone parent to avoid cycles
        }
    }
} 