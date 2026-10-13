//! The XML parser behind the `Document::parse_*` methods.
//!
//! The public contract — what is accepted, what is rejected, what happens to content outside the
//! root element — is documented in the [crate documentation](crate#reading-and-writing-documents)
//! and on the individual entry points
//! ([`Document::parse_string`](crate::Document::parse_string) and friends). What follows is the
//! implementation-level view, which only matters when changing this file.
//!
//! # Layers
//!
//! The parser enforces every well-formedness rule that is decidable *locally*: while looking at one
//! element, one attribute or one string. Rules that need a whole-document view (namespace scope,
//! unique ids, `xml:lang` inheritance) are checked by whole-document validation instead, so that
//! two passes over a document never duplicate the same check.
//!
//! The parser never panics: every failure is a typed [`XmlError`].
//!
//! # Entities
//!
//! `DOCTYPE` declarations are read and ignored: this library does not process DTDs, so no entity,
//! attribute type or content model can be declared. Consequently
//!
//! * the five predefined entities `amp`, `lt`, `gt`, `apos`, `quot` are expanded — they must be
//!   recognised whether declared or not ([`rule.entities.predefined-entities-recognized`](crate::xml_spec::rule_set::rule_entities_predefined_entities_recognized));
//! * character references (`&#…;`, `&#x…;`) are expanded and the resulting character must be legal
//!   in XML ([`rule.entities.charref-legal-character`](crate::xml_spec::rule_set::rule_entities_charref_legal_character));
//! * any other general entity reference is an error
//!   ([`rule.attributes.no-undeclared-entity-refs`](crate::xml_spec::rule_set::rule_attributes_no_undeclared_entity_refs), [`rule.entities.entity-declared-wfc`](crate::xml_spec::rule_set::rule_entities_entity_declared_wfc)), because
//!   without DTD processing it can never be declared.
//!
//! # Document-level content policy
//!
//! The data model of this library is one root element plus the nodes below it, so content *outside*
//! the root element is handled as follows (the same table appears in the crate documentation, which
//! is the public home of this policy):
//!
//! | input outside the root | result |
//! | --- | --- |
//! | whitespace | ignored |
//! | a comment | **discarded** (the model has nowhere to put it) |
//! | a processing instruction | **discarded** (the model has nowhere to put it) |
//! | non-whitespace text | [`XmlError::ContentOutsideRoot`] |
//! | a second element | [`XmlError::MultipleRootElements`] |
//! | no element at all | [`XmlError::MissingRoot`] |
//!
//! # Encodings
//!
//! Only UTF-8 is supported. An optional UTF-8 byte-order mark is accepted
//! ([`rule.entities.utf8-bom-optional`](crate::xml_spec::rule_set::rule_entities_utf8_bom_optional)); the input must be valid UTF-8
//! ([`rule.entities.illegal-byte-sequence-fatal`](crate::xml_spec::rule_set::rule_entities_illegal_byte_sequence_fatal)); and if the XML declaration names an encoding, it
//! must name UTF-8 ([`rule.entities.encoding-must-match-declaration`](crate::xml_spec::rule_set::rule_entities_encoding_must_match_declaration)).

use quick_xml::Reader;
use quick_xml::XmlVersion;
use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::attributes::AttrError;
use quick_xml::events::{BytesDecl, BytesStart, Event};
use std::collections::{BTreeMap, HashMap};
use std::io::Read;
use std::path::Path;

use crate::Namespace;
use crate::cancellation::CancellationCheck;
use crate::document::Document;
use crate::element::Element;
use crate::error::{XmlError, XmlResult};
use crate::qualified_name::QualifiedName;
use crate::xml_spec::{self, NCName, XmlDeclaration};

/// The UTF-8 byte-order mark ([`rule.entities.utf8-bom-optional`](crate::xml_spec::rule_set::rule_entities_utf8_bom_optional)).
const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

/// Reads a file and parses it; the implementation of
/// [`Document::parse_file`](crate::Document::parse_file).
///
/// # Errors
///
/// Returns [`XmlError::Io`] if the file cannot be opened or read, plus every error of
/// [`parse_bytes`].
pub(crate) fn parse_file<P: AsRef<Path>>(path: P) -> XmlResult<Document> {
    // rule: rule.entities.bom-encoding-detection (the byte-order mark is handled by `parse_bytes`)
    let mut file = std::fs::File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    Document::parse_bytes(&bytes)
}

