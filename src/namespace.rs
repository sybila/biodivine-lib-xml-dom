use crate::error::XmlError;

/// Represents an XML namespace with URI and optional prefix.
///
/// # Conditions for a valid namespace:
/// - The URI must not be empty.
/// - The prefix, if present, must not be empty and must not contain a colon (`:`).
/// - *Other validation rules may be added in the future (e.g., that URI is actually a valid URI).*
///
/// Use the provided constructors and setter methods to ensure validity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Namespace {
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
        Ok(Self { uri, prefix })
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
            uri: uri_str.to_string(),
            prefix: None,
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
            uri: uri_str.to_string(),
            prefix: Some(prefix_str.to_string()),
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
        &self.uri
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
        self.prefix.as_deref()
    }

    /// Set the namespace URI, validating XML rules.
    ///
    /// # Errors
    /// Returns [`XmlError`] if the new URI is empty.
    ///
    /// # Examples
    /// ```rust
    /// use biodivine_lib_xml_dom::Namespace;
    /// let mut ns = Namespace::default("http://example.com").unwrap();
    /// assert!(ns.set_uri("http://new.com").is_ok());
    /// assert_eq!(ns.uri(), "http://new.com");
    /// assert!(ns.set_uri("").is_err());
    /// ```
    pub fn set_uri<U: AsRef<str>>(&mut self, uri: U) -> Result<(), XmlError> {
        let uri_str = uri.as_ref();
        Self::validate(uri_str, self.prefix.as_deref())?;
        self.uri = uri_str.to_string();
        Ok(())
    }

    /// Set the namespace prefix, validating XML rules.
    ///
    /// # Errors
    /// Returns [`XmlError`] if the new prefix is empty or contains a colon.
    ///
    /// # Examples
    /// ```rust
    /// use biodivine_lib_xml_dom::Namespace;
    /// let mut ns = Namespace::default("http://example.com").unwrap();
    /// assert!(ns.set_prefix("ex").is_ok());
    /// assert_eq!(ns.prefix(), Some("ex"));
    /// assert!(ns.set_prefix("").is_err());
    /// assert!(ns.set_prefix("ex:bad").is_err());
    /// ns.unset_prefix();
    /// assert_eq!(ns.prefix(), None);
    /// ```
    pub fn set_prefix<P: AsRef<str>>(&mut self, prefix: P) -> Result<(), XmlError> {
        let prefix_str = prefix.as_ref();
        Self::validate(&self.uri, Some(prefix_str))?;
        self.prefix = Some(prefix_str.to_string());
        Ok(())
    }

    /// Remove the namespace prefix (set to `None`).
    ///
    /// # Examples
    /// ```rust
    /// use biodivine_lib_xml_dom::Namespace;
    /// let mut ns = Namespace::prefixed("http://example.com", "ex").unwrap();
    /// ns.unset_prefix();
    /// assert_eq!(ns.prefix(), None);
    /// ```
    pub fn unset_prefix(&mut self) {
        self.prefix = None;
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
}
