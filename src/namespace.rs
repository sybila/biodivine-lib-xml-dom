use crate::error::XmlError;
use std::sync::Arc;

/// Represents an XML namespace with URI and optional prefix.
///
/// [`Namespace`] is immutable and thread-safe by design. The namespace data is shared
/// behind an `Arc` pointer, so copying namespaces should be relatively cheap. Just make
/// sure to prefer cloning existing namespaces instead of creating new ones to reduce memory
/// usage as much as possible.
///
/// # Conditions for a valid namespace:
/// - The URI must not be empty.
/// - The prefix, if present, must not be empty and must not contain a colon (`:`).
/// - *Other validation rules may be added in the future (e.g., that URI is actually a valid URI).*
///
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Namespace {
    data: Arc<NamespaceData>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct NamespaceData {
    uri: String,
    prefix: Option<String>,
}

impl Namespace {
    /// Create a new namespace with URI and optional prefix, validating XML rules.
    ///
    /// # Errors
    /// Returns `XmlError` if the URI is empty, or if the prefix is empty or contains a colon.
    ///
    /// # Examples
    /// ```rust
    /// use biodivine_lib_xml_dom::Namespace;
    /// let ns = Namespace::new("http://example.com".to_string(), Some("ex".to_string()));
    /// assert!(ns.is_ok());
    /// let ns = Namespace::new("".to_string(), Some("ex".to_string()));
    /// assert!(ns.is_err());
    /// ```
    pub fn new(uri: String, prefix: Option<String>) -> Result<Self, XmlError> {
        Self::validate(&uri, prefix.as_deref())?;
        Ok(Self {
            data: Arc::new(NamespaceData { uri, prefix }),
        })
    }

    /// Create a default namespace (no prefix), validating XML rules.
    ///
    /// # Errors
    /// Returns `XmlError` if the URI is empty.
    ///
    /// # Examples
    /// ```rust
    /// use biodivine_lib_xml_dom::Namespace;
    /// let ns = Namespace::default("http://example.com");
    /// assert!(ns.is_ok());
    /// let ns = Namespace::default("");
    /// assert!(ns.is_err());
    /// ```
    pub fn default<U: AsRef<str>>(uri: U) -> Result<Self, XmlError> {
        let uri_str = uri.as_ref();
        Self::validate(uri_str, None)?;
        Ok(Self {
            data: Arc::new(NamespaceData {
                uri: uri_str.to_string(),
                prefix: None,
            }),
        })
    }

    /// Create a prefixed namespace, validating XML rules.
    ///
    /// # Errors
    /// Returns [`XmlError`] if the URI is empty, or if the prefix is empty or contains a colon.
    ///
    /// # Examples
    /// ```rust
    /// use biodivine_lib_xml_dom::Namespace;
    /// let ns = Namespace::prefixed("http://example.com", "ex");
    /// assert!(ns.is_ok());
    /// let ns = Namespace::prefixed("http://example.com", "ex:bad");
    /// assert!(ns.is_err());
    /// ```
    pub fn prefixed<U: AsRef<str>, P: AsRef<str>>(uri: U, prefix: P) -> Result<Self, XmlError> {
        let uri_str = uri.as_ref();
        let prefix_str = prefix.as_ref();
        Self::validate(uri_str, Some(prefix_str))?;
        Ok(Self {
            data: Arc::new(NamespaceData {
                uri: uri_str.to_string(),
                prefix: Some(prefix_str.to_string()),
            }),
        })
    }

    /// Get a reference to the namespace URI.
    ///
    /// # Examples
    /// ```rust
    /// use biodivine_lib_xml_dom::Namespace;
    /// let ns = Namespace::default("http://example.com").unwrap();
    /// assert_eq!(ns.uri(), "http://example.com");
    /// ```
    pub fn uri(&self) -> &str {
        &self.data.uri
    }

    /// Get a reference to the namespace prefix, if any.
    ///
    /// # Examples
    /// ```rust
    /// use biodivine_lib_xml_dom::Namespace;
    /// let ns = Namespace::prefixed("http://example.com", "ex").unwrap();
    /// assert_eq!(ns.prefix(), Some("ex"));
    /// let ns = Namespace::default("http://example.com").unwrap();
    /// assert_eq!(ns.prefix(), None);
    /// ```
    pub fn prefix(&self) -> Option<&str> {
        self.data.prefix.as_deref()
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
        // URI must not be empty.
        // Note: Based on XML spec 1.0, empty URI is technically valid,
        // but it has no meaning. In 1.1, a meaning was given to it (it
        // "cancels" the previous namespace declaration), but the 1.1 spec
        // is not widely adopted.
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
        let namespace = Namespace::prefixed("http://example.com", "ex").unwrap();
        let element = doc.create_element_with_namespace("test".to_string(), namespace.clone());

        assert_eq!(element.name(), "test");
        assert_eq!(element.namespace(), Some(namespace));
        assert_eq!(element.qualified_name(), "ex:test");
    }

    #[test]
    fn test_namespace_validation() {
        // Valid namespace
        assert!(Namespace::prefixed("http://example.com", "ex").is_ok());
        // Prefix with colon
        assert!(Namespace::prefixed("http://example.com", "ex:bad").is_err());
        // Empty prefix
        assert!(Namespace::prefixed("http://example.com", "").is_err());
        // Empty URI
        assert!(Namespace::prefixed("", "ex").is_err());
        // Default namespace with empty URI
        assert!(Namespace::default("").is_err());
        // Default namespace with valid URI
        assert!(Namespace::default("http://example.com").is_ok());
        // New with None prefix
        assert!(Namespace::new("http://example.com".to_string(), None).is_ok());
        // New with Some valid prefix
        assert!(Namespace::new("http://example.com".to_string(), Some("ex".to_string())).is_ok());
        // New with Some invalid prefix
        assert!(
            Namespace::new("http://example.com".to_string(), Some("ex:bad".to_string())).is_err()
        );
    }

    #[test]
    fn test_namespace_equality() {
        let ns1 = Namespace::prefixed("http://example.com", "ex").unwrap();
        let ns2 = Namespace::prefixed("http://example.com", "ex").unwrap();
        let ns3 = Namespace::prefixed("http://example.com", "other").unwrap();
        let ns4 = Namespace::default("http://example.com").unwrap();
        let ns5 = Namespace::default("http://different.com").unwrap();

        // Same URI and prefix, but different Arc pointers
        assert_eq!(ns1, ns2);
        // Same URI, different prefix
        assert_ne!(ns1, ns3);
        // Same URI, one with prefix, one without
        assert_ne!(ns1, ns4);
        // Different URI
        assert_ne!(ns1, ns5);
        // Default namespace equality
        let ns6 = Namespace::default("http://example.com").unwrap();
        assert_eq!(ns4, ns6);

        // Clone should produce an equal Namespace (same Arc pointer)
        let ns1_clone = ns1.clone();
        assert_eq!(ns1, ns1_clone);
        // They should be equal, and their internal Arc pointers should be the same
        let arc1: *const _ = &*ns1.data;
        let arc1_clone: *const _ = &*ns1_clone.data;
        assert_eq!(
            arc1, arc1_clone,
            "Cloned Namespace should share the same Arc pointer"
        );

        // But two independently created identical Namespaces should not share the same Arc pointer
        let arc2: *const _ = &*ns2.data;
        assert_ne!(
            arc1, arc2,
            "New Namespace with same data should not share Arc pointer"
        );
    }
}