/// Parses a string; the implementation of
/// [`Document::parse_string`](crate::Document::parse_string).
///
/// # Errors
///
/// Returns every error of [`parse_bytes`].
pub(crate) fn parse_string(xml: &str) -> XmlResult<Document> {
    Document::parse_bytes(xml.as_bytes())
}

/// Parses a generic reader; the implementation of
/// [`Document::parse_reader`](crate::Document::parse_reader).
///
/// The reader is consumed completely: a DOM holds the whole document in memory anyway, so there is
/// nothing to gain from streaming, and reading the input as a whole keeps byte-order-mark handling,
/// UTF-8 validation and the XML-declaration position check simple and exact.
///
/// # Errors
///
/// Returns [`XmlError::Io`] if the reader fails, plus every error of [`parse_bytes`].
pub(crate) fn parse_reader<R: Read>(mut reader: R) -> XmlResult<Document> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    Document::parse_bytes(&bytes)
}

/// Parses raw bytes; the implementation behind every other entry point.
///
/// # Errors
///
/// Returns a typed [`XmlError`] for every way in which the input can fail to be a well-formed XML
/// document that this library supports:
///
/// - [`XmlError::InvalidUtf8`] if the input is not valid UTF-8
///   ([`rule.entities.illegal-byte-sequence-fatal`](crate::xml_spec::rule_set::rule_entities_illegal_byte_sequence_fatal));
/// - [`XmlError::UnsupportedEncoding`] / [`XmlError::UnsupportedXmlVersion`] if the XML declaration
///   names an encoding other than UTF-8 or a version other than 1.0;
/// - [`XmlError::MalformedXml`] for malformed markup (unbalanced, mismatched or unclosed tags,
///   `<` inside an attribute value);
/// - [`XmlError::MultipleRootElements`] / [`XmlError::ContentOutsideRoot`] /
///   [`XmlError::MissingRoot`] for the top-level content rules;
/// - [`XmlError::InvalidName`] / [`XmlError::InvalidNamespace`] / [`XmlError::ReservedPrefix`] /
///   [`XmlError::UndeclaredPrefix`] for name and namespace problems;
/// - [`XmlError::InvalidComment`] / [`XmlError::InvalidCData`] /
///   [`XmlError::InvalidProcessingInstruction`] for illegal nodes;
/// - [`XmlError::UndeclaredEntityReference`] / [`XmlError::InvalidCharacterReference`] for
///   references that cannot be expanded;
/// - [`XmlError::DuplicateAttribute`] when one element specifies the same attribute twice;
/// - [`XmlError::Cancelled`] if a cancellation trigger registered on this thread fires.
///
/// # Panics
///
/// Panics only if the document has more than [`MAX_NODES`](crate::MAX_NODES) nodes.
pub(crate) fn parse_bytes(bytes: &[u8]) -> XmlResult<Document> {
    let bytes = match bytes.strip_prefix(UTF8_BOM) {
        Some(stripped) => stripped,
        None => bytes,
    };
    let input = std::str::from_utf8(bytes).map_err(|error| {
        XmlError::InvalidUtf8(format!("the input is not valid UTF-8: {error}"))
    })?;
    Parser::new(input).run()
}

/// Maps a `quick-xml` reader error onto a typed [`XmlError`].
fn reader_error(error: quick_xml::Error) -> XmlError {
    XmlError::MalformedXml(format!("{error}"))
}

/// Maps a `quick-xml` attribute error onto a typed [`XmlError`].
///
/// Duplicate attribute names get their own variant, because that is the one attribute error the
/// XML specification names explicitly
/// ([`rule.elements-and-tags.unique-attribute-specification`](crate::xml_spec::rule_set::rule_elements_and_tags_unique_attribute_specification)); everything else is malformed markup.
fn attribute_error(error: AttrError) -> XmlError {
    match error {
        AttrError::Duplicated(first, second) => XmlError::DuplicateAttribute(format!(
            "the attribute at byte {first} repeats the one at byte {second}"
        )),
        other => XmlError::MalformedXml(format!("invalid attribute: {other}")),
    }
}

