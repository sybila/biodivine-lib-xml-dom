use crate::error::XmlError;
use std::fmt;
use std::ops::Deref;

/// Helper to implement `TryFrom<&str>` and `TryFrom<String>` for wrapper types.
///
/// Takes a validation closure, a display name for error messages, and the raw string.
/// Returns `Ok(wrapper)` if valid, or `Err(XmlError::InvalidXml(...))` otherwise.
fn try_from_str<F, W>(s: &str, display_name: &str, validate: F) -> Result<W, XmlError>
where
    F: FnOnce(&str) -> bool,
    W: From<String>,
{
    if validate(s) {
        Ok(W::from(s.to_string()))
    } else {
        Err(XmlError::InvalidXml(format!(
            "'{s}' is not a valid {display_name}"
        )))
    }
}

/// The `xml` prefix is by definition bound to this namespace.
/// It may, but need not, be declared, and must not be undeclared or bound to any other namespace.
/// Other prefixes must not be bound to this namespace, and it must not be declared as the default namespace.
pub(crate) const RESERVED_XML_URI: &str = "http://www.w3.org/XML/1998/namespace";

/// The `xmlns` prefix is used only to declare namespace bindings and is by definition bound
/// to this namespace. It must not be declared or undeclared. Other prefixes must not be bound
/// to this namespace, and it must not be declared as the default namespace.
pub(crate) const RESERVED_XMLNS_URI: &str = "http://www.w3.org/2000/xmlns/";

/// Represents a valid XML NCName (Name without colons).
///
/// An NCName is an XML Name that does not contain a colon (`:`). It is used for both
/// namespace prefixes and the local part of a qualified name (QName).
///
/// This type guarantees that the contained string is a valid NCName per the XML 1.0
/// Fifth Edition specification. Validity is enforced at construction time via
/// [`TryFrom`] — once constructed, the value is guaranteed valid.
///
/// # Example
///
/// ```rust
/// use biodivine_lib_xml_dom::xml_spec::NCName;
/// use std::convert::TryInto;
///
/// let name: NCName = "myElement".try_into().unwrap();
/// assert_eq!(name.as_str(), "myElement");
/// assert_eq!(name.as_ref(), "myElement");
/// assert_eq!(name.to_string(), "myElement");
///
/// // Invalid NCNames are rejected
/// let invalid1: Result<NCName, _> = "<tag>".try_into();
/// assert!(invalid1.is_err());
/// let invalid2: Result<NCName, _> = "123".try_into();
/// assert!(invalid2.is_err());
/// let invalid3: Result<NCName, _> = "a:b".try_into();
/// assert!(invalid3.is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NCName(String);

impl NCName {
    /// Returns the NCName as a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use biodivine_lib_xml_dom::xml_spec::NCName;
    /// let name: NCName = "foo".try_into().unwrap();
    /// assert_eq!(name.as_str(), "foo");
    /// ```
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for NCName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for NCName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NCName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq<str> for NCName {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for NCName {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<String> for NCName {
    fn eq(&self, other: &String) -> bool {
        self.0 == *other
    }
}

impl<'a> From<&'a NCName> for std::borrow::Cow<'a, str> {
    fn from(ncname: &'a NCName) -> Self {
        std::borrow::Cow::Borrowed(&ncname.0)
    }
}

impl TryFrom<&str> for NCName {
    type Error = XmlError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if is_valid_ncname(s) {
            Ok(NCName(s.to_string()))
        } else {
            Err(XmlError::InvalidXml(format!("'{s}' is not a valid NCName")))
        }
    }
}

impl TryFrom<String> for NCName {
    type Error = XmlError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        if is_valid_ncname(&s) {
            Ok(NCName(s))
        } else {
            Err(XmlError::InvalidXml(format!("'{s}' is not a valid NCName")))
        }
    }
}

