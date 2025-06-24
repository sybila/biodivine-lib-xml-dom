use parking_lot::RwLock;
use std::sync::Arc;

use crate::element::Element;
use crate::error::{XmlError, XmlResult};
use crate::namespace::Namespace;

/// Internal document structure that handles Arc complexity
#[derive(Debug)]
pub(crate) struct InternalDocument {
    /// Unique identifier for this document
    id: u64,
    /// Root element of the document
    root: RwLock<Option<Element>>,
    /// Next available prefix for auto-generated prefixes
    next_prefix_id: RwLock<u32>,
}

impl InternalDocument {
    pub(crate) fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);

        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            root: RwLock::new(None),
            next_prefix_id: RwLock::new(0),
        }
    }

    pub(crate) fn id(&self) -> u64 {
        self.id
    }

    pub(crate) fn set_root(&self, root: Element) -> XmlResult<()> {
        if !root.belongs_to_document(self) {
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

    pub(crate) fn generate_prefix(&self) -> String {
        let mut id = self.next_prefix_id.write();
        *id += 1;
        format!("ns{}", id)
    }
}

impl Clone for InternalDocument {
    fn clone(&self) -> Self {
        Self {
            id: self.id(),
            root: RwLock::new(self.root.read().clone()),
            next_prefix_id: RwLock::new(*self.next_prefix_id.read()),
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
        Self::new()
    }
}

impl Document {
    /// Create a new empty XML document
    pub fn new() -> Self {
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
    pub fn create_element(&self, name: String) -> Element {
        Element::new(self.clone(), name)
    }

    /// Create a new namespaced element in this document
    pub fn create_element_with_namespace(&self, name: String, namespace: Namespace) -> Element {
        Element::with_namespace(self.clone(), name, namespace)
    }

    /// Generate a unique prefix for a namespace
    pub fn generate_prefix(&self) -> String {
        self.internal.generate_prefix()
    }
}
