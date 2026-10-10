//! Parsing and serialization: the public entry points, implemented by the parser and the
//! serializer that back them.
//!
//! Everything here is a `Document` method (or, for parsing, an associated function) rather than a
//! free function, so that the whole lifetime of a document — create, parse, edit, validate,
//! serialize — is discoverable from one place. The conceptual contract these methods implement is
//! documented on [`Document`] itself; this module contains the methods and their
//! error conditions.

use std::io::{BufRead, Write};
use std::path::Path;
use std::str::FromStr;

use crate::document::Document;
use crate::error::{XmlError, XmlResult};
use crate::io::WriteOptions;

impl Document {
    /// Parses an XML document from a string.
    ///
    /// This is the usual way to create a document from existing XML. The input must be a
    /// well-formed XML 1.0 document with exactly one root element; content outside the root element
    /// is handled as described in the [parsing contract](Document#parsing).
    ///
    /// # Errors
    ///
    /// Returns a typed [`XmlError`] for every way in which the input can fail to be a well-formed
    /// XML document that this library supports:
    ///
    /// - [`XmlError::InvalidUtf8`] if the input is not valid UTF-8
    ///   ([`rule.entities.illegal-byte-sequence-fatal`](crate::xml_spec::rule_set::rule_entities_illegal_byte_sequence_fatal));
    /// - [`XmlError::UnsupportedEncoding`] / [`XmlError::UnsupportedXmlVersion`] if the XML
    ///   declaration names an encoding other than UTF-8 or a version other than 1.0;
    /// - [`XmlError::MalformedXml`] for malformed markup (unbalanced, mismatched or unclosed tags,
    ///   duplicate attributes, `<` inside an attribute value, an empty document);
    /// - [`XmlError::MultipleRootElements`] / [`XmlError::ContentOutsideRoot`] /
    ///   [`XmlError::MissingRoot`] for the top-level content rules;
    /// - [`XmlError::InvalidName`] / [`XmlError::InvalidNamespace`] / [`XmlError::ReservedPrefix`] /
    ///   [`XmlError::UndeclaredPrefix`] for name and namespace problems;
    /// - [`XmlError::InvalidComment`] / [`XmlError::InvalidCData`] /
    ///   [`XmlError::InvalidProcessingInstruction`] for illegal nodes;
    /// - [`XmlError::UndeclaredEntityReference`] / [`XmlError::InvalidCharacterReference`] for
    ///   references that cannot be expanded;
    /// - [`XmlError::DuplicateAttribute`] when one element specifies the same attribute twice.
    ///
    /// The function never panics.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use biodivine_lib_xml_dom::Document;
    ///
    /// let document = Document::parse_string("<root><child>text</child></root>").unwrap();
    /// assert_eq!(document.root().unwrap().children().len(), 1);
    /// ```
    pub fn parse_string(xml: &str) -> XmlResult<Document> {
        crate::io::parse_string(xml)
    }

    /// Parses an XML document from raw bytes.
    ///
    /// The bytes must be UTF-8; an optional UTF-8 byte-order mark is accepted and ignored
    /// ([`rule.entities.utf8-bom-optional`](crate::xml_spec::rule_set::rule_entities_utf8_bom_optional)).
    ///
    /// # Errors
    ///
    /// As [`Document::parse_string`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use biodivine_lib_xml_dom::Document;
    ///
    /// let document = Document::parse_bytes(b"<root/>").unwrap();
    /// assert_eq!(document.root().unwrap().local_name(), "root");
    /// ```
    pub fn parse_bytes(bytes: &[u8]) -> XmlResult<Document> {
        crate::io::parse_bytes(bytes)
    }

    /// Parses an XML document from a reader.
    ///
    /// The reader is consumed completely: a document object model holds the whole tree in memory
    /// anyway, so there is nothing to gain from streaming, and reading the input as a whole keeps
    /// byte-order-mark handling, UTF-8 validation and the position check of the XML declaration
    /// exact.
    ///
    /// # Errors
    ///
    /// Returns [`XmlError::Io`] if the reader fails, plus every error of
    /// [`Document::parse_string`].
    pub fn parse_reader<R: BufRead>(reader: R) -> XmlResult<Document> {
        crate::io::parse_reader(reader)
    }