/// Maps any `quick-xml` error onto a typed [`XmlError`], keeping the escaping classification.
fn quick_error(error: quick_xml::Error) -> XmlError {
    match error {
        quick_xml::Error::Escape(inner) => escape_error(inner),
        other => XmlError::MalformedXml(format!("{other}")),
    }
}

/// Maps a `quick-xml` escaping error onto a typed [`XmlError`].
fn escape_error(error: quick_xml::escape::EscapeError) -> XmlError {
    use quick_xml::escape::EscapeError;
    match error {
        EscapeError::UnrecognizedEntity(_, name) => {
            XmlError::UndeclaredEntityReference(name)
        }
        EscapeError::InvalidCharRef(inner) => {
            XmlError::InvalidCharacterReference(inner.to_string())
        }
        EscapeError::UnterminatedEntity(_) => XmlError::MalformedXml(
            "an entity reference is not terminated by `;`".to_string(),
        ),
        EscapeError::TooManyNestedEntities => {
            XmlError::MalformedXml("too many nested entity references".to_string())
        }
    }
}

/// The parser's state.
struct Parser<'input> {
    /// The underlying tokenizer.
    reader: Reader<&'input [u8]>,
    /// The document being built.
    document: Document,
    /// The elements that are currently open, outermost first.
    stack: Vec<Element>,
    /// The namespace bindings in scope, outermost first; the last entry belongs to
    /// `stack.last()`.
    scopes: Vec<HashMap<Option<NCName>, String>>,
    /// Text that has been read but not yet stored as a node.
    ///
    /// Text arrives in pieces (a `&amp;` splits it into three events), so the pieces are
    /// accumulated here and stored as a single text node, which also makes the output independent
    /// of how the tokenizer chunked the input.
    pending_text: String,
    /// Whether the root element has already been seen.
    root_seen: bool,
    /// Whether the current event is the first one (the only place an XML declaration may appear).
    first_event: bool,
}

impl<'input> Parser<'input> {
    /// Creates a parser that builds its tree into a fresh document.
    fn new(input: &'input str) -> Self {
        Self::with_document(input, Document::new())
    }

    /// Creates a parser that builds its tree into `document`.
    ///
    /// The document's capacity policy applies while parsing, so a document with a small arena
    /// collects garbage during the parse. That is safe because every node the parser still needs is
    /// externally referenced: the elements that are still open are held on the parser's stack, and
    /// the root element is installed in the document as soon as its start tag is seen.
    fn with_document(input: &'input str, document: Document) -> Self {
        let mut reader = Reader::from_str(input);
        // rule: rule.well-formedness.comment-no-double-hyphen
        reader.config_mut().check_comments = true;
        // rule: rule.elements-and-tags.end-tag-must-match-start-tag
        reader.config_mut().check_end_names = true;
        Self {
            reader,
            document,
            stack: Vec::new(),
            scopes: vec![HashMap::new()],
            pending_text: String::new(),
            root_seen: false,
            first_event: true,
        }
    }

    /// Parses the input.
    fn run(mut self) -> XmlResult<Document> {
        // A long document can take a while to parse, so honour any cancellation trigger that the
        // caller registered on this thread.
        let mut check = CancellationCheck::new();
        loop {
            check.tick()?;
            let event = self.reader.read_event().map_err(reader_error)?;
            let is_eof = matches!(event, Event::Eof);
            self.handle(event)?;
            if is_eof {
                break;
            }
            self.first_event = false;
        }

        self.flush_text()?;
        if let Some(element) = self.stack.last() {
            // rule: rule.elements-and-tags.every-start-tag-must-have-end-tag
            return Err(XmlError::MalformedXml(format!(
                "element `<{}>` is never closed",
                element.local_name()
            )));
        }
        if !self.root_seen {
            // rule: rule.well-formedness.document-production
            return Err(XmlError::MissingRoot);
        }
        Ok(self.document)
    }

