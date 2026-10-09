//! Reading and writing XML documents.
//!
//! # Status
//!
//! The parser and serializer in this module are being rewritten in the next step (goal G3). The
//! current implementation is the pre-rewrite algorithm ported onto the arena API, so that the
//! crate keeps compiling and its round-trip tests keep running. The known defects of that
//! algorithm are listed in `docs/design/REVIEW.md` §2 (D1, D2, D8, D9) and remain open until G3:
//!
//! * D1 — a general entity reference aborts the process (`unimplemented!`); predefined entities
//!   are not expanded;
//! * D2 — element prefixes are dropped on output and missing namespace declarations are never
//!   reported;
//! * D8 — several locally decidable well-formedness rules are not checked by this crate;
//! * D9 — no XML declaration on output, no empty-element style, no write options.

use quick_xml::Writer;
use quick_xml::events::{BytesCData, BytesEnd, BytesPI, BytesStart, BytesText, Event};
use quick_xml::{Reader, XmlVersion};
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use crate::Namespace;
use crate::QualifiedName;
use crate::document::Document;
use crate::element::Element;
use crate::error::{XmlError, XmlResult};
use crate::node::NodeContent;
use crate::xml_spec::NCName;

/// Parses an XML document from a file.
///
/// # Errors
///
/// Returns [`XmlError::Io`] if the file cannot be opened or read, and the same errors as
/// [`parse_reader`] if its content is not a well-formed XML document.
pub fn parse_file<P: AsRef<Path>>(path: P) -> XmlResult<Document> {
    let file = File::open(path)?;
    parse_reader(BufReader::new(file))
}

/// Parses an XML document from a string.
///
/// # Errors
///
/// Returns the same errors as [`parse_reader`].
pub fn parse_string(xml: &str) -> XmlResult<Document> {
    parse_reader(BufReader::new(xml.as_bytes()))
}

