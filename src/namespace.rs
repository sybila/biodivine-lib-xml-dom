use crate::error::XmlError;
use std::sync::Arc;

/// The `xml` prefix is by definition bound to this namespace.
/// It may, but need not, be declared, and must not be undeclared or bound to any other namespace.
/// Other prefixes must not be bound to this namespace, and it must not be declared as the default namespace.
///
/// See: <https://www.w3.org/TR/xml-names11/#xmlReserved>
const RESERVED_XML_URI: &str = "http://www.w3.org/XML/1998/namespace";

/// The `xmlns` prefix is used only to declare namespace bindings and is by definition bound
/// to this namespace. It must not be declared or undeclared. Other prefixes must not be bound
/// to this namespace, and it must not be declared as the default namespace.
///
/// See: <https://www.w3.org/TR/xml-names11/#xmlReserved>
const RESERVED_XMLNS_URI: &str = "http://www.w3.org/2000/xmlns/";

/// Represents an XML namespace with URI and optional prefix.
///
/// [`Namespace`] is immutable and thread-safe by design. The namespace data is shared
/// behind an `Arc` pointer, so copying namespaces should be relatively cheap. Just make
/// sure to prefer cloning existing namespaces instead of creating new ones to reduce memory
/// usage as much as possible.
///
/// # Equality
/// Two [`Namespace`] objects are considered equal if they have the same URI **and** the same prefix.
/// If you want to compare only the namespace URIs, use [`Namespace::is_equal_ns`]. This is what
/// the XML specification considers as "equal" namespaces in the context of namespace declarations.
///
/// # Conditions for a valid namespace:
/// - The URI must not be empty.
/// - The prefix, if present, must be a valid XML NCName per XML 1.0 Fifth Edition:
///   non-empty, no colon (`:`), must start with a letter (including Unicode), underscore,
///   and may contain letters, digits, underscores, hyphens, and periods (including Unicode).
/// - The prefix `xml` must only be bound to `http://www.w3.org/XML/1998/namespace`.
/// - The prefix `xmlns` must never be declared.
/// - No prefix other than `xml` may be bound to `http://www.w3.org/XML/1998/namespace`.
/// - No prefix may be bound to `http://www.w3.org/2000/xmlns/`.
/// - The default namespace (no prefix) must not be `http://www.w3.org/XML/1998/namespace`
///   or `http://www.w3.org/2000/xmlns/`.
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
    /// Returns `XmlError` if:
    /// - The URI is empty.
    /// - The prefix is not a valid XML NCName (per XML 1.0 Fifth Edition Unicode ranges).
    /// - The prefix is `xml` but the URI is not `http://www.w3.org/XML/1998/namespace`.
    /// - The prefix is `xmlns` (this prefix is reserved and cannot be declared).
    /// - The prefix is not `xml` but the URI is `http://www.w3.org/XML/1998/namespace`.
    /// - The URI is `http://www.w3.org/2000/xmlns/` (this URI cannot be bound to any prefix).
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

    /// Create a namespace without a prefix (default namespace), validating XML rules.
    ///
    /// # Errors
    /// Returns `XmlError` if:
    /// - The URI is empty.
    /// - The URI is `http://www.w3.org/XML/1998/namespace` (reserved for the `xml` prefix only).
    /// - The URI is `http://www.w3.org/2000/xmlns/` (must not be declared as default namespace).
    ///
    /// # Examples
    /// ```rust
    /// use biodivine_lib_xml_dom::Namespace;
    /// let ns = Namespace::without_prefix("http://example.com");
    /// assert!(ns.is_ok());
    /// let ns = Namespace::without_prefix("");
    /// assert!(ns.is_err());
    /// ```
    pub fn without_prefix<U: AsRef<str>>(uri: U) -> Result<Self, XmlError> {
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
    /// Returns [`XmlError`] if:
    /// - The URI is empty.
    /// - The prefix is not a valid XML NCName (per XML 1.0 Fifth Edition Unicode ranges).
    /// - The prefix is `xml` but the URI is not `http://www.w3.org/XML/1998/namespace`.
    /// - The prefix is `xmlns` (this prefix is reserved and cannot be declared).
    /// - The prefix is not `xml` but the URI is `http://www.w3.org/XML/1998/namespace`.
    /// - The URI is `http://www.w3.org/2000/xmlns/` (this URI cannot be bound to any prefix).
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
    /// let ns = Namespace::without_prefix("http://example.com").unwrap();
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
    /// let ns = Namespace::without_prefix("http://example.com").unwrap();
    /// assert_eq!(ns.prefix(), None);
    /// ```
    pub fn prefix(&self) -> Option<&str> {
        self.data.prefix.as_deref()
    }

    /// Validate the URI and prefix according to XML namespace rules.
    fn validate(uri: &str, prefix: Option<&str>) -> Result<(), XmlError> {
        // URI must not be empty
        if uri.is_empty() {
            return Err(XmlError::NamespaceError(
                "Namespace URI must not be empty".to_string(),
            ));
        }

        // Prefix must be a valid NCName
        if let Some(p) = prefix {
            if !is_valid_ncname(p) {
                return Err(XmlError::NamespaceError(format!(
                    "Namespace prefix '{}' is not a valid XML NCName",
                    p
                )));
            }

            // `xmlns` prefix must never be declared
            if p == "xmlns" {
                return Err(XmlError::NamespaceError(
                    "The prefix 'xmlns' is reserved and cannot be declared".to_string(),
                ));
            }

            // `xml` prefix must only bind to its reserved URI
            if p == "xml" {
                if uri != RESERVED_XML_URI {
                    return Err(XmlError::NamespaceError(format!(
                        "The prefix 'xml' can only be bound to '{}', not '{}'",
                        RESERVED_XML_URI, uri
                    )));
                }
            } else {
                // No prefix other than `xml` may bind to the reserved XML URI
                if uri == RESERVED_XML_URI {
                    return Err(XmlError::NamespaceError(format!(
                        "The URI '{}' can only be bound to the 'xml' prefix, not '{}'",
                        RESERVED_XML_URI, p
                    )));
                }
            }
        }

        // Neither reserved URI may be used as a default namespace
        if prefix.is_none() && (uri == RESERVED_XML_URI || uri == RESERVED_XMLNS_URI) {
            return Err(XmlError::NamespaceError(format!(
                "The URI '{}' cannot be declared as the default namespace",
                uri
            )));
        }

        // No prefix may bind to the reserved xmlns URI
        if uri == RESERVED_XMLNS_URI {
            return Err(XmlError::NamespaceError(format!(
                "The URI '{}' cannot be bound to any prefix",
                RESERVED_XMLNS_URI
            )));
        }

        Ok(())
    }

    /// Compare two namespaces for equality based only on their URI. This is what
    /// the XML specification considers as "equal" namespaces in the context
    /// of namespace declarations.
    ///
    /// # Examples
    /// ```rust
    /// use biodivine_lib_xml_dom::Namespace;
    /// let ns1 = Namespace::prefixed("http://example.com", "ex").unwrap();
    /// let ns2 = Namespace::without_prefix("http://example.com").unwrap();
    /// assert!(Namespace::is_equal_ns(&ns1, &ns2));
    /// let ns3 = Namespace::without_prefix("http://different.com").unwrap();
    /// assert!(!Namespace::is_equal_ns(&ns1, &ns3));
    /// ```
    pub fn is_equal_ns(a: &Namespace, b: &Namespace) -> bool {
        a.uri() == b.uri()
    }
}