    /// Handles one tokenizer event.
    fn handle(&mut self, event: Event<'input>) -> XmlResult<()> {
        match event {
            Event::Decl(declaration) => self.handle_declaration(&declaration),
            Event::Start(ref start) => {
                self.flush_text()?;
                self.open_element(start, false)
            }
            Event::Empty(ref start) => {
                self.flush_text()?;
                self.open_element(start, true)
            }
            Event::End(_) => {
                self.flush_text()?;
                self.close_element()
            }
            Event::Text(text) => {
                let content = text.xml10_content().map_err(|error| {
                    XmlError::InvalidUtf8(format!("invalid text content: {error}"))
                })?;
                self.push_character_data(&content)
            }
            Event::GeneralRef(reference) => {
                let character = self.resolve_reference(&reference)?;
                let mut buffer = [0u8; 4];
                self.push_character_data(character.encode_utf8(&mut buffer))
            }
            Event::CData(cdata) => {
                self.flush_text()?;
                self.require_inside_root()?;
                let content = decode_bytes(&cdata, "CDATA section")?;
                let element = self.stack.last().expect("checked above");
                let node = self.document.create_cdata(content)?;
                element.append_child_checked(node)
            }
            Event::Comment(comment) => {
                self.flush_text()?;
                if self.stack.is_empty() {
                    // Documented policy (see the module documentation): the data model has no place
                    // for top-level comments, so they are discarded rather than rejected.
                    return Ok(());
                }
                let content = decode_bytes(&comment, "comment")?;
                let element = self.stack.last().expect("checked above");
                let node = self.document.create_comment(content)?;
                element.append_child_checked(node)
            }
            Event::PI(instruction) => {
                self.flush_text()?;
                if self.stack.is_empty() {
                    // Same policy as for comments: discarded, see the module documentation.
                    return Ok(());
                }
                let target = decode_bytes(
                    instruction.target(),
                    "processing instruction target",
                )?;
                // `PI ::= '<?' PITarget (S (Char* - (Char* '?>' Char*)))? '?>'`: the whitespace
                // that separates the target from the content is syntax, so exactly one character
                // of it is removed. Everything else - including leading and trailing whitespace of
                // the content itself - is data and is kept verbatim, which is what makes the
                // content round-trip exactly (`rule.well-formedness.pi-pass-through`).
                let content = decode_bytes(
                    instruction.content(),
                    "processing instruction content",
                )?;
                let content = content
                    .strip_prefix([' ', '\t', '\n', '\r'])
                    .unwrap_or(content);
                let element = self.stack.last().expect("checked above");
                let node = self
                    .document
                    .create_processing_instruction(target, content)?;
                element.append_child_checked(node)
            }
            // `DOCTYPE` declarations are read and ignored: this library does not process DTDs, so
            // there is nothing to validate and nothing to record.
            Event::DocType(_) => Ok(()),
            Event::Eof => Ok(()),
        }
    }

    /// Handles `<?xml …?>`.
    ///
    /// `quick-xml` reports any `<?xml …?>` as a declaration, wherever it appears, so the position
    /// has to be checked here: the XML declaration may only be the very first thing in a document
    /// (`XMLDecl ::= '<?xml' …`, XML 1.0 §2.8). Anywhere else the same text would be a processing
    /// instruction whose target is `xml`, which the specification forbids
    /// ([`rule.well-formedness.pi-target-not-xml`](crate::xml_spec::rule_set::rule_well_formedness_pi_target_not_xml)).
    fn handle_declaration(&mut self, declaration: &BytesDecl<'_>) -> XmlResult<()> {
        if !self.first_event {
            // rule: rule.well-formedness.pi-target-not-xml
            return Err(XmlError::InvalidProcessingInstruction(
                "`xml` is a reserved processing-instruction target, and the XML declaration may \
                 only appear at the very beginning of a document"
                    .to_string(),
            ));
        }

        let version = declaration.version().map_err(|error| {
            XmlError::MalformedXml(format!("invalid XML version: {error}"))
        })?;
        let version = std::str::from_utf8(&version).map_err(|error| {
            XmlError::InvalidUtf8(format!("invalid XML version: {error}"))
        })?;
        if version != xml_spec::XML_VERSION {
            return Err(XmlError::UnsupportedXmlVersion(version.to_string()));
        }

        let encoding = match declaration.encoding() {
            None => None,
            Some(result) => {
                let encoding = result.map_err(attribute_error)?;
                let encoding = std::str::from_utf8(&encoding).map_err(|error| {
                    XmlError::InvalidUtf8(format!("invalid encoding name: {error}"))
                })?;
                if !xml_spec::encoding_is_utf8(encoding) {
                    // rule: rule.entities.encoding-must-match-declaration
                    return Err(XmlError::UnsupportedEncoding(encoding.to_string()));
                }
                Some(encoding.to_string())
            }
        };

        let standalone = match declaration.standalone() {
            None => None,
            Some(result) => {
                let value = result.map_err(attribute_error)?;
                let value = std::str::from_utf8(&value).map_err(|error| {
                    XmlError::InvalidUtf8(format!("invalid standalone value: {error}"))
                })?;
                match value {
                    "yes" => Some(true),
                    "no" => Some(false),
                    other => {
                        return Err(XmlError::MalformedXml(format!(
                            "the `standalone` pseudo-attribute must be `yes` or `no`, not `{other}`"
                        )));
                    }
                }
            }
        };

        // The version and the encoding were checked above, with the more specific errors.
        self.document.set_xml_declaration(Some(XmlDeclaration::new(
            version, encoding, standalone,
        )?));
        Ok(())
    }