/// Parses an XML document from a generic reader.
///
/// # Errors
///
/// Returns a typed [`XmlError`] if the input is not a well-formed XML document. The variants that
/// can be reported are:
///
/// - [`XmlError::MalformedXml`] — malformed markup, including unbalanced or mismatched tags and a
///   missing or duplicated root element;
/// - [`XmlError::InvalidUtf8`] — the input is not valid UTF-8;
/// - [`XmlError::InvalidName`] — an element, attribute or namespace prefix is not a valid `NCName`,
///   or a qualified name is malformed;
/// - [`XmlError::UndeclaredPrefix`] — a prefix is used but never declared in scope;
/// - [`XmlError::ReservedPrefix`] — the reserved `xml`/`xmlns` prefixes are used illegally;
/// - [`XmlError::InvalidNamespace`] — a namespace declaration is illegal, e.g. `xmlns:p=""`;
/// - [`XmlError::DuplicateAttribute`] — the same attribute (by name or by expanded name) appears
///   twice on one element;
/// - [`XmlError::InvalidComment`], [`XmlError::InvalidCData`],
///   [`XmlError::InvalidProcessingInstruction`] — a comment, CDATA section or processing
///   instruction is illegal;
/// - [`XmlError::UndeclaredEntityReference`], [`XmlError::InvalidCharacterReference`] — an entity
///   or character reference could not be expanded (see the module documentation for the entity
///   policy; this is not implemented yet, see D1 in `docs/design/REVIEW.md`).
///
/// [`XmlError::Io`] is reported only by [`parse_file`], which opens the reader for this function.
pub fn parse_reader<R: BufRead>(reader: R) -> XmlResult<Document> {
    let mut xml_reader = Reader::from_reader(reader);

    let document = Document::empty();
    let mut stack: Vec<Element> = Vec::new();
    let mut ns_stack: Vec<HashMap<Option<NCName>, String>> = vec![HashMap::new()];
    let mut buffer = Vec::new();

    loop {
        match xml_reader.read_event_into(&mut buffer) {
            Ok(Event::Start(ref event)) => {
                let ns_map = extend_namespace_map(ns_stack.last().expect("root scope"), event)?;
                ns_stack.push(ns_map.clone());
                let element = parse_element(&document, event, &ns_map)?;
                match stack.last() {
                    Some(parent) => parent.append_child_checked(element.clone())?,
                    None => {
                        document.set_root_checked(element.clone())?;
                    }
                }
                stack.push(element);
            }
            Ok(Event::End(_)) => {
                stack.pop();
                ns_stack.pop();
            }
            Ok(Event::Empty(ref event)) => {
                let ns_map = extend_namespace_map(ns_stack.last().expect("root scope"), event)?;
                let element = parse_element(&document, event, &ns_map)?;
                match stack.last() {
                    Some(parent) => parent.append_child_checked(element)?,
                    None => {
                        document.set_root_checked(element)?;
                    }
                }
            }
            Ok(Event::Text(event)) => {
                if let Some(current) = stack.last() {
                    let text = event.xml10_content().map_err(|error| {
                        XmlError::MalformedXml(format!("invalid text content: {error}"))
                    })?;
                    append_text(current, text.as_ref())?;
                }
            }
            Ok(Event::Comment(event)) => {
                if let Some(current) = stack.last() {
                    let comment = decode(&event, "comment")?;
                    current.append_child_checked(document.create_comment(comment)?)?;
                }
            }
            Ok(Event::CData(event)) => {
                if let Some(current) = stack.last() {
                    let cdata = decode(&event, "CDATA section")?;
                    current.append_child_checked(document.create_cdata(cdata)?)?;
                }
            }
            Ok(Event::PI(event)) => {
                if let Some(current) = stack.last() {
                    let target = decode(event.target(), "processing instruction target")?;
                    let data = std::str::from_utf8(event.content())
                        .map_err(|error| {
                            XmlError::InvalidUtf8(format!(
                                "invalid UTF-8 in processing instruction content: {error}"
                            ))
                        })?
                        .trim();
                    current.append_child_checked(
                        document.create_processing_instruction(target, data)?,
                    )?;
                }
            }
            Ok(Event::Eof) => break,
            // The XML declaration is not interpreted yet; see D8/D9.
            Ok(Event::Decl(_)) => {}
            // The document type declaration is read and ignored: this crate does not process DTDs.
            Ok(Event::DocType(_)) => {}
            Ok(Event::GeneralRef(_)) => {
                // Known defect D1: entities are not expanded yet.
                unimplemented!("Custom entities are currently not supported.")
            }
            Err(error) => {
                return Err(XmlError::MalformedXml(format!("{error}")));
            }
        }
        buffer.clear();
    }

    Ok(document)
}

/// Maps a `quick-xml` attribute error onto a typed [`XmlError`].
///
/// Duplicate attribute names get their own variant, because that is the one attribute error the
/// XML specification names explicitly
/// (`rule.elements-and-tags.unique-attribute-specification`); everything else is malformed markup.
fn attribute_error(error: quick_xml::events::attributes::AttrError) -> XmlError {
    match error {
        quick_xml::events::attributes::AttrError::Duplicated(first, second) => {
            XmlError::DuplicateAttribute(format!(
                "duplicate attribute at byte {first} (first declared at byte {second})"
            ))
        }
        other => XmlError::MalformedXml(format!("invalid attribute: {other}")),
    }
}

/// Decodes a byte slice as UTF-8 with a typed error.
fn decode<'a>(bytes: &'a [u8], what: &str) -> XmlResult<&'a str> {
    std::str::from_utf8(bytes)
        .map_err(|error| XmlError::InvalidUtf8(format!("invalid UTF-8 in {what}: {error}")))
}

/// Clones the current namespace scope and applies the declarations of `event`.
fn extend_namespace_map(
    parent: &HashMap<Option<NCName>, String>,
    event: &BytesStart,
) -> XmlResult<HashMap<Option<NCName>, String>> {
    let mut ns_map = parent.clone();
    for (prefix, uri) in extract_namespace_declarations(event)? {
        if prefix.is_none() && uri.is_empty() {
            // `xmlns=""` removes the default namespace from scope.
            ns_map.remove(&None);
        } else {
            ns_map.insert(prefix, uri);
        }
    }
    Ok(ns_map)
}

