use crate::error::XmlError;

/// Represents an XML namespace with URI and optional prefix
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Namespace {
    pub uri: String,
    pub prefix: Option<String>,
}

impl Namespace {
    /// Create a new namespace with URI and optional prefix, validating XML rules.
    pub fn new(uri: String, prefix: Option<String>) -> Result<Self, XmlError> {
        Self::validate(&uri, prefix.as_deref())?;
        Ok(Self { uri, prefix })
    }

    /// Create a default namespace (no prefix), validating XML rules.
    pub fn default(uri: String) -> Result<Self, XmlError> {
        Self::validate(&uri, None)?;
        Ok(Self { uri, prefix: None })
    }

    /// Create a prefixed namespace, validating XML rules.
    pub fn prefixed(uri: String, prefix: String) -> Result<Self, XmlError> {
        Self::validate(&uri, Some(&prefix))?;
        Ok(Self {
            uri,
            prefix: Some(prefix),
        })
    }

    /// Validate the URI and prefix according to XML namespace rules.
    fn validate(uri: &str, prefix: Option<&str>) -> Result<(), XmlError> {
        // Prefix must not contain a colon
        if let Some(p) = prefix {
            if p.contains(':') {
                return Err(XmlError::NamespaceError(
                    "Namespace prefix must not contain ':'".to_string(),
                ));
            }
            if p.is_empty() {
                return Err(XmlError::NamespaceError(
                    "Namespace prefix must not be empty".to_string(),
                ));
            }
        }
        // URI must not be empty
        if uri.is_empty() {
            return Err(XmlError::NamespaceError(
                "Namespace URI must not be empty".to_string(),
            ));
        }
        // Additional XML namespace rules can be added here
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Document;

    #[test]
    fn test_namespace_support() {
        let doc = Document::new();
        let namespace =
            Namespace::prefixed("http://example.com".to_string(), "ex".to_string()).unwrap();
        let element = doc.create_element_with_namespace("test".to_string(), namespace.clone());

        assert_eq!(element.name(), "test");
        assert_eq!(element.namespace(), Some(namespace));
        assert_eq!(element.qualified_name(), "ex:test");
    }

    #[test]
    fn test_namespace_validation() {
        // Valid namespace
        assert!(Namespace::prefixed("http://example.com".to_string(), "ex".to_string()).is_ok());
        // Prefix with colon
        assert!(
            Namespace::prefixed("http://example.com".to_string(), "ex:bad".to_string()).is_err()
        );
        // Empty prefix
        assert!(Namespace::prefixed("http://example.com".to_string(), "".to_string()).is_err());
        // Empty URI
        assert!(Namespace::prefixed("".to_string(), "ex".to_string()).is_err());
        // Default namespace with empty URI
        assert!(Namespace::default("".to_string()).is_err());
        // Default namespace with valid URI
        assert!(Namespace::default("http://example.com".to_string()).is_ok());
        // New with None prefix
        assert!(Namespace::new("http://example.com".to_string(), None).is_ok());
        // New with Some valid prefix
        assert!(Namespace::new("http://example.com".to_string(), Some("ex".to_string())).is_ok());
        // New with Some invalid prefix
        assert!(
            Namespace::new("http://example.com".to_string(), Some("ex:bad".to_string())).is_err()
        );
    }
}