    /// Opens an element, either into the stack (a start tag) or as a leaf (an empty-element tag).
    fn open_element(&mut self, start: &BytesStart<'_>, empty: bool) -> XmlResult<()> {
        if self.stack.is_empty() {
            if self.root_seen {
                // rule: rule.well-formedness.single-root-element
                return Err(XmlError::MultipleRootElements);
            }
            self.root_seen = true;
        }

        let scope = self.extended_scope(start)?;
        let name = decode_bytes(start.name().into_inner(), "element name")?;
        let qualified_name =
            QualifiedName::resolve_element_with_namespace_map(name, &scope)?;
        let element = self.document.create_element(qualified_name);

        let declarations = Self::namespace_declarations(start)?;
        for (prefix, uri) in declarations {
            match prefix {
                Some(prefix) => {
                    element.declare_namespace(Namespace::prefixed(&uri, &prefix)?)
                }
                None if uri.is_empty() => element.undeclare_default_namespace(),
                None => element.declare_namespace(Namespace::without_prefix(&uri)?),
            }
        }

        let mut attributes: BTreeMap<QualifiedName, String> = BTreeMap::new();
        for attribute in start.attributes() {
            let attribute = attribute.map_err(attribute_error)?;
            let key = decode_bytes(attribute.key.into_inner(), "attribute name")?;
            if attribute.value.contains(&b'<') {
                // rule: rule.elements-and-tags.no-less-than-in-attribute-values
                return Err(XmlError::MalformedXml(format!(
                    "the value of attribute `{key}` contains a `<`"
                )));
            }
            if key == "xmlns" || key.starts_with("xmlns:") {
                continue;
            }
            // rule: rule.attributes.values-must-be-normalized
            // rule: rule.attributes.entity-refs-expanded
            // rule: rule.attributes.char-refs-expanded
            let value = attribute
                .normalized_value_with(
                    XmlVersion::Explicit1_0,
                    1,
                    resolve_predefined_entity,
                )
                .map_err(quick_error)?;
            let name =
                QualifiedName::resolve_attribute_with_namespace_map(key, &scope)?;
            if attributes.contains_key(&name) {
                // rule: rule.namespace-usage.attributes-unique-expanded-name
                return Err(XmlError::DuplicateAttribute(name.to_string()));
            }
            attributes.insert(name, value.to_string());
        }
        for (name, value) in attributes {
            element.set_attribute_checked(name, value)?;
        }

        if let Some(parent) = self.stack.last() {
            parent.append_child_checked(element.clone())?;
        } else {
            self.document.set_root_checked(element.clone())?;
        }

        if !empty {
            self.scopes.push(scope);
            self.stack.push(element);
        }
        Ok(())
    }

    /// Closes the innermost element.
    fn close_element(&mut self) -> XmlResult<()> {
        if self.stack.pop().is_none() {
            return Err(XmlError::MalformedXml(
                "an end tag appears without a matching start tag".to_string(),
            ));
        }
        self.scopes.pop();
        Ok(())
    }