/// Check if a string is a valid XML NCName (for namespace prefixes).
///
/// NCName = Name - (Char* ':' Char*), where Name uses NameStartChar and NameChar
/// from XML 1.0 Fifth Edition. This implementation uses the full Unicode ranges.
fn is_valid_ncname(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    if s.contains(':') {
        return false;
    }
    let mut chars = s.chars();
    if let Some(first) = chars.next()
        && !is_ncname_start_char(first)
    {
        return false;
    }
    chars.all(is_ncname_char)
}

/// Check if a char is valid as the first character of an NCName.
/// Based on XML 1.0 NameStartChar minus ':'.
fn is_ncname_start_char(c: char) -> bool {
    let cp = c as u32;
    c == '_'
        || matches!(cp, 65..=90 | 97..=122)
        || matches!(cp, 0xC0..=0xD6 | 0xD8..=0xF6 | 0xF8..=0x2FF)
        || matches!(cp, 0x370..=0x37D | 0x37F..=0x1FFF)
        || matches!(cp, 0x200C..=0x200D | 0x2070..=0x218F)
        || matches!(cp, 0x2C00..=0x2FEF | 0x3001..=0xD7FF)
        || matches!(cp, 0xF900..=0xFDCF | 0xFDF0..=0xFFFD)
        || matches!(cp, 0x10000..=0xEFFFF)
}