/// Represents a valid XML character data string.
///
/// A `Text` contains only characters legal in XML documents per the `Char` production:
/// `#x9 | #xA | #xD | [#x20-#xD7FF] | [#xE000-#xFFFD] | [#x10000-#x10FFFF]`.
/// This excludes control characters (except tab, LF, CR) and surrogate code points.
///
/// Validity is enforced at construction time via [`TryFrom`].
///
/// # Example
///
/// ```rust
/// use biodivine_lib_xml_dom::xml_spec::Text;
/// use std::convert::TryInto;
///
/// let text: Text = "Hello, World!".try_into().unwrap();
/// assert_eq!(text.as_str(), "Hello, World!");
///
/// // Control characters are rejected
/// let invalid: Result<Text, _> = "control \u{01}".try_into();
/// assert!(invalid.is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Text(String);

impl Text {
    /// Returns the text as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for Text {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for Text {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<&str> for Text {
    type Error = XmlError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        try_from_str(s, "XML text", is_valid_text).map(Text)
    }
}

impl TryFrom<String> for Text {
    type Error = XmlError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        try_from_str(&s, "XML text", is_valid_text).map(Text)
    }
}

/// Check if a string contains only legal XML characters (Char production).
fn is_valid_text(s: &str) -> bool {
    s.chars().all(is_legal_char)
}

/// Check if a char is a legal XML character per the Char production.
/// Char ::= #x9 | #xA | #xD | [#x20-#xD7FF] | [#xE000-#xFFFD] | [#x10000-#x10FFFF]
fn is_legal_char(c: char) -> bool {
    let cp = c as u32;
    matches!(cp, 0x9 | 0xA | 0xD | 0x20..=0xD7FF | 0xE000..=0xFFFD | 0x10000..=0x10FFFF)
}

/// Represents a valid CDATA section content.
///
/// CDATA section content must not contain the string `]]>`, which terminates the section.
/// No other restrictions apply — all legal XML characters are permitted.
///
/// Validity is enforced at construction time via [`TryFrom`].
///
/// # Example
///
/// ```rust
/// use biodivine_lib_xml_dom::xml_spec::CData;
/// use std::convert::TryInto;
///
/// let cdata: CData = "This is safe content".try_into().unwrap();
/// assert_eq!(cdata.as_str(), "This is safe content");
///
/// // Content containing ]]> is rejected
/// let invalid: Result<CData, _> = "contains ]]> end".try_into();
/// assert!(invalid.is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CData(String);

impl CData {
    /// Returns the CDATA content as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for CData {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for CData {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<&str> for CData {
    type Error = XmlError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        try_from_str(s, "CDATA content", |s| !s.contains("]]>")).map(CData)
    }
}

impl TryFrom<String> for CData {
    type Error = XmlError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        try_from_str(&s, "CDATA content", |s| !s.contains("]]>")).map(CData)
    }
}

/// Represents a valid XML comment content.
///
/// Per XML 1.0 §2.5, comment content must not contain the string `--` (double-hyphen),
/// and must not end with a single hyphen `-` (since the comment closes with `-->`).
///
/// Validity is enforced at construction time via [`TryFrom`].
///
/// # Example
///
/// ```rust
/// use biodivine_lib_xml_dom::xml_spec::Comment;
/// use std::convert::TryInto;
///
/// let comment: Comment = " This is valid ".try_into().unwrap();
/// assert_eq!(comment.as_str(), " This is valid ");
///
/// // Double hyphen is rejected
/// let invalid: Result<Comment, _> = "has -- double hyphen".try_into();
/// assert!(invalid.is_err());
///
/// // Trailing hyphen is rejected
/// let invalid2: Result<Comment, _> = "ends with -".try_into();
/// assert!(invalid2.is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Comment(String);

impl Comment {
    /// Returns the comment content as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for Comment {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for Comment {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Comment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<&str> for Comment {
    type Error = XmlError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        try_from_str(s, "XML comment", |s| !s.contains("--") && !s.ends_with('-')).map(Comment)
    }
}

impl TryFrom<String> for Comment {
    type Error = XmlError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        try_from_str(&s, "XML comment", |s| {
            !s.contains("--") && !s.ends_with('-')
        })
        .map(Comment)
    }
}

