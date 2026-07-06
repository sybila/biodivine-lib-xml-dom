use crate::error::XmlError;

/// The `xml` prefix is by definition bound to this namespace.
/// It may, but need not, be declared, and must not be undeclared or bound to any other namespace.
/// Other prefixes must not be bound to this namespace, and it must not be declared as the default namespace.
pub(crate) const RESERVED_XML_URI: &str = "http://www.w3.org/XML/1998/namespace";

/// The `xmlns` prefix is used only to declare namespace bindings and is by definition bound
/// to this namespace. It must not be declared or undeclared. Other prefixes must not be bound
/// to this namespace, and it must not be declared as the default namespace.
pub(crate) const RESERVED_XMLNS_URI: &str = "http://www.w3.org/2000/xmlns/";

/// Check if a string is a valid XML NCName (for namespace prefixes).
///
/// NCName = Name - (Char* ':' Char*), where Name uses NameStartChar and NameChar
/// from XML 1.0 Fifth Edition. This implementation uses the full Unicode ranges.
pub(crate) fn is_valid_ncname(s: &str) -> bool {
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
pub(crate) fn is_ncname_start_char(c: char) -> bool {
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
pub(crate) fn is_ncname_char(c: char) -> bool {
    let cp = c as u32;
    is_ncname_start_char(c)
        || c == '-'
        || c == '.'
        || matches!(cp, 48..=57 | 0xB7 | 0x0300..=0x036F | 0x203F..=0x2040)
}

/// Validate the URI and prefix according to XML namespace rules.
pub(crate) fn validate_namespace(uri: &str, prefix: Option<&str>) -> Result<(), XmlError> {
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
                "Namespace prefix '{p}' is not a valid XML NCName"
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
                    "The prefix 'xml' can only be bound to '{RESERVED_XML_URI}', not '{uri}'"
                )));
            }
        } else {
            // No prefix other than `xml` may bind to the reserved XML URI
            if uri == RESERVED_XML_URI {
                return Err(XmlError::NamespaceError(format!(
                    "The URI '{RESERVED_XML_URI}' can only be bound to the 'xml' prefix, not '{p}'"
                )));
            }
        }
    }

    // Neither reserved URI may be used as a default namespace
    if prefix.is_none() && (uri == RESERVED_XML_URI || uri == RESERVED_XMLNS_URI) {
        return Err(XmlError::NamespaceError(format!(
            "The URI '{uri}' cannot be declared as the default namespace"
        )));
    }

    // No prefix may bind to the reserved xmlns URI
    if uri == RESERVED_XMLNS_URI {
        return Err(XmlError::NamespaceError(format!(
            "The URI '{RESERVED_XMLNS_URI}' cannot be bound to any prefix"
        )));
    }

    Ok(())
}

/// Validate that a local name (the part of a QName after the colon, or the whole
/// name if unqualified) is a valid NCName.
///
/// The local part of a QName must be an NCName per the XML Namespaces specification.
///
/// # Errors
///
/// Returns an error if the name is empty, starts with an invalid character,
/// or contains characters not allowed in an NCName.
pub(crate) fn validate_local_name(name: &str) -> Result<(), XmlError> {
    if name.is_empty() {
        return Err(XmlError::NamespaceError(
            "Local name must not be empty".to_string(),
        ));
    }
    if !is_valid_ncname(name) {
        return Err(XmlError::NamespaceError(format!(
            "Local name '{name}' is not a valid NCName"
        )));
    }
    Ok(())
}

/// Split a qualified name string into its prefix and local name components,
/// validating that the format is correct (at most one colon) and both parts
/// are valid NCNames.
///
/// Returns `(prefix, local_name)` where `prefix` is `None` for unprefixed names.
///
/// # Errors
///
/// Returns an error if:
/// - The name contains more than one colon
/// - The prefix is not a valid NCName
/// - The local name is not a valid NCName
pub(crate) fn split_qname(qname: &str) -> Result<(Option<&str>, &str), XmlError> {
    if qname.is_empty() {
        return Err(XmlError::NamespaceError(
            "Qualified name must not be empty".to_string(),
        ));
    }

    // Check for multiple colons — a QName may have at most one.
    if qname.matches(':').count() > 1 {
        return Err(XmlError::NamespaceError(format!(
            "Qualified name '{qname}' contains more than one colon"
        )));
    }

    if let Some(colon_pos) = qname.find(':') {
        let prefix = &qname[..colon_pos];
        let local_name = &qname[colon_pos + 1..];

        if prefix.is_empty() {
            return Err(XmlError::NamespaceError(
                "Qualified name cannot have an empty prefix".to_string(),
            ));
        }

        validate_ncname(prefix)?;
        validate_local_name(local_name)?;

        Ok((Some(prefix), local_name))
    } else {
        validate_local_name(qname)?;
        Ok((None, qname))
    }
}