/// Appends a text node to `element`, merging it with the previous text node if there is one.
///
/// The merging is what makes the parsed tree independent of how the underlying parser chunked the
/// input; see REVIEW D9.
fn append_text(element: &Element, text: &str) -> XmlResult<()> {
    if text.is_empty() {
        return Ok(());
    }
    let document = element.document();
    if let Some(last) = element.children().last()
        && let Some(previous) = last.text()
    {
        let merged = document.create_text(format!("{}{text}", previous.as_str()))?;
        last.replace_with(merged);
        return Ok(());
    }
    element.append_child_checked(document.create_text(text)?)
}

fn parse_element(
    document: &Document,
    event: &BytesStart,
    ns_map: &HashMap<Option<NCName>, String>,
) -> XmlResult<Element> {
    let name = decode(event.name().into_inner(), "element name")?;
    let qualified_name = QualifiedName::resolve_element_with_namespace_map(name, ns_map)?;
    let element = document.create_element(qualified_name);

    for (prefix, uri) in extract_namespace_declarations(event)? {
        match prefix {
            Some(prefix) => element.declare_namespace(Namespace::prefixed(&uri, &prefix)?),
            None if uri.is_empty() => element.undeclare_default_namespace(),
            None => element.declare_namespace(Namespace::without_prefix(&uri)?),
        }
    }

    let mut attributes: BTreeMap<QualifiedName, String> = BTreeMap::new();
    for attribute in event.attributes() {
        let attribute = attribute.map_err(attribute_error)?;
        let key = decode(attribute.key.into_inner(), "attribute name")?;
        if key == "xmlns" || key.starts_with("xmlns:") {
            continue;
        }
        let value = attribute
            .normalized_value(XmlVersion::Explicit1_0)
            .map_err(|error| XmlError::MalformedXml(format!("invalid attribute value: {error}")))?;
        let name = QualifiedName::resolve_attribute_with_namespace_map(key, ns_map)?;
        if attributes.contains_key(&name) {
            return Err(XmlError::DuplicateAttribute(name.to_string()));
        }
        attributes.insert(name, value.to_string());
    }
    for (name, value) in attributes {
        element.set_attribute_checked(name, value)?;
    }
    Ok(element)
}