    /// Parses an XML document from a file.
    ///
    /// # Errors
    ///
    /// Returns [`XmlError::Io`] if the file cannot be opened or read, plus every error of
    /// [`Document::parse_bytes`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use biodivine_lib_xml_dom::Document;
    ///
    /// let document = Document::parse_file("document.xml").unwrap();
    /// println!("{}", document.root().is_some());
    /// ```
    pub fn parse_file<P: AsRef<Path>>(path: P) -> XmlResult<Document> {
        crate::io::parse_file(path)
    }

    /// Serializes this document into a string, using the default options.
    ///
    /// A document without a root element and without an XML declaration serializes to the empty
    /// string. The output round-trips: parsing it produces an equal tree.
    ///
    /// # Errors
    ///
    /// Returns [`XmlError::InvalidUtf8`] if the serialized output is not valid UTF-8. This cannot
    /// happen for a document built through this API (every payload is valid UTF-8 by construction),
    /// so the variant is reported rather than panicking.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use biodivine_lib_xml_dom::Document;
    ///
    /// let document = Document::parse_string("<root/>").unwrap();
    /// assert_eq!(document.write_string().unwrap(), "<root/>");
    /// ```
    pub fn write_string(&self) -> XmlResult<String> {
        crate::io::write_string(self)
    }

    /// Serializes this document into a string with explicit options.
    ///
    /// See [`WriteOptions`] for the declaration and empty-element styles.
    ///
    /// # Errors
    ///
    /// As [`Document::write_string`].
    pub fn write_string_with(&self, options: &WriteOptions) -> XmlResult<String> {
        crate::io::write_string_with(self, options)
    }

    /// Writes this document to a file, using the default options.
    ///
    /// The file is created (or truncated) and flushed before this method returns.
    ///
    /// # Errors
    ///
    /// Returns [`XmlError::Io`] if the file cannot be created or written, plus the errors of
    /// [`Document::write_string`].
    pub fn write_file<P: AsRef<Path>>(&self, path: P) -> XmlResult<()> {
        crate::io::write_file(self, path)
    }

    /// Writes this document to a file with explicit options.
    ///
    /// # Errors
    ///
    /// As [`Document::write_file`].
    pub fn write_file_with<P: AsRef<Path>>(
        &self,
        path: P,
        options: &WriteOptions,
    ) -> XmlResult<()> {
        crate::io::write_file_with(self, path, options)
    }

    /// Writes this document to a writer, using the default options.
    ///
    /// # Errors
    ///
    /// Returns [`XmlError::Io`] if the underlying writer fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use biodivine_lib_xml_dom::Document;
    ///
    /// let document = Document::parse_string("<root/>").unwrap();
    /// let mut buffer = Vec::new();
    /// document.write_writer(&mut buffer).unwrap();
    /// assert_eq!(String::from_utf8(buffer).unwrap(), "<root/>");
    /// ```
    pub fn write_writer<W: Write>(&self, writer: W) -> XmlResult<()> {
        crate::io::write_writer(self, writer)
    }

    /// Writes this document to a writer with explicit options.
    ///
    /// # Errors
    ///
    /// As [`Document::write_writer`].
    pub fn write_writer_with<W: Write>(&self, writer: W, options: &WriteOptions) -> XmlResult<()> {
        crate::io::write_writer_with(self, writer, options)
    }
}

impl FromStr for Document {
    type Err = XmlError;

    /// Parses an XML document from a string; the interop spelling of [`Document::parse_string`].
    ///
    /// ```rust
    /// use biodivine_lib_xml_dom::Document;
    ///
    /// let document: Document = "<root/>".parse().unwrap();
    /// assert_eq!(document.root().unwrap().local_name(), "root");
    /// ```
    fn from_str(xml: &str) -> XmlResult<Self> {
        Self::parse_string(xml)
    }
}
