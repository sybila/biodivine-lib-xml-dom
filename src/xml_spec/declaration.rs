//! The XML declaration (`<?xml version="1.0" encoding="UTF-8" standalone="yes"?>`).
//!
//! XML 1.0 §2.8 defines the declaration as `XMLDecl ::= '<?xml' VersionInfo EncodingDecl?
//! SDDecl? S? '?>'`. It is *not* a processing instruction and not part of the document tree; it is
//! a property of the document, which is why it lives on [`crate::Document`] rather than in the
//! arena as a node.
//!
//! This crate only supports XML 1.0 with UTF-8 input, so the interesting part is the *checking*:
//!
//! * `rule.entities.encoding-must-match-declaration` — the declared encoding must describe the
//!   actual encoding of the input; since only UTF-8 is supported, anything else is an error;
//! * `rule.entities.encoding-name-case-insensitive` — `UTF-8`, `utf-8` and `Utf8` all name the
//!   same encoding ("the name ... is case-insensitive");
//! * `rule.entities.no-encoding-legal-utf-required` — a declaration without an encoding is legal
//!   as long as the input is valid UTF-8 (and it must be, for a character to be usable);
//! * `rule.entities.default-encoding-utf8` — UTF-8 is the default when no encoding is declared.

use std::fmt;

/// The version string this library supports.
pub const XML_VERSION: &str = "1.0";

/// The only encoding this library supports.
pub const UTF8_ENCODING_NAME: &str = "UTF-8";

/// The XML declaration of a document.
///
/// See the module documentation for why this is a value on the document rather than a node.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct XmlDeclaration {
    version: String,
    encoding: Option<String>,
    standalone: Option<bool>,
}

impl XmlDeclaration {
    /// Creates a declaration from its parts, without validating the version or encoding.
    ///
    /// Validation is the parser's job (it has to report [`crate::XmlError::UnsupportedXmlVersion`]
    /// or [`crate::XmlError::UnsupportedEncoding`]), so this constructor is deliberately cheap and
    /// infallible.
    pub fn new(
        version: impl Into<String>,
        encoding: Option<impl Into<String>>,
        standalone: Option<bool>,
    ) -> Self {
        Self {
            version: version.into(),
            encoding: encoding.map(Into::into),
            standalone,
        }
    }

    /// The declaration this crate writes by default: version 1.0, UTF-8, no `standalone`.
    pub fn utf8() -> Self {
        Self::new(XML_VERSION, Some(UTF8_ENCODING_NAME), None)
    }

    /// The declared XML version.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// The declared encoding, if the declaration contains one.
    pub fn encoding(&self) -> Option<&str> {
        self.encoding.as_deref()
    }

    /// The declared `standalone` value, if the declaration contains one.
    pub fn standalone(&self) -> Option<bool> {
        self.standalone
    }

    /// The pseudo-attributes of the declaration, without the surrounding `<?xml` and `?>`.
    ///
    /// This is the form the serializer needs: `quick-xml` writes `<?`, the content and `?>`
    /// around a declaration event, so the content must be exactly `version="…" …`.
    pub fn pseudo_attributes(&self) -> String {
        let mut result = format!("version=\"{}\"", self.version);
        if let Some(encoding) = &self.encoding {
            result.push_str(&format!(" encoding=\"{encoding}\""));
        }
        match self.standalone {
            Some(true) => result.push_str(" standalone=\"yes\""),
            Some(false) => result.push_str(" standalone=\"no\""),
            None => {}
        }
        result
    }
}

impl fmt::Display for XmlDeclaration {
    /// Renders the declaration exactly as it must appear at the start of a document.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<?xml version=\"{}\"", self.version)?;
        if let Some(encoding) = &self.encoding {
            write!(f, " encoding=\"{encoding}\"")?;
        }
        match self.standalone {
            Some(true) => write!(f, " standalone=\"yes\"")?,
            Some(false) => write!(f, " standalone=\"no\"")?,
            None => {}
        }
        write!(f, "?>")
    }
}

/// Whether `encoding` names UTF-8.
///
/// `rule.entities.encoding-name-case-insensitive`: the encoding name is compared
/// case-insensitively. The `UTF-8` spelling is also accepted in its `utf8` form, which is what
/// every XML processor in practice accepts (the specification's own `EncName` production requires
/// the letters and the digits with an optional separator, and `utf8` is a valid `EncName` that
/// unambiguously names the same encoding).
pub fn encoding_is_utf8(encoding: &str) -> bool {
    encoding.eq_ignore_ascii_case("utf-8") || encoding.eq_ignore_ascii_case("utf8")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml_spec::rules::assert_rule_exists;

    #[test]
    fn declaration_rendering() {
        assert_eq!(
            XmlDeclaration::utf8().to_string(),
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>"
        );
        assert_eq!(
            XmlDeclaration::new("1.0", None::<String>, None).to_string(),
            "<?xml version=\"1.0\"?>"
        );
        assert_eq!(
            XmlDeclaration::new("1.0", Some("UTF-8"), Some(true)).to_string(),
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>"
        );
        assert_eq!(
            XmlDeclaration::new("1.0", Some("UTF-8"), Some(false)).to_string(),
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>"
        );
    }

    #[test]
    fn pseudo_attributes() {
        assert_eq!(
            XmlDeclaration::utf8().pseudo_attributes(),
            "version=\"1.0\" encoding=\"UTF-8\""
        );
        assert_eq!(
            XmlDeclaration::new("1.0", None::<String>, Some(false)).pseudo_attributes(),
            "version=\"1.0\" standalone=\"no\""
        );
    }

    #[test]
    fn accessors() {
        let declaration = XmlDeclaration::new("1.0", Some("UTF-8"), Some(true));
        assert_eq!(declaration.version(), "1.0");
        assert_eq!(declaration.encoding(), Some("UTF-8"));
        assert_eq!(declaration.standalone(), Some(true));
        assert_eq!(XmlDeclaration::utf8().standalone(), None);
    }

    #[test]
    fn encoding_names_are_case_insensitive() {
        // rule: rule.entities.encoding-name-case-insensitive.md
        assert_rule_exists("rule.entities.encoding-name-case-insensitive.md");
        assert_rule_exists("rule.entities.encoding-must-match-declaration.md");
        assert_rule_exists("rule.entities.default-encoding-utf8.md");
        assert_rule_exists("rule.entities.no-encoding-legal-utf-required.md");

        assert!(encoding_is_utf8("UTF-8"));
        assert!(encoding_is_utf8("utf-8"));
        assert!(encoding_is_utf8("Utf8"));
        assert!(encoding_is_utf8("uTf-8"));
        assert!(!encoding_is_utf8("UTF-16"));
        assert!(!encoding_is_utf8("ISO-8859-1"));
        assert!(!encoding_is_utf8(""));
    }
}