    /// Resolves a general entity or character reference to the character it denotes.
    fn resolve_reference(
        &mut self,
        reference: &quick_xml::events::BytesRef<'_>,
    ) -> XmlResult<char> {
        if reference.is_char_ref() {
            let character = reference.resolve_char_ref().map_err(quick_error)?;
            let Some(character) = character else {
                return Err(XmlError::InvalidCharacterReference(
                    decode_bytes(reference, "character reference")?.to_string(),
                ));
            };
            if !xml_spec::is_legal_char(character) {
                // rule: rule.entities.charref-legal-character
                return Err(XmlError::InvalidCharacterReference(format!(
                    "`&#{};` denotes a character that is not legal in XML",
                    decode_bytes(reference, "character reference")?
                )));
            }
            Ok(character)
        } else {
            let name = decode_bytes(reference, "entity reference")?;
            let replacement = resolve_predefined_entity(name).ok_or_else(|| {
                // rule: rule.attributes.no-undeclared-entity-refs
                XmlError::UndeclaredEntityReference(name.to_string())
            })?;
            // The predefined entities all resolve to a single character.
            Ok(replacement.chars().next().expect("never empty"))
        }
    }

    /// Appends character data, remembering whether it may appear outside the root element.
    fn push_character_data(&mut self, content: &str) -> XmlResult<()> {
        if self.stack.is_empty() {
            // rule: rule.well-formedness.document-production
            if content.chars().all(crate::xml_spec::is_space) {
                return Ok(());
            }
            return Err(XmlError::ContentOutsideRoot);
        }
        self.pending_text.push_str(content);
        Ok(())
    }

    /// Stores the accumulated text as a single text node.
    fn flush_text(&mut self) -> XmlResult<()> {
        if self.pending_text.is_empty() {
            return Ok(());
        }
        let text = std::mem::take(&mut self.pending_text);
        let element = self
            .stack
            .last()
            .expect("text is only accumulated inside an element");
        let node = self.document.create_text(text)?;
        element.append_child_checked(node)
    }

    /// Fails unless the parser is currently inside the root element.
    fn require_inside_root(&self) -> XmlResult<()> {
        if self.stack.is_empty() {
            // rule: rule.well-formedness.document-production
            return Err(XmlError::ContentOutsideRoot);
        }
        Ok(())
    }

    /// Returns the namespace scope that applies to `start`: the enclosing scope plus the element's
    /// own declarations.
    fn extended_scope(
        &self,
        start: &BytesStart<'_>,
    ) -> XmlResult<HashMap<Option<NCName>, String>> {
        let mut scope = self
            .scopes
            .last()
            .expect("root scope always exists")
            .clone();
        for (prefix, uri) in Self::namespace_declarations(start)? {
            if prefix.is_none() && uri.is_empty() {
                // rule: rule.namespace-usage.empty-default-namespace
                scope.remove(&None);
            } else {
                // rule: rule.namespace-usage.prefix-declaration-scope
                scope.insert(prefix, uri);
            }
        }
        Ok(scope)
    }

    /// Extracts the namespace declarations written on a start tag.
    ///
    /// # Errors
    ///
    /// Returns [`XmlError::InvalidNamespace`] if a prefix is undeclared with an empty value, which
    /// the Namespaces specification forbids ([`rule.namespace-usage.no-prefix-undeclaring`](crate::xml_spec::rule_set::rule_namespace_usage_no_prefix_undeclaring)), and the
    /// errors of [`NCName::try_from`] if a prefix is not a valid `NCName`. Duplicate declarations
    /// are already rejected by the attribute iterator
    /// ([`rule.namespace-usage.attributes-unique-expanded-name`](crate::xml_spec::rule_set::rule_namespace_usage_attributes_unique_expanded_name)).
    fn namespace_declarations(
        start: &BytesStart<'_>,
    ) -> XmlResult<Vec<(Option<NCName>, String)>> {
        let mut declarations = Vec::new();
        for attribute in start.attributes() {
            let attribute = attribute.map_err(attribute_error)?;
            let key = decode_bytes(attribute.key.into_inner(), "attribute name")?;
            let prefix = if let Some(prefix) = key.strip_prefix("xmlns:") {
                if attribute.value.is_empty() {
                    // rule: rule.namespace-usage.no-prefix-undeclaring
                    return Err(XmlError::InvalidNamespace(format!(
                        "the namespace prefix `{prefix}` may not be undeclared with an empty value"
                    )));
                }
                Some(NCName::try_from(prefix)?)
            } else if key == "xmlns" {
                None
            } else {
                continue;
            };
            let value = attribute
                .normalized_value_with(
                    XmlVersion::Explicit1_0,
                    1,
                    resolve_predefined_entity,
                )
                .map_err(quick_error)?;
            declarations.push((prefix, value.to_string()));
        }
        Ok(declarations)
    }
}

