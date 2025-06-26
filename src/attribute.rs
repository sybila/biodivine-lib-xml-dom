use crate::namespace::Namespace;

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