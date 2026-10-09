use parking_lot::RwLock;
use std::sync::Arc;

use crate::element::Element;
use crate::error::{XmlError, XmlResult};
use crate::qualified_name::QualifiedName;

/// Internal document structure that handles Arc complexity
#[derive(Debug)]
pub(crate) struct InternalDocument {
    /// Root element of the document
    root: RwLock<Option<Element>>,
}

impl InternalDocument {
    pub(crate) fn new() -> Self {
        Self {
            root: RwLock::new(None),
        }
    }

    pub(crate) fn belongs_to_document(&self, doc: &Document) -> bool {
        std::ptr::eq(self, &*doc.internal)
    }

    pub(crate) fn set_root(&self, root: Element) -> XmlResult<()> {
        if !self.belongs_to_document(&root.document()) {
            return Err(XmlError::InvalidOperation(
                "Element belongs to a different document".to_string(),
            ));
        }
        *self.root.write() = Some(root);
        Ok(())
    }

    pub(crate) fn root(&self) -> Option<Element> {
        self.root.read().clone()
    }
}

impl Clone for InternalDocument {
    fn clone(&self) -> Self {
        Self {
            root: RwLock::new(self.root.read().clone()),
        }
    }
}

/// Public document structure that wraps the internal document
#[derive(Debug, Clone)]
pub struct Document {
    pub(crate) internal: Arc<InternalDocument>,
}

impl Default for Document {
    fn default() -> Self {
        Self::empty()
    }
}

impl Document {
    /// Create a new empty XML document
    pub fn empty() -> Self {
        Self {
            internal: Arc::new(InternalDocument::new()),
        }
    }

    /// Set the root element
    pub fn set_root(&self, root: Element) -> XmlResult<()> {
        self.internal.set_root(root)
    }

    /// Get the root element
    pub fn root(&self) -> Option<Element> {
        self.internal.root()
    }

    /// Create a new element in this document
    pub fn create_element(&self, qualified_name: QualifiedName) -> Element {
        Element::new(self.clone(), qualified_name)
    }
}

impl PartialEq for Document {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.internal, &other.internal)
    }
}

impl Eq for Document {}