/// Builds the typed error for input that is not valid UTF-8.
///
/// Marked cold because it only runs on the error path in practice.
#[cold]
fn utf8_error(what: &str, error: std::str::Utf8Error) -> XmlError {
    XmlError::InvalidUtf8(format!("invalid UTF-8 in {what}: {error}"))
}

/// Decodes a byte slice as UTF-8 with a typed [`XmlError::InvalidUtf8`].
fn decode_bytes<'a>(bytes: &'a [u8], what: &str) -> XmlResult<&'a str> {
    std::str::from_utf8(bytes).map_err(|error| utf8_error(what, error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::GcConfig;

    /// Builds `<root>` with `count` children, each carrying an attribute and a text node.
    fn sample(count: usize) -> String {
        let mut xml = String::from("<root>");
        for index in 0..count {
            xml.push_str(&format!("<child n=\"{index}\">text {index}</child>"));
        }
        xml.push_str("</root>");
        xml
    }

    /// Parses `input` into a document whose arena fills up immediately, so that the parser triggers
    /// a collection before almost every allocation.
    fn parse_with_tiny_arena(input: &str) -> Document {
        let document = Document::with_gc_config(
            GcConfig::new()
                .with_initial_capacity(2)
                .with_minimum_free_fraction(0.25),
        );
        let parsed = Parser::with_document(input, document.clone())
            .run()
            .expect("the sample is well formed");
        assert!(parsed.ptr_eq(&document));
        assert!(
            document.capacity() > 2,
            "the tiny arena should have collected and grown while parsing"
        );
        document
    }

    /// The parser keeps every node it still needs externally referenced: open elements are held on
    /// its stack, and the root is installed in the document as soon as its start tag is read. A
    /// collection in the middle of a parse therefore cannot lose a node.
    #[test]
    fn a_collection_during_parsing_keeps_the_tree_intact() {
        let xml = sample(40);
        let reference = Document::parse_string(&xml)
            .expect("parses with the default configuration");
        let tiny = parse_with_tiny_arena(&xml);

        assert_eq!(tiny.node_count(), reference.node_count());
        assert_eq!(
            tiny.write_string().expect("serializes"),
            reference.write_string().expect("serializes")
        );
        let root = tiny.root().expect("the root survived");
        assert_eq!(root.children().len(), 40);
        assert_eq!(
            root.children()[39]
                .as_element()
                .expect("a child element")
                .local_name(),
            "child"
        );
        assert!(tiny.validate().is_ok());
    }

    /// The error path must not be disturbed by a collection either: an unclosed element is still
    /// reported, and the document it was parsed into is still consistent.
    #[test]
    fn a_collection_does_not_hide_a_parse_error() {
        let mut unclosed = sample(20);
        unclosed.truncate(unclosed.len() - "</root>".len());

        let document =
            Document::with_gc_config(GcConfig::new().with_initial_capacity(2));
        let parsed = Parser::with_document(&unclosed, document.clone()).run();
        assert!(matches!(parsed, Err(XmlError::MalformedXml(_))));
        assert!(
            document.capacity() > 2,
            "the tiny arena should have collected while parsing the failing input"
        );

        // A mismatched end tag is reported as well.
        let parsed =
            Parser::with_document("<root><child></root>", Document::new()).run();
        assert!(matches!(parsed, Err(XmlError::MalformedXml(_))));
    }
}
