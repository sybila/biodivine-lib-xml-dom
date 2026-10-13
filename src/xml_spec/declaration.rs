//! The XML declaration (`<?xml version="1.0" encoding="UTF-8" standalone="yes"?>`).
//!
//! XML 1.0 §2.8 defines the declaration as `XMLDecl ::= '<?xml' VersionInfo EncodingDecl?
//! SDDecl? S? '?>'`. It is *not* a processing instruction and not part of the document tree; it is
//! a property of the document, which is why it lives on [`crate::Document`] rather than in the
//! arena as a node.
//!
//! This crate only supports XML 1.0 with UTF-8 input, so the interesting part is the *checking*:
//!
//! * [`rule.entities.encoding-must-match-declaration`](crate::xml_spec::rule_set::rule_entities_encoding_must_match_declaration) — the declared encoding must describe the
//!   actual encoding of the input; since only UTF-8 is supported, anything else is an error;
//! * [`rule.entities.encoding-name-case-insensitive`](crate::xml_spec::rule_set::rule_entities_encoding_name_case_insensitive) — `UTF-8`, `utf-8` and `Utf8` all name the
//!   same encoding ("the name ... is case-insensitive");
//! * [`rule.entities.no-encoding-legal-utf-required`](crate::xml_spec::rule_set::rule_entities_no_encoding_legal_utf_required) — a declaration without an encoding is legal
//!   as long as the input is valid UTF-8 (and it must be, for a character to be usable);
//! * [`rule.entities.default-encoding-utf8`](crate::xml_spec::rule_set::rule_entities_default_encoding_utf8) — UTF-8 is the default when no encoding is declared.

use std::fmt;

use crate::error::{XmlError, XmlResult};

/// The version string this library supports.
pub const XML_VERSION: &str = "1.0";

/// The only encoding this library supports.
pub const UTF8_ENCODING_NAME: &str = "UTF-8";

/// The XML declaration of a document.
///
/// The declaration is a property of the document rather than a node of its tree. A value of this
/// type is always one this library can write and read back: the version is
/// [`XML_VERSION`] and the encoding, if present, names UTF-8.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct XmlDeclaration {
    version: String,
    encoding: Option<String>,
    standalone: Option<bool>,
}

impl XmlDeclaration {
    /// Creates a declaration from its parts.
    ///
    /// The encoding is kept with the spelling it was given (`utf-8` stays `utf-8`), so that a
    /// parsed declaration is written back as it was read.
    ///
    /// # Errors
    ///
    /// - [`XmlError::UnsupportedXmlVersion`] if `version` is not [`XML_VERSION`].
    /// - [`XmlError::UnsupportedEncoding`] if `encoding` does not name UTF-8 (see
    ///   [`encoding_is_utf8`]).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use biodivine_lib_xml_dom::xml_spec::XmlDeclaration;
    ///
    /// let declaration = XmlDeclaration::new("1.0", Some("utf-8"), Some(true)).unwrap();
    /// assert_eq!(
    ///     declaration.to_string(),
    ///     r#"<?xml version="1.0" encoding="utf-8" standalone="yes"?>"#
    /// );
    /// assert!(XmlDeclaration::new("1.1", None::<&str>, None).is_err());
    /// assert!(XmlDeclaration::new("1.0", Some("UTF-16"), None).is_err());
    /// ```
    pub fn new(
        version: impl Into<String>,
        encoding: Option<impl Into<String>>,
        standalone: Option<bool>,
    ) -> XmlResult<Self> {
        let version = version.into();
        if version != XML_VERSION {
            return Err(XmlError::UnsupportedXmlVersion(version));
        }
        let encoding = encoding.map(Into::into);
        if let Some(encoding) = &encoding
            && !encoding_is_utf8(encoding)
        {
            // rule: rule.entities.encoding-must-match-declaration
            return Err(XmlError::UnsupportedEncoding(encoding.clone()));
        }
        Ok(Self {
            version,
            encoding,
            standalone,
        })
    }

    /// The declaration this crate writes by default: version 1.0, UTF-8, no `standalone`.
    pub fn utf8() -> Self {
        Self {
            version: XML_VERSION.to_string(),
            encoding: Some(UTF8_ENCODING_NAME.to_string()),
            standalone: None,
        }
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
    pub(crate) fn pseudo_attributes(&self) -> String {
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
/// [`rule.entities.encoding-name-case-insensitive`](crate::xml_spec::rule_set::rule_entities_encoding_name_case_insensitive): the encoding name is compared
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
            XmlDeclaration::new("1.0", None::<String>, None)
                .unwrap()
                .to_string(),
            "<?xml version=\"1.0\"?>"
        );
        assert_eq!(
            XmlDeclaration::new("1.0", Some("UTF-8"), Some(true))
                .unwrap()
                .to_string(),
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>"
        );
        assert_eq!(
            XmlDeclaration::new("1.0", Some("UTF-8"), Some(false))
                .unwrap()
                .to_string(),
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
            XmlDeclaration::new("1.0", None::<String>, Some(false))
                .unwrap()
                .pseudo_attributes(),
            "version=\"1.0\" standalone=\"no\""
        );
    }

    #[test]
    fn unsupported_values_are_rejected() {
        assert!(matches!(
            XmlDeclaration::new("1.0\" foo=\"", None::<String>, None),
            Err(XmlError::UnsupportedXmlVersion(_))
        ));
        assert!(matches!(
            XmlDeclaration::new("1.0", Some("UTF-8\" foo=\""), None),
            Err(XmlError::UnsupportedEncoding(_))
        ));
    }

    #[test]
    fn accessors() {
        let declaration =
            XmlDeclaration::new("1.0", Some("UTF-8"), Some(true)).unwrap();
        assert_eq!(declaration.version(), "1.0");
        assert_eq!(declaration.encoding(), Some("UTF-8"));
        assert_eq!(declaration.standalone(), Some(true));
        assert_eq!(XmlDeclaration::utf8().standalone(), None);
    }

    #[test]
    fn encoding_names_are_case_insensitive() {
        // rule: rule.entities.encoding-name-case-insensitive
        assert_rule_exists("rule.entities.encoding-name-case-insensitive");
        assert_rule_exists("rule.entities.encoding-must-match-declaration");
        assert_rule_exists("rule.entities.default-encoding-utf8");
        assert_rule_exists("rule.entities.no-encoding-legal-utf-required");

        assert!(encoding_is_utf8("UTF-8"));
        assert!(encoding_is_utf8("utf-8"));
        assert!(encoding_is_utf8("Utf8"));
        assert!(encoding_is_utf8("uTf-8"));
        assert!(!encoding_is_utf8("UTF-16"));
        assert!(!encoding_is_utf8("ISO-8859-1"));
        assert!(!encoding_is_utf8(""));
    }
}
