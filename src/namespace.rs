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