/// Extracts the namespace declarations carried by a start tag.
///
/// # Errors
///
/// Returns [`XmlError::InvalidNamespace`] if a prefix is undeclared with an empty string (which
/// the Namespaces specification forbids), and [`XmlError::DuplicateAttribute`] if the same prefix
/// is declared twice on one element.
fn extract_namespace_declarations(event: &BytesStart) -> XmlResult<Vec<(Option<NCName>, String)>> {
    let mut declarations = Vec::new();
    let mut seen: Vec<Option<NCName>> = Vec::new();
    for attribute in event.attributes() {
        let attribute = attribute.map_err(attribute_error)?;
        let key = decode(attribute.key.into_inner(), "attribute name")?;
        let value = attribute
            .normalized_value(XmlVersion::Explicit1_0)
            .map_err(|error| XmlError::MalformedXml(format!("invalid attribute value: {error}")))?;
        let prefix = if let Some(prefix) = key.strip_prefix("xmlns:") {
            if value.is_empty() {
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
        if seen.contains(&prefix) {
            return Err(XmlError::DuplicateAttribute(key.to_string()));
        }
        seen.push(prefix.clone());
        declarations.push((prefix, value.to_string()));
    }
    Ok(declarations)
}

/// Writes an XML document to a file.
///
/// # Errors
///
/// Returns [`XmlError::Io`] if the file cannot be created or written.
pub fn write_file<P: AsRef<Path>>(doc: &Document, path: P) -> XmlResult<()> {
    let file = File::create(path)?;
    write_writer(doc, BufWriter::new(file))
}

/// Serializes an XML document into a string.
///
/// A document without a root element serializes to the empty string.
///
/// # Errors
///
/// Returns [`XmlError::InvalidUtf8`] if the serialized output is not valid UTF-8, which cannot
/// happen for UTF-8 input but is reported rather than panicking.
pub fn write_string(doc: &Document) -> XmlResult<String> {
    let mut buffer = Vec::new();
    write_writer(doc, &mut buffer)?;
    String::from_utf8(buffer)
        .map_err(|error| XmlError::InvalidUtf8(format!("invalid UTF-8 in output: {error}")))
}

/// Writes an XML document to a generic writer.
///
/// # Errors
///
/// Returns [`XmlError::Io`] if the underlying writer fails.
pub fn write_writer<W: Write>(doc: &Document, writer: W) -> XmlResult<()> {
    let mut xml_writer = Writer::new(writer);
    if let Some(root) = doc.root() {
        write_element(&mut xml_writer, &root)?;
    }
    Ok(())
}

/// Serializes one element subtree to a string.
///
/// Used by the `Display` implementations of [`Element`] and [`Node`].
///
/// # Panics
///
/// Writing into a `String` cannot fail, so the intermediate `Result` is unwrapped with a message
/// that explains why this is unreachable.
pub(crate) fn write_element_to_string(element: &Element) -> String {
    let mut buffer = Vec::new();
    let mut xml_writer = Writer::new(&mut buffer);
    write_element(&mut xml_writer, element).expect("writing XML into a memory buffer cannot fail");
    String::from_utf8(buffer).expect("the serializer only ever emits UTF-8")
}

/// Writes an element and its subtree.
fn write_element<W: Write>(writer: &mut Writer<W>, element: &Element) -> XmlResult<()> {
    let mut attributes: Vec<(String, String)> = Vec::new();
    for (prefix, namespace) in element.namespace_declarations() {
        match (prefix, namespace) {
            (Some(prefix), Some(namespace)) => {
                attributes.push((format!("xmlns:{prefix}"), namespace.uri().to_string()))
            }
            (None, Some(namespace)) => {
                attributes.push(("xmlns".to_string(), namespace.uri().to_string()));
            }
            (None, None) => attributes.push(("xmlns".to_string(), String::new())),
            // `xmlns:prefix=""` is not a valid declaration, so it cannot be stored.
            (Some(_), None) => {}
        }
    }
    for (name, value) in element.attributes() {
        let key = match name.namespace().and_then(|namespace| namespace.prefix()) {
            Some(prefix) => format!("{prefix}:{}", name.local_name()),
            None => name.local_name().to_string(),
        };
        attributes.push((key, value.to_string()));
    }

    let name = element.qualified_name();
    let local = name.local_name().to_string();
    let start = BytesStart::new(&local).with_attributes(
        attributes
            .iter()
            .map(|(key, value)| (key.as_bytes(), value.as_bytes()))
            .collect::<Vec<_>>(),
    );
    writer.write_event(Event::Start(start))?;

    for child in element.children() {
        match child.content() {
            NodeContent::Element(child) => write_element(writer, &child)?,
            NodeContent::Text(text) => {
                if !text.as_str().is_empty() {
                    writer.write_event(Event::Text(BytesText::new(text.as_str())))?;
                }
            }
            NodeContent::Comment(comment) => {
                writer.write_event(Event::Comment(BytesText::new(comment.as_str())))?;
            }
            NodeContent::CData(cdata) => {
                writer.write_event(Event::CData(BytesCData::new(cdata.as_str())))?;
            }
            NodeContent::ProcessingInstruction(target, data) => {
                let content = if data.as_str().is_empty() {
                    target.as_str().to_string()
                } else {
                    format!("{} {}", target.as_str(), data.as_str())
                };
                writer.write_event(Event::PI(BytesPI::new(&content)))?;
            }
        }
    }

    writer.write_event(Event::End(BytesEnd::new(&local)))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml_spec::{PiData, PiTarget};

    #[test]
    fn test_parse_and_write_simple_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<root>
    <child id="1">Hello, World!</child>
    <child id="2">Another child</child>
</root>"#;

        let doc = parse_string(xml).unwrap();
        let output = write_string(&doc).unwrap();
        let doc2 = parse_string(&output).unwrap();
        assert_eq!(
            doc.root().unwrap().qualified_name(),
            doc2.root().unwrap().qualified_name()
        );
    }

    #[test]
    fn test_parse_with_namespaces() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<html:html xmlns:html="http://www.w3.org/1999/xhtml">
    <html:head>
        <html:title>Test Page</html:title>
    </html:head>
</html:html>"#;

        let doc = parse_string(xml).unwrap();
        let root = doc.root().unwrap();
        assert_eq!(root.local_name(), "html");
        assert_eq!(
            root.namespace()
                .map(|namespace| namespace.uri().to_string()),
            Some("http://www.w3.org/1999/xhtml".to_string())
        );
        assert_eq!(root.qualified_name().to_string(), "html:html");
    }

    #[test]
    fn test_write_created_document() {
        let doc = Document::empty();
        let html_ns = Namespace::prefixed("http://www.w3.org/1999/xhtml", "html").unwrap();
        let root = doc.create_element(QualifiedName::with_namespace("html", &html_ns).unwrap());
        root.declare_namespace(html_ns);
        doc.set_root(root.clone());

        let head = doc.create_element(QualifiedName::without_namespace("head").unwrap());
        let title = doc.create_element(QualifiedName::without_namespace("title").unwrap());
        title.append_child(doc.create_text("Test Page").unwrap());
        head.append_child(title);
        root.append_child(head);

        let output = write_string(&doc).unwrap();
        assert!(output.contains("<title>Test Page</title>"), "{output}");
    }

    #[test]
    fn test_scoped_namespaces() {
        let xml = r#"<root xmlns:default="http://default.com">
    <child xmlns:ex="http://example.com">
        <ex:element>Hello, <ex:s>World!</ex:s></ex:element>
        <nested xmlns:ex="http://example-another.com">
            <ex:element>Different namespace <ex:s>here!</ex:s></ex:element>
            <deep xmlns:ex="http://example-third.com">
                <ex:element>Third namespace</ex:element>
            </deep>
        </nested>
        <back_to_original>
            <ex:element>Back to first namespace</ex:element>
        </back_to_original>
    </child>
    <default:element>Default namespace element</default:element>
</root>"#;

        let doc = parse_string(xml).unwrap();
        let root = doc.root().unwrap();
        assert_eq!(root.local_name(), "root");
        assert_eq!(
            root.namespace_declarations()
                .get(&Some(crate::xml_spec::nc_name("default"))),
            Some(&Some(
                Namespace::prefixed("http://default.com", "default").unwrap()
            ))
        );

        let first_child = root.child_elements()[0].clone();
        assert_eq!(
            first_child.get_namespace(Some(&crate::xml_spec::nc_name("ex"))),
            Some(Namespace::prefixed("http://example.com", "ex").unwrap())
        );
        let nested = first_child.child_elements()[1].clone();
        assert_eq!(
            nested.get_namespace(Some(&crate::xml_spec::nc_name("ex"))),
            Some(Namespace::prefixed("http://example-another.com", "ex").unwrap())
        );
        let deep = nested.child_elements()[1].clone();
        assert_eq!(
            deep.get_namespace(Some(&crate::xml_spec::nc_name("ex"))),
            Some(Namespace::prefixed("http://example-third.com", "ex").unwrap())
        );
        let back = first_child.child_elements()[2].clone();
        assert_eq!(
            back.get_namespace(Some(&crate::xml_spec::nc_name("ex"))),
            Some(Namespace::prefixed("http://example.com", "ex").unwrap())
        );
    }

    #[test]
    fn test_mixed_content() {
        let xml = r#"<a> some text <b> other text </b> more text <c> other text </c> </a>"#;
        let doc = parse_string(xml).unwrap();
        let root = doc.root().unwrap();
        let actual = render(&root);
        assert_eq!(
            actual,
            vec![
                "text:` some text `",
                "element:b",
                "text:` more text `",
                "element:c",
                "text:` `",
            ]
        );
    }

    #[test]
    fn test_namespaced_attributes() {
        let xml = r#"<root xmlns:ex="http://example.com" ex:attr="value" attr2="other" />"#;
        let doc = parse_string(xml).unwrap();
        let root = doc.root().unwrap();
        let attributes = root.attributes();
        let namespaced = attributes
            .iter()
            .find(|(name, _)| name.local_name() == "attr" && name.namespace().is_some())
            .expect("missing namespaced attribute");
        assert_eq!(namespaced.1.as_ref(), "value");
        assert_eq!(
            namespaced.0.namespace().unwrap().uri(),
            "http://example.com"
        );
        let plain = attributes
            .iter()
            .find(|(name, _)| name.local_name() == "attr2")
            .expect("missing attr2");
        assert_eq!(plain.1.as_ref(), "other");
        assert!(plain.0.namespace().is_none());
    }

    #[test]
    fn test_comment_parsing_and_serialization() {
        let xml = r#"<root><!-- one --><child>Hello</child><!-- two --></root>"#;
        let doc = parse_string(xml).unwrap();
        let root = doc.root().unwrap();
        let comments: Vec<String> = root
            .children()
            .iter()
            .filter_map(|node| node.comment())
            .map(|comment| comment.as_str().to_string())
            .collect();
        assert_eq!(comments, vec![" one ".to_string(), " two ".to_string()]);

        let output = write_string(&doc).unwrap();
        let doc2 = parse_string(&output).unwrap();
        let reparsed = doc2
            .root()
            .unwrap()
            .children()
            .iter()
            .filter_map(|node| node.comment())
            .count();
        assert_eq!(reparsed, 2);
    }

    #[test]
    fn test_cdata_parsing_and_serialization() {
        let xml = r#"<root><![CDATA[with <tags> and &entities;]]></root>"#;
        let doc = parse_string(xml).unwrap();
        let root = doc.root().unwrap();
        let cdata = root
            .children()
            .iter()
            .find_map(|node| node.cdata())
            .expect("missing CDATA");
        assert_eq!(cdata.as_str(), "with <tags> and &entities;");

        let output = write_string(&doc).unwrap();
        assert!(output.contains("<![CDATA[with <tags> and &entities;]]>"));
    }

    #[test]
    fn test_processing_instruction_parsing_and_serialization() {
        let xml = r#"<root><?target data="value"?></root>"#;
        let doc = parse_string(xml).unwrap();
        let root = doc.root().unwrap();
        let (target, data) = root
            .children()
            .iter()
            .find_map(|node| node.processing_instruction())
            .expect("missing PI");
        assert_eq!(
            (target, data),
            (
                PiTarget::try_from("target").unwrap(),
                PiData::try_from("data=\"value\"").unwrap()
            )
        );
        let output = write_string(&doc).unwrap();
        assert!(output.contains("<?target data=\"value\"?>"), "{output}");
    }

    #[test]
    fn test_empty_default_namespace_removes_scope() {
        let xml = r#"<root xmlns="http://default.org">
    <in_ns>has default namespace</in_ns>
    <child xmlns="">
        <no_ns>should have no namespace</no_ns>
        <nested xmlns="http://other.org">
            <back_in_ns>nested back in a namespace</back_in_ns>
        </nested>
    </child>
    <after_empty>back to default namespace</after_empty>
</root>"#;

        let doc = parse_string(xml).unwrap();
        let root = doc.root().unwrap();
        assert_eq!(
            root.namespace()
                .map(|namespace| namespace.uri().to_string()),
            Some("http://default.org".to_string())
        );

        let child = root.child_elements()[1].clone();
        assert!(child.namespace().is_none());
        let no_ns = child.child_elements()[0].clone();
        assert!(no_ns.namespace().is_none());
        let nested = child.child_elements()[1].clone();
        assert_eq!(
            nested
                .namespace()
                .map(|namespace| namespace.uri().to_string()),
            Some("http://other.org".to_string())
        );
        let after = root.child_elements()[2].clone();
        assert_eq!(
            after
                .namespace()
                .map(|namespace| namespace.uri().to_string()),
            Some("http://default.org".to_string())
        );
    }

    #[test]
    fn test_duplicate_attribute_is_rejected() {
        let error = parse_string(r#"<root a="1" a="2"/>"#).unwrap_err();
        assert!(
            matches!(error, XmlError::DuplicateAttribute(_)),
            "unexpected error: {error}"
        );
    }

    /// Renders the child structure of an element as strings, for structural assertions.
    fn render(element: &Element) -> Vec<String> {
        element
            .children()
            .iter()
            .map(|node| match node.content() {
                NodeContent::Element(child) => {
                    format!("element:{}", child.qualified_name().local_name())
                }
                NodeContent::Text(text) => format!("text:`{}`", text.as_str()),
                NodeContent::Comment(comment) => format!("comment:`{}`", comment.as_str()),
                NodeContent::CData(cdata) => format!("cdata:`{}`", cdata.as_str()),
                NodeContent::ProcessingInstruction(target, data) => {
                    format!("pi:`{}`:`{}`", target.as_str(), data.as_str())
                }
            })
            .collect()
    }
}