/// Check if a char is valid within an NCName (non-first position).
/// Based on XML 1.0 NameChar minus ':'.
fn is_ncname_char(c: char) -> bool {
    let cp = c as u32;
    is_ncname_start_char(c)
        || c == '-'
        || c == '.'
        || matches!(cp, 48..=57 | 0xB7 | 0x0300..=0x036F | 0x203F..=0x2040)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Document;
    use crate::qualified_name::QualifiedName;

    /// Helper function to verify that a rule file exists for the test
    /// This ensures that if a rule is removed, the test will break and be noticed
    fn verify_rule_exists(rule_file: &str) {
        let rule_path = format!("specification/rules/{}", rule_file);
        assert!(
            std::path::Path::new(&rule_path).exists(),
            "Rule file {} does not exist",
            rule_file
        );
    }

    #[test]
    fn test_namespace_support() {
        let doc = Document::empty();
        let namespace = Namespace::prefixed("http://example.com", "ex").unwrap();
        let element =
            doc.create_element(QualifiedName::with_namespace("test", &namespace).unwrap());

        assert_eq!(element.name(), "test");
        assert_eq!(element.namespace(), Some(namespace));
        assert_eq!(element.qualified_name().qualified_name_string(), "ex:test");
    }

    #[test]
    fn test_namespace_validation() {
        // rule: rule.namespace-basics.empty-string-not-namespace-name.md
        verify_rule_exists("rule.namespace-basics.empty-string-not-namespace-name.md");
        // Valid namespace
        assert!(Namespace::prefixed("http://example.com", "ex").is_ok());
        // Prefix with colon
        assert!(Namespace::prefixed("http://example.com", "ex:bad").is_err());
        // Empty prefix
        assert!(Namespace::prefixed("http://example.com", "").is_err());
        // Empty URI
        assert!(Namespace::prefixed("", "ex").is_err());
        // Default namespace with empty URI
        assert!(Namespace::without_prefix("").is_err());
        // Default namespace with valid URI
        assert!(Namespace::without_prefix("http://example.com").is_ok());
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
        let ns4 = Namespace::without_prefix("http://example.com").unwrap();
        let ns5 = Namespace::without_prefix("http://different.com").unwrap();

        // Same URI and prefix, but different Arc pointers
        assert_eq!(ns1, ns2);
        // Same URI, different prefix
        assert_ne!(ns1, ns3);
        // Same URI, one with prefix, one without
        assert_ne!(ns1, ns4);
        // Different URI
        assert_ne!(ns1, ns5);
        // Default namespace equality
        let ns6 = Namespace::without_prefix("http://example.com").unwrap();
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

    #[test]
    fn test_namespace_is_equal_ns() {
        let ns1 = Namespace::prefixed("http://example.com", "ex").unwrap();
        let ns2 = Namespace::without_prefix("http://example.com").unwrap();
        let ns3 = Namespace::prefixed("http://example.com", "other").unwrap();
        let ns4 = Namespace::without_prefix("http://different.com").unwrap();
        // Same URI, different prefixes
        assert!(Namespace::is_equal_ns(&ns1, &ns2));
        assert!(Namespace::is_equal_ns(&ns1, &ns3));
        // Different URIs
        assert!(!Namespace::is_equal_ns(&ns1, &ns4));
        assert!(!Namespace::is_equal_ns(&ns2, &ns4));
        // Identical objects
        assert!(Namespace::is_equal_ns(&ns1, &ns1));
    }

    #[test]
    fn test_reserved_xml_prefix() {
        // rule: rule.namespace-basics.xml-prefix-fixed-binding.md
        verify_rule_exists("rule.namespace-basics.xml-prefix-fixed-binding.md");
        // rule: rule.namespace-basics.no-other-prefix-to-xml-namespace.md
        verify_rule_exists("rule.namespace-basics.no-other-prefix-to-xml-namespace.md");
        // rule: rule.namespace-basics.xml-namespace-not-default.md
        verify_rule_exists("rule.namespace-basics.xml-namespace-not-default.md");
        // `xml` prefix with its reserved URI is valid
        assert!(Namespace::prefixed(RESERVED_XML_URI, "xml").is_ok());

        // `xml` prefix with a different URI is invalid
        assert!(Namespace::prefixed("http://example.com", "xml").is_err());

        // Any other prefix with the reserved XML URI is invalid
        assert!(Namespace::prefixed(RESERVED_XML_URI, "ex").is_err());
        assert!(Namespace::prefixed(RESERVED_XML_URI, "foo").is_err());

        // The reserved XML URI as default namespace is invalid
        assert!(Namespace::without_prefix(RESERVED_XML_URI).is_err());
    }

    #[test]
    fn test_reserved_xmlns_prefix() {
        // rule: rule.namespace-basics.xmlns-prefix-not-declared.md
        verify_rule_exists("rule.namespace-basics.xmlns-prefix-not-declared.md");
        // rule: rule.namespace-basics.no-other-prefix-to-xmlns-namespace.md
        verify_rule_exists("rule.namespace-basics.no-other-prefix-to-xmlns-namespace.md");
        // rule: rule.namespace-basics.xmlns-namespace-not-default.md
        verify_rule_exists("rule.namespace-basics.xmlns-namespace-not-default.md");
        // `xmlns` prefix must never be declared
        assert!(Namespace::prefixed("http://example.com", "xmlns").is_err());
        assert!(Namespace::prefixed(RESERVED_XMLNS_URI, "xmlns").is_err());

        // The reserved xmlns URI cannot be bound to any prefix
        assert!(Namespace::prefixed(RESERVED_XMLNS_URI, "ex").is_err());
        assert!(Namespace::prefixed(RESERVED_XMLNS_URI, "foo").is_err());

        // The reserved xmlns URI as default namespace is invalid
        assert!(Namespace::without_prefix(RESERVED_XMLNS_URI).is_err());
    }

    #[test]
    fn test_prefix_ncname_validation() {
        // rule: rule.namespace-usage.prefix-and-localpart-ncname.md
        verify_rule_exists("rule.namespace-usage.prefix-and-localpart-ncname.md");
        // rule: rule.well-formedness.name-starts-with-valid-char.md
        verify_rule_exists("rule.well-formedness.name-starts-with-valid-char.md");
        // rule: rule.well-formedness.name-valid-chars.md
        verify_rule_exists("rule.well-formedness.name-valid-chars.md");
        // Valid prefixes
        assert!(Namespace::prefixed("http://example.com", "ex").is_ok());
        assert!(Namespace::prefixed("http://example.com", "ex_1").is_ok());
        assert!(Namespace::prefixed("http://example.com", "_private").is_ok());
        assert!(Namespace::prefixed("http://example.com", "a-b").is_ok());
        assert!(Namespace::prefixed("http://example.com", "a.b").is_ok());
        assert!(Namespace::prefixed("http://example.com", "A1_B-c.d").is_ok());
        assert!(Namespace::prefixed("http://www.w3.org/XML/1998/namespace", "xml").is_ok());
        assert!(Namespace::without_prefix("http://example.com").is_ok());

        // Invalid: empty prefix
        assert!(Namespace::prefixed("http://example.com", "").is_err());

        // Invalid: contains colon
        assert!(Namespace::prefixed("http://example.com", "ex:bad").is_err());

        // Invalid: starts with digit
        assert!(Namespace::prefixed("http://example.com", "1bad").is_err());

        // Invalid: contains whitespace
        assert!(Namespace::prefixed("http://example.com", "has space").is_err());

        // Invalid: contains special characters
        assert!(Namespace::prefixed("http://example.com", "bad!").is_err());
        assert!(Namespace::prefixed("http://example.com", "bad@ns").is_err());
        assert!(Namespace::prefixed("http://example.com", "bad#ns").is_err());

        // Invalid: reserved prefixes
        assert!(Namespace::prefixed("http://example.com", "xmlns").is_err());
        assert!(Namespace::prefixed("http://example.com", "xml").is_err());

        // Invalid: reserved URI with wrong prefix
        assert!(Namespace::prefixed("http://www.w3.org/XML/1998/namespace", "ex").is_err());
        assert!(Namespace::prefixed("http://www.w3.org/2000/xmlns/", "ex").is_err());

        // Invalid: empty URI
        assert!(Namespace::without_prefix("").is_err());
    }

    #[test]
    fn test_ncname_helper() {
        // rule: rule.well-formedness.name-starts-with-valid-char.md
        verify_rule_exists("rule.well-formedness.name-starts-with-valid-char.md");
        // rule: rule.well-formedness.name-valid-chars.md
        verify_rule_exists("rule.well-formedness.name-valid-chars.md");
        assert!(is_valid_ncname("a"));
        assert!(is_valid_ncname("_"));
        assert!(is_valid_ncname("a1"));
        assert!(is_valid_ncname("_1"));
        assert!(is_valid_ncname("a-b"));
        assert!(is_valid_ncname("a.b"));
        assert!(is_valid_ncname("a_b"));
        assert!(is_valid_ncname("XMLSchema"));

        // Unicode start characters (NameStartChar ranges)
        assert!(is_valid_ncname("\u{00C0}")); // Latin capital A with grave
        assert!(is_valid_ncname("\u{0391}")); // Greek capital alpha
        assert!(is_valid_ncname("\u{4E00}")); // CJK unified ideograph
        assert!(is_valid_ncname("\u{0628}")); // Arabic letter beh
        assert!(is_valid_ncname("\u{0915}")); // Devanagari letter ka
        assert!(is_valid_ncname("\u{0410}")); // Cyrillic capital A
        assert!(is_valid_ncname("\u{1D400}")); // Byblish letter alaf (supplementary plane)

        // Unicode continuing characters (NameChar ranges)
        assert!(is_valid_ncname("a\u{0300}")); // combining grave accent
        assert!(is_valid_ncname("\u{00C0}bc")); // starts with Latin capital A with grave

        assert!(!is_valid_ncname(""));
        assert!(!is_valid_ncname("1a"));
        assert!(!is_valid_ncname("-a"));
        assert!(!is_valid_ncname(".a"));
        assert!(!is_valid_ncname("a:b"));
        assert!(!is_valid_ncname("a b"));
        assert!(!is_valid_ncname("a!b"));
    }

    #[test]
    fn test_unicode_prefixes() {
        // Valid Unicode prefixes should be accepted
        assert!(Namespace::prefixed("http://example.com", "\u{00C0}").is_ok());
        assert!(Namespace::prefixed("http://example.com", "\u{4E00}\u{4E00}").is_ok());
        assert!(Namespace::prefixed("http://example.com", "\u{0391}bc").is_ok());
    }

    #[test]
    fn test_uri_comparison_case_sensitive() {
        // rule: rule.namespace-basics.uri-comparison-literal-case-sensitive.md
        verify_rule_exists("rule.namespace-basics.uri-comparison-literal-case-sensitive.md");
        let ns1 = Namespace::prefixed("http://example.org/ns", "ex").unwrap();
        let ns2 = Namespace::prefixed("http://example.org/NS", "ex").unwrap();

        // These should be different namespaces due to case sensitivity
        assert!(!Namespace::is_equal_ns(&ns1, &ns2));
        assert_eq!(ns1.uri(), "http://example.org/ns");
        assert_eq!(ns2.uri(), "http://example.org/NS");
    }
}