/// Represents a valid processing instruction target.
///
/// A PI target must be a valid XML `Name` (not an NCName — colons are allowed)
/// and must not match `xml` case-insensitively.
///
/// Validity is enforced at construction time via [`TryFrom`].
///
/// # Example
///
/// ```rust
/// use biodivine_lib_xml_dom::xml_spec::PiTarget;
/// use std::convert::TryInto;
///
/// let target: PiTarget = "xml-stylesheet".try_into().unwrap();
/// assert_eq!(target.as_str(), "xml-stylesheet");
///
/// // Case-insensitive xml is rejected
/// let invalid: Result<PiTarget, _> = "xml".try_into();
/// assert!(invalid.is_err());
/// let invalid2: Result<PiTarget, _> = "XML".try_into();
/// assert!(invalid2.is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PiTarget(String);

impl PiTarget {
    /// Returns the PI target as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for PiTarget {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for PiTarget {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PiTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<&str> for PiTarget {
    type Error = XmlError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        try_from_str(s, "PI target", |s| {
            is_valid_name(s) && !s.eq_ignore_ascii_case("xml")
        })
        .map(PiTarget)
    }
}

impl TryFrom<String> for PiTarget {
    type Error = XmlError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        try_from_str(&s, "PI target", |s| {
            is_valid_name(s) && !s.eq_ignore_ascii_case("xml")
        })
        .map(PiTarget)
    }
}

/// Represents a valid processing instruction content.
///
/// PI content must not contain the string `?>`, which terminates the PI.
///
/// Validity is enforced at construction time via [`TryFrom`].
///
/// # Example
///
/// ```rust
/// use biodivine_lib_xml_dom::xml_spec::PiData;
/// use std::convert::TryInto;
///
/// let data: PiData = "type=\"text/css\" href=\"style.css\"".try_into().unwrap();
/// assert_eq!(data.as_str(), "type=\"text/css\" href=\"style.css\"");
///
/// // Content containing ?> is rejected
/// let invalid: Result<PiData, _> = "has ?> in it".try_into();
/// assert!(invalid.is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PiData(String);

impl PiData {
    /// Returns the PI content as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for PiData {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for PiData {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PiData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<&str> for PiData {
    type Error = XmlError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        try_from_str(s, "PI content", |s| !s.contains("?>")).map(PiData)
    }
}

impl TryFrom<String> for PiData {
    type Error = XmlError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        try_from_str(&s, "PI content", |s| !s.contains("?>")).map(PiData)
    }
}

/// Check if a string is a valid XML Name (for PI targets, element names, etc.).
///
/// Name = NameStartChar NameChar*
/// Unlike NCName, XML Names may contain colons.
fn is_valid_name(s: &str) -> bool {
    is_valid_name_with_colon(s, true)
}

/// Check if a string is a valid XML NCName (for namespace prefixes).
///
/// NCName = Name - (Char* ':' Char*), where Name uses NameStartChar and NameChar
/// from XML 1.0 Fifth Edition. This implementation uses the full Unicode ranges.
///
/// Note: This function is private in `xml_spec`, because instead of performing manual checks,
/// we should be using [`NCName`] struct instead.
fn is_valid_ncname(s: &str) -> bool {
    is_valid_name_with_colon(s, false)
}

/// Core name validation with a parameter for whether colons are allowed.
///
/// When `allow_colon` is `true`, `:` is permitted as both a start char and a name char
/// (full XML Name). When `false`, `:` is forbidden (NCName).
fn is_valid_name_with_colon(s: &str, allow_colon: bool) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    if let Some(first) = chars.next()
        && !is_name_start_char(first, allow_colon)
    {
        return false;
    }
    chars.all(|c| is_name_char(c, allow_colon))
}

/// Check if a char is valid as the first character of an XML Name or NCName.
///
/// When `allow_colon` is `true`, `:` is permitted (Name). When `false`, it is not (NCName).
fn is_name_start_char(c: char, allow_colon: bool) -> bool {
    let cp = c as u32;
    (allow_colon && c == ':')
        || c == '_'
        || matches!(cp, 65..=90 | 97..=122)
        || matches!(cp, 0xC0..=0xD6 | 0xD8..=0xF6 | 0xF8..=0x2FF)
        || matches!(cp, 0x370..=0x37D | 0x37F..=0x1FFF)
        || matches!(cp, 0x200C..=0x200D | 0x2070..=0x218F)
        || matches!(cp, 0x2C00..=0x2FEF | 0x3001..=0xD7FF)
        || matches!(cp, 0xF900..=0xFDCF | 0xFDF0..=0xFFFD)
        || matches!(cp, 0x10000..=0xEFFFF)
}