/// Validate that an NCName string is valid.
fn validate_ncname(name: &str) -> Result<(), XmlError> {
    if !is_valid_ncname(name) {
        return Err(XmlError::NamespaceError(format!(
            "'{name}' is not a valid NCName"
        )));
    }
    Ok(())
}

/// Validate constraints on a namespace prefix during QName resolution.
///
/// This checks the XML Namespaces specification rules for the `xml` prefix:
/// - The `xml` prefix must only be bound to `http://www.w3.org/XML/1998/namespace`.
/// - The `xmlns` prefix must never be used as a namespace prefix in a QName.
///
/// # Parameters
///
/// - `prefix`: The prefix from the QName (or empty string for unprefixed).
/// - `resolved_uri`: The URI the prefix resolved to, if any.
///
/// # Errors
///
/// Returns an error if the prefix violates namespace constraints.
pub(crate) fn validate_resolved_prefix(
    prefix: &str,
    resolved_uri: Option<&str>,
) -> Result<(), XmlError> {
    if prefix == "xmlns" {
        return Err(XmlError::NamespaceError(
            "The prefix 'xmlns' is reserved and cannot be used in a qualified name".to_string(),
        ));
    }

    if prefix == "xml" {
        match resolved_uri {
            Some(uri) if uri == RESERVED_XML_URI => Ok(()),
            Some(uri) => Err(XmlError::NamespaceError(format!(
                "The prefix 'xml' can only be bound to '{RESERVED_XML_URI}', not '{uri}'"
            ))),
            None => Err(XmlError::NamespaceError(format!(
                "The prefix 'xml' must be bound to '{RESERVED_XML_URI}'"
            ))),
        }
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_namespace_validation() {
        // rule: rule.namespace-basics.empty-string-not-namespace-name.md
        verify_rule_exists("rule.namespace-basics.empty-string-not-namespace-name.md");
        // Valid namespace
        assert!(validate_namespace("http://example.com", Some("ex")).is_ok());
        // Prefix with colon
        assert!(validate_namespace("http://example.com", Some("ex:bad")).is_err());
        // Empty prefix
        assert!(validate_namespace("http://example.com", Some("")).is_err());
        // Empty URI
        assert!(validate_namespace("", Some("ex")).is_err());
        // Default namespace with empty URI
        assert!(validate_namespace("", None).is_err());
        // Default namespace with valid URI
        assert!(validate_namespace("http://example.com", None).is_ok());
        // New with None prefix
        assert!(validate_namespace("http://example.com", None).is_ok());
        // New with Some valid prefix
        assert!(validate_namespace("http://example.com", Some("ex")).is_ok());
        // New with Some invalid prefix
        assert!(validate_namespace("http://example.com", Some("ex:bad")).is_err());
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
        assert!(validate_namespace(RESERVED_XML_URI, Some("xml")).is_ok());

        // `xml` prefix with a different URI is invalid
        assert!(validate_namespace("http://example.com", Some("xml")).is_err());

        // Any other prefix with the reserved XML URI is invalid
        assert!(validate_namespace(RESERVED_XML_URI, Some("ex")).is_err());
        assert!(validate_namespace(RESERVED_XML_URI, Some("foo")).is_err());

        // The reserved XML URI as default namespace is invalid
        assert!(validate_namespace(RESERVED_XML_URI, None).is_err());
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
        assert!(validate_namespace("http://example.com", Some("xmlns")).is_err());
        assert!(validate_namespace(RESERVED_XMLNS_URI, Some("xmlns")).is_err());

        // The reserved xmlns URI cannot be bound to any prefix
        assert!(validate_namespace(RESERVED_XMLNS_URI, Some("ex")).is_err());
        assert!(validate_namespace(RESERVED_XMLNS_URI, Some("foo")).is_err());

        // The reserved xmlns URI as default namespace is invalid
        assert!(validate_namespace(RESERVED_XMLNS_URI, None).is_err());
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
        assert!(validate_namespace("http://example.com", Some("ex")).is_ok());
        assert!(validate_namespace("http://example.com", Some("ex_1")).is_ok());
        assert!(validate_namespace("http://example.com", Some("_private")).is_ok());
        assert!(validate_namespace("http://example.com", Some("a-b")).is_ok());
        assert!(validate_namespace("http://example.com", Some("a.b")).is_ok());
        assert!(validate_namespace("http://example.com", Some("A1_B-c.d")).is_ok());
        assert!(validate_namespace("http://www.w3.org/XML/1998/namespace", Some("xml")).is_ok());
        assert!(validate_namespace("http://example.com", None).is_ok());

        // Invalid: empty prefix
        assert!(validate_namespace("http://example.com", Some("")).is_err());

        // Invalid: contains colon
        assert!(validate_namespace("http://example.com", Some("ex:bad")).is_err());

        // Invalid: starts with digit
        assert!(validate_namespace("http://example.com", Some("1bad")).is_err());

        // Invalid: contains whitespace
        assert!(validate_namespace("http://example.com", Some("has space")).is_err());

        // Invalid: contains special characters
        assert!(validate_namespace("http://example.com", Some("bad!")).is_err());
        assert!(validate_namespace("http://example.com", Some("bad@ns")).is_err());
        assert!(validate_namespace("http://example.com", Some("bad#ns")).is_err());

        // Invalid: reserved prefixes
        assert!(validate_namespace("http://example.com", Some("xmlns")).is_err());
        assert!(validate_namespace("http://example.com", Some("xml")).is_err());

        // Invalid: reserved URI with wrong prefix
        assert!(validate_namespace("http://www.w3.org/XML/1998/namespace", Some("ex")).is_err());
        assert!(validate_namespace("http://www.w3.org/2000/xmlns/", Some("ex")).is_err());

        // Invalid: empty URI
        assert!(validate_namespace("", None).is_err());
    }

    #[test]
    fn test_uri_comparison_case_sensitive() {
        // rule: rule.namespace-basics.uri-comparison-literal-case-sensitive.md
        verify_rule_exists("rule.namespace-basics.uri-comparison-literal-case-sensitive.md");
        assert!(validate_namespace("http://example.org/ns", Some("ex")).is_ok());
        assert!(validate_namespace("http://example.org/NS", Some("ex")).is_ok());
    }

    #[test]
    fn test_split_qname_valid() {
        // rule: rule.namespace-usage.qname-format.md
        verify_rule_exists("rule.namespace-usage.qname-format.md");
        // rule: rule.namespace-usage.zero-or-one-colon.md
        verify_rule_exists("rule.namespace-usage.zero-or-one-colon.md");
        // rule: rule.namespace-usage.prefix-and-localpart-ncname.md
        verify_rule_exists("rule.namespace-usage.prefix-and-localpart-ncname.md");
        // Unprefixed
        let (prefix, local) = split_qname("foo").unwrap();
        assert_eq!(prefix, None);
        assert_eq!(local, "foo");

        // Prefixed
        let (prefix, local) = split_qname("ex:foo").unwrap();
        assert_eq!(prefix, Some("ex"));
        assert_eq!(local, "foo");

        // Unicode
        let (prefix, local) = split_qname("\u{00C0}:\u{4E00}").unwrap();
        assert_eq!(prefix, Some("\u{00C0}"));
        assert_eq!(local, "\u{4E00}");
    }

    #[test]
    fn test_split_qname_invalid() {
        // rule: rule.namespace-usage.zero-or-one-colon.md
        verify_rule_exists("rule.namespace-usage.zero-or-one-colon.md");
        // Multiple colons
        assert!(split_qname("a:b:c").is_err());

        // Empty name
        assert!(split_qname("").is_err());

        // Empty prefix
        assert!(split_qname(":foo").is_err());

        // Empty local part
        assert!(split_qname("ex:").is_err());

        // Invalid prefix (starts with digit)
        assert!(split_qname("1a:foo").is_err());

        // Invalid local part (starts with digit)
        assert!(split_qname("ex:1foo").is_err());

        // Invalid local part (contains space)
        assert!(split_qname("ex:foo bar").is_err());
    }

    #[test]
    fn test_validate_local_name() {
        // rule: rule.namespace-usage.prefix-and-localpart-ncname.md
        verify_rule_exists("rule.namespace-usage.prefix-and-localpart-ncname.md");
        assert!(validate_local_name("foo").is_ok());
        assert!(validate_local_name("_").is_ok());
        assert!(validate_local_name("a-b").is_ok());
        assert!(validate_local_name("a.b").is_ok());
        assert!(validate_local_name("a1").is_ok());

        assert!(validate_local_name("").is_err());
        assert!(validate_local_name("1a").is_err());
        assert!(validate_local_name("a b").is_err());
        assert!(validate_local_name("a:b").is_err());
    }

    #[test]
    fn test_validate_resolved_prefix() {
        // rule: rule.namespace-basics.xml-prefix-fixed-binding.md
        verify_rule_exists("rule.namespace-basics.xml-prefix-fixed-binding.md");
        // Valid: regular prefix
        assert!(validate_resolved_prefix("ex", Some("http://example.com")).is_ok());
        assert!(validate_resolved_prefix("ex", None).is_ok());

        // Valid: xml prefix with correct URI
        assert!(validate_resolved_prefix("xml", Some(RESERVED_XML_URI)).is_ok());

        // Invalid: xml prefix with wrong URI
        assert!(validate_resolved_prefix("xml", Some("http://example.com")).is_err());

        // Invalid: xml prefix unbound
        assert!(validate_resolved_prefix("xml", None).is_err());

        // Invalid: xmlns prefix
        assert!(validate_resolved_prefix("xmlns", Some("http://example.com")).is_err());
    }
}
