use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use crate::error::{XmlError, XmlResult};
use crate::namespace::Namespace;
use crate::element::Element;

/// Internal document structure that handles Arc complexity
#[derive(Debug)]
pub(crate) struct InternalDocument {
    /// Unique identifier for this document
    id: u64,
    /// Root element of the document
    root: RwLock<Option<Arc<Element>>>,
    /// Default namespace declarations
    default_namespaces: RwLock<HashMap<String, String>>,
    /// Prefixed namespace declarations
    prefixed_namespaces: RwLock<HashMap<String, String>>,
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
            default_namespaces: RwLock::new(HashMap::new()),
            prefixed_namespaces: RwLock::new(HashMap::new()),
            next_prefix_id: RwLock::new(0),
        }
    }

    pub(crate) fn id(&self) -> u64 {
        self.id
    }

    pub(crate) fn set_root(&self, root: Arc<Element>) -> XmlResult<()> {
        // Ensure the element belongs to this document
        if !root.belongs_to_document(self) {
            return Err(XmlError::InvalidOperation(
                "Element belongs to a different document".to_string(),
            ));
        }

        *self.root.write() = Some(root);
        Ok(())
    }

    pub(crate) fn root(&self) -> Option<Arc<Element>> {
        self.root.read().clone()
    }

    pub(crate) fn declare_default_namespace(&self, uri: String) {
        self.default_namespaces.write().insert("".to_string(), uri);
    }

    pub(crate) fn declare_namespace(&self, prefix: String, uri: String) {
        self.prefixed_namespaces.write().insert(prefix, uri);
    }

    pub(crate) fn get_namespace_uri(&self, prefix: &str) -> Option<String> {
        if prefix.is_empty() {
            // Return default namespace
            self.default_namespaces.read().get("").cloned()
        } else {
            self.prefixed_namespaces.read().get(prefix).cloned()
        }
    }

    pub(crate) fn generate_prefix(&self) -> String {
        let mut id = self.next_prefix_id.write();
        *id += 1;
        format!("ns{}", id)
    }

    pub(crate) fn resolve_qualified_name(&self, qualified_name: &str) -> XmlResult<(String, Option<Namespace>)> {
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

impl Clone for InternalDocument {
    fn clone(&self) -> Self {
        Self {
            id: self.id(),
            root: RwLock::new(self.root.read().clone()),
            default_namespaces: RwLock::new(self.default_namespaces.read().clone()),
            prefixed_namespaces: RwLock::new(self.prefixed_namespaces.read().clone()),
            next_prefix_id: RwLock::new(*self.next_prefix_id.read()),
        }
    }
}

/// Public document structure that wraps the internal document
#[derive(Debug, Clone)]
pub struct Document {
    internal: Arc<InternalDocument>,
}

impl Document {
    /// Create a new empty XML document
    pub fn new() -> Self {
        Self {
            internal: Arc::new(InternalDocument::new()),
        }
    }

    /// Set the root element
    pub fn set_root(&self, root: Arc<Element>) -> XmlResult<()> {
        self.internal.set_root(root)
    }

    /// Get the root element
    pub fn root(&self) -> Option<Arc<Element>> {
        self.internal.root()
    }

    /// Create a new element in this document
    pub fn create_element(&self, name: String) -> Arc<Element> {
        Arc::new(Element::new(self.internal.clone(), name))
    }

    /// Create a new namespaced element in this document
    pub fn create_element_with_namespace(&self, name: String, namespace: Namespace) -> Arc<Element> {
        Arc::new(Element::with_namespace(self.internal.clone(), name, namespace))
    }

    /// Declare a default namespace
    pub fn declare_default_namespace(&self, uri: String) {
        self.internal.declare_default_namespace(uri);
    }

    /// Declare a prefixed namespace
    pub fn declare_namespace(&self, prefix: String, uri: String) {
        self.internal.declare_namespace(prefix, uri);
    }

    /// Get namespace URI by prefix
    pub fn get_namespace_uri(&self, prefix: &str) -> Option<String> {
        self.internal.get_namespace_uri(prefix)
    }

    /// Generate a unique prefix for a namespace
    pub fn generate_prefix(&self) -> String {
        self.internal.generate_prefix()
    }

    /// Resolve a qualified name to local name and namespace
    pub fn resolve_qualified_name(&self, qualified_name: &str) -> XmlResult<(String, Option<Namespace>)> {
        self.internal.resolve_qualified_name(qualified_name)
    }
} 