/// Check if a char is valid within an XML Name or NCName (non-first position).
///
/// When `allow_colon` is `true`, `:` is permitted (Name). When `false`, it is not (NCName).
fn is_name_char(c: char, allow_colon: bool) -> bool {
    let cp = c as u32;
    is_name_start_char(c, allow_colon)
        || c == '-'
        || c == '.'
        || matches!(cp, 48..=57 | 0xB7 | 0x0300..=0x036F | 0x203F..=0x2040)
}

/// Validate the URI and prefix according to XML namespace rules.
pub(crate) fn validate_namespace(uri: &str, prefix: Option<&NCName>) -> Result<(), XmlError> {
    // URI must not be empty
    if uri.is_empty() {
        return Err(XmlError::NamespaceError(
            "Namespace URI must not be empty".to_string(),
        ));
    }

    // Prefix must be a valid NCName (guaranteed by NCName type)
    if let Some(p) = prefix {
        // `xmlns` prefix must never be declared
        if p == "xmlns" {
            return Err(XmlError::NamespaceError(
                "The prefix 'xmlns' is reserved and cannot be declared".to_string(),
            ));
        }

        // `xml` prefix must only bind to its reserved URI
        if p == "xml" {
            validate_xml_prefix_binding(Some(uri))?;
        } else if uri == RESERVED_XML_URI {
            // No prefix other than `xml` may bind to the reserved XML URI
            return Err(XmlError::NamespaceError(format!(
                "The URI '{RESERVED_XML_URI}' can only be bound to the 'xml' prefix, not '{p}'"
            )));
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

/// Split a qualified name string into its prefix and local name components,
/// validating that the format is correct (at most one colon) and both parts
/// are valid NCNames.
///
/// Returns `(prefix, local_name)` where `prefix` is `None` for unprefixed names.
/// Both returned [`NCName`] values are guaranteed valid.
///
/// # Errors
///
/// Returns an error if:
/// - The name contains more than one colon
/// - The prefix is not a valid NCName
/// - The local name is not a valid NCName
pub(crate) fn split_qname(qname: &str) -> Result<(Option<NCName>, NCName), XmlError> {
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
        let prefix_str = &qname[..colon_pos];
        let local_str = &qname[colon_pos + 1..];

        if prefix_str.is_empty() {
            return Err(XmlError::NamespaceError(
                "Qualified name cannot have an empty prefix".to_string(),
            ));
        }

        let prefix = NCName::try_from(prefix_str)?;
        let local_name = NCName::try_from(local_str)?;

        Ok((Some(prefix), local_name))
    } else {
        let local_name = NCName::try_from(qname)?;
        Ok((None, local_name))
    }
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
        validate_xml_prefix_binding(resolved_uri)
    } else {
        Ok(())
    }
}

/// Validate that the `xml` prefix is bound to its reserved URI.
///
/// # Parameters
///
/// - `uri`: The URI the `xml` prefix is bound to, if any.
///
/// # Errors
///
/// Returns an error if the URI is not the reserved XML namespace URI.
fn validate_xml_prefix_binding(uri: Option<&str>) -> Result<(), XmlError> {
    match uri {
        Some(u) if u == RESERVED_XML_URI => Ok(()),
        Some(u) => Err(XmlError::NamespaceError(format!(
            "The prefix 'xml' can only be bound to '{RESERVED_XML_URI}', not '{u}'"
        ))),
        None => Err(XmlError::NamespaceError(format!(
            "The prefix 'xml' must be bound to '{RESERVED_XML_URI}'"
        ))),
    }
}

/// Helper function to create an NCName. Panics if the string is invalid.
/// Primarily intended for use in tests and examples.
pub fn nc_name(s: &str) -> NCName {
    NCName::try_from(s).unwrap()
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
        assert!(validate_namespace("http://example.com", Some(&nc_name("ex"))).is_ok());
        // Empty URI
        assert!(validate_namespace("", Some(&nc_name("ex"))).is_err());
        // Default namespace with empty URI
        assert!(validate_namespace("", None).is_err());
        // Default namespace with valid URI
        assert!(validate_namespace("http://example.com", None).is_ok());
        // New with None prefix
        assert!(validate_namespace("http://example.com", None).is_ok());
        // New with Some valid prefix
        assert!(validate_namespace("http://example.com", Some(&nc_name("ex"))).is_ok());
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
        assert!(validate_namespace(RESERVED_XML_URI, Some(&nc_name("xml"))).is_ok());

        // `xml` prefix with a different URI is invalid
        assert!(validate_namespace("http://example.com", Some(&nc_name("xml"))).is_err());

        // Any other prefix with the reserved XML URI is invalid
        assert!(validate_namespace(RESERVED_XML_URI, Some(&nc_name("ex"))).is_err());
        assert!(validate_namespace(RESERVED_XML_URI, Some(&nc_name("foo"))).is_err());

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
        assert!(validate_namespace("http://example.com", Some(&nc_name("xmlns"))).is_err());
        assert!(validate_namespace(RESERVED_XMLNS_URI, Some(&nc_name("xmlns"))).is_err());

        // The reserved xmlns URI cannot be bound to any prefix
        assert!(validate_namespace(RESERVED_XMLNS_URI, Some(&nc_name("ex"))).is_err());
        assert!(validate_namespace(RESERVED_XMLNS_URI, Some(&nc_name("foo"))).is_err());

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
        assert!(validate_namespace("http://example.com", Some(&nc_name("ex"))).is_ok());
        assert!(validate_namespace("http://example.com", Some(&nc_name("ex_1"))).is_ok());
        assert!(validate_namespace("http://example.com", Some(&nc_name("_private"))).is_ok());
        assert!(validate_namespace("http://example.com", Some(&nc_name("a-b"))).is_ok());
        assert!(validate_namespace("http://example.com", Some(&nc_name("a.b"))).is_ok());
        assert!(validate_namespace("http://example.com", Some(&nc_name("A1_B-c.d"))).is_ok());
        assert!(validate_namespace(RESERVED_XML_URI, Some(&nc_name("xml"))).is_ok());
        assert!(validate_namespace("http://example.com", None).is_ok());

        // Invalid: reserved URI with wrong prefix
        assert!(validate_namespace(RESERVED_XML_URI, Some(&nc_name("ex"))).is_err());
        assert!(validate_namespace(RESERVED_XMLNS_URI, Some(&nc_name("ex"))).is_err());

        // Invalid: empty URI
        assert!(validate_namespace("", None).is_err());
    }

    #[test]
    fn test_uri_comparison_case_sensitive() {
        // rule: rule.namespace-basics.uri-comparison-literal-case-sensitive.md
        verify_rule_exists("rule.namespace-basics.uri-comparison-literal-case-sensitive.md");
        assert!(validate_namespace("http://example.org/ns", Some(&nc_name("ex"))).is_ok());
        assert!(validate_namespace("http://example.org/NS", Some(&nc_name("ex"))).is_ok());
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
        assert_eq!(local.as_str(), "foo");

        // Prefixed
        let (prefix, local) = split_qname("ex:foo").unwrap();
        assert_eq!(prefix, Some(nc_name("ex")));
        assert_eq!(local.as_str(), "foo");

        // Unicode
        let (prefix, local) = split_qname("\u{00C0}:\u{4E00}").unwrap();
        assert_eq!(prefix, Some(nc_name("\u{00C0}")));
        assert_eq!(local.as_str(), "\u{4E00}");
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

    #[test]
    fn test_text_wrapper() {
        // rule: rule.well-formedness.legal-characters.md
        verify_rule_exists("rule.well-formedness.legal-characters.md");
        // Valid text
        let text: Text = "Hello, World!".try_into().unwrap();
        assert_eq!(text.as_str(), "Hello, World!");

        // Tab, LF, CR are allowed
        let text: Text = "line1\nline2\ttab\r\ncr".try_into().unwrap();
        assert_eq!(text.as_str(), "line1\nline2\ttab\r\ncr");

        // Surrogates are rejected (U+D800 and U+DFFF are surrogates)
        // We can't write surrogates directly in Rust strings, so we test via
        // the is_legal_char function through invalid UTF-8 byte sequences
        // that would decode to surrogates if they were valid UTF-8.
        // Instead, we verify control chars and other illegal ranges.
        assert!(Text::try_from("control \u{01}").is_err());
        assert!(Text::try_from("control \u{1F}").is_err());

        // Unicode in legal range
        let text: Text = "\u{00C0}\u{4E00}\u{0628}".try_into().unwrap();
        assert_eq!(text.as_str(), "\u{00C0}\u{4E00}\u{0628}");
    }

    #[test]
    fn test_cdata_wrapper() {
        // rule: rule.document-structure.cdata-section-must-not-contain-cdend.md
        verify_rule_exists("rule.document-structure.cdata-section-must-not-contain-cdend.md");
        // Valid CDATA
        let cdata: CData = "This is safe content".try_into().unwrap();
        assert_eq!(cdata.as_str(), "This is safe content");

        // CDATA with special chars is fine
        let cdata: CData = "contains < > & \" ' chars".try_into().unwrap();
        assert_eq!(cdata.as_str(), "contains < > & \" ' chars");

        // ]]> is rejected
        assert!(CData::try_from("contains ]]> end").is_err());
        assert!(CData::try_from("]]> at start").is_err());
    }

    #[test]
    fn test_comment_wrapper() {
        // rule: rule.well-formedness.comment-no-double-hyphen.md
        verify_rule_exists("rule.well-formedness.comment-no-double-hyphen.md");
        // rule: rule.well-formedness.comment-no-triple-hyphen.md
        verify_rule_exists("rule.well-formedness.comment-no-triple-hyphen.md");
        // Valid comment
        let comment: Comment = " This is valid ".try_into().unwrap();
        assert_eq!(comment.as_str(), " This is valid ");

        // Single hyphen is fine
        let comment: Comment = "a-b".try_into().unwrap();
        assert_eq!(comment.as_str(), "a-b");

        // Double hyphen is rejected
        assert!(Comment::try_from("has -- double").is_err());

        // Trailing hyphen is rejected
        assert!(Comment::try_from("ends with -").is_err());

        // Triple hyphen is rejected (contains --)
        assert!(Comment::try_from("has --- triple").is_err());
    }

    #[test]
    fn test_pi_target_wrapper() {
        // rule: rule.well-formedness.pi-target-is-name.md
        verify_rule_exists("rule.well-formedness.pi-target-is-name.md");
        // rule: rule.well-formedness.pi-target-not-xml.md
        verify_rule_exists("rule.well-formedness.pi-target-not-xml.md");
        // Valid PI targets
        let target: PiTarget = "xml-stylesheet".try_into().unwrap();
        assert_eq!(target.as_str(), "xml-stylesheet");

        let target: PiTarget = "mytarget".try_into().unwrap();
        assert_eq!(target.as_str(), "mytarget");

        // PI target with colon is valid (Name, not NCName)
        let target: PiTarget = "my:target".try_into().unwrap();
        assert_eq!(target.as_str(), "my:target");

        // Case-insensitive xml is rejected
        assert!(PiTarget::try_from("xml").is_err());
        assert!(PiTarget::try_from("XML").is_err());
        assert!(PiTarget::try_from("Xml").is_err());
        assert!(PiTarget::try_from("xMl").is_err());

        // Invalid name (starts with digit)
        assert!(PiTarget::try_from("1invalid").is_err());
    }

    #[test]
    fn test_pi_data_wrapper() {
        // rule: rule.well-formedness.pi-no-contains-close.md
        verify_rule_exists("rule.well-formedness.pi-no-contains-close.md");
        // Valid PI data
        let data: PiData = r#"type="text/css" href="style.css""#.try_into().unwrap();
        assert_eq!(data.as_str(), r#"type="text/css" href="style.css""#);

        let data: PiData = r#"echo "Hello, World!";"#.try_into().unwrap();
        assert_eq!(data.as_str(), r#"echo "Hello, World!";"#);

        // ?> is rejected
        assert!(PiData::try_from("has ?> in it").is_err());
        assert!(PiData::try_from("?> at start").is_err());
    }
}
