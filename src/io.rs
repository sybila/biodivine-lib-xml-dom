use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Reader;
use quick_xml::Writer;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use crate::document::Document;
use crate::element::Element;
use crate::error::{XmlError, XmlResult};
use crate::namespace::{Attribute, Namespace};

/// Parse XML from a file
pub fn parse_file<P: AsRef<Path>>(path: P) -> XmlResult<Document> {
    let file = File::open(path)
        .map_err(|e| XmlError::InvalidXml(format!("Failed to open file: {}", e)))?;
    let reader = BufReader::new(file);
    parse_reader(reader)
}

/// Parse XML from a string
pub fn parse_string(xml: &str) -> XmlResult<Document> {
    let reader = BufReader::new(xml.as_bytes());
    parse_reader(reader)
}

/// Parse XML from a generic reader
pub fn parse_reader<R: BufRead>(reader: R) -> XmlResult<Document> {
    let mut xml_reader = Reader::from_reader(reader);
    xml_reader.trim_text(false);

    let doc = Document::new();
    let mut stack: Vec<Element> = Vec::new();
    let mut buf = Vec::new();

    loop {
        match xml_reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let element = parse_start_element(&doc, e)?;
                if let Some(parent) = stack.last() {
                    parent.add_child_element(element.clone())?;
                } else {
                    doc.set_root(element.clone())?;
                }
                stack.push(element);
            }
            Ok(Event::End(_)) => {
                stack.pop();
            }
            Ok(Event::Text(e)) => {
                if let Some(current) = stack.last() {
                    let text = e.unescape().map_err(|e| {
                        XmlError::InvalidXml(format!("Invalid text content: {}", e))
                    })?;
                    current.add_text(text.to_string());
                }
            }
            Ok(Event::Eof) => break,
            Ok(Event::Comment(_)) => {}
            Ok(Event::Decl(_)) => {}
            Ok(Event::PI(_)) => {}
            Ok(Event::CData(e)) => {
                if let Some(current) = stack.last() {
                    let text = std::str::from_utf8(&e).map_err(|e| {
                        XmlError::InvalidXml(format!("Invalid CDATA content: {}", e))
                    })?;
                    current.add_text(text.to_string());
                }
            }
            Ok(Event::DocType(_)) => {}
            Ok(Event::Empty(ref e)) => {
                let element = parse_empty_element(&doc, e)?;
                if let Some(parent) = stack.last() {
                    parent.add_child_element(element);
                } else {
                    doc.set_root(element)?;
                }
            }
            Err(e) => return Err(XmlError::InvalidXml(format!("XML parsing error: {}", e))),
        }
        buf.clear();
    }
    Ok(doc)
}

/// Parse a start element and its attributes
fn parse_start_element(doc: &Document, e: &BytesStart) -> XmlResult<Element> {
    let name = std::str::from_utf8(e.name().into_inner()).map_err(|e| {
        XmlError::InvalidXml(format!("Invalid UTF-8 in element name: {}", e))
    })?;

    let (local_name, default_namespace) = parse_element_name(name);
    let namespace_declarations = extract_namespace_declarations(e)?;
    let attributes = extract_regular_attributes(e)?;

    // Create the initial element
    let element = if let Some(ns) = default_namespace {
        doc.create_element_with_namespace(local_name.clone(), ns)
    } else {
        doc.create_element(local_name.clone())
    };

    // Declare namespaces on the element
    for (prefix, uri) in namespace_declarations {
        if prefix.is_empty() {
            element.declare_default_namespace(uri);
        } else {
            element.declare_namespace(prefix, uri);
        }
    }

    // Add regular attributes
    for attr in attributes {
        element.add_attribute(attr);
    }

    // Resolve the element's namespace if it has a qualified name
    resolve_element_namespace(doc, element, name, local_name)
}

/// Parse an empty (self-closing) element and its attributes
fn parse_empty_element(doc: &Document, e: &BytesStart) -> XmlResult<Element> {
    let name = std::str::from_utf8(e.name().into_inner()).map_err(|e| {
        XmlError::InvalidXml(format!("Invalid UTF-8 in element name: {}", e))
    })?;

    let (local_name, default_namespace) = parse_element_name(name);
    let namespace_declarations = extract_namespace_declarations(e)?;
    let attributes = extract_regular_attributes(e)?;

    // Create the initial element
    let element = if let Some(ns) = default_namespace {
        doc.create_element_with_namespace(local_name.clone(), ns)
    } else {
        doc.create_element(local_name.clone())
    };

    // Declare namespaces on the element
    for (prefix, uri) in namespace_declarations {
        if prefix.is_empty() {
            element.declare_default_namespace(uri);
        } else {
            element.declare_namespace(prefix, uri);
        }
    }

    // Add regular attributes
    for attr in attributes {
        element.add_attribute(attr);
    }

    // Resolve the element's namespace if it has a qualified name
    resolve_element_namespace(doc, element, name, local_name)
}

/// Parse element name and extract local name and default namespace
fn parse_element_name(name: &str) -> (String, Option<Namespace>) {
    if let Some(colon_pos) = name.find(':') {
        let local_name = &name[colon_pos + 1..];
        (local_name.to_string(), None)
    } else {
        (name.to_string(), None)
    }
}

/// Extract namespace declarations from attributes
fn extract_namespace_declarations(e: &BytesStart) -> XmlResult<Vec<(String, String)>> {
    let mut namespace_declarations = Vec::new();
    for attr in e.attributes() {
        let attr = attr
            .map_err(|e| XmlError::InvalidXml(format!("Invalid attribute: {}", e)))?;
        let key = std::str::from_utf8(attr.key.into_inner()).map_err(|e| {
            XmlError::InvalidXml(format!("Invalid UTF-8 in attribute name: {}", e))
        })?;
        let value = attr.unescape_value().map_err(|e| {
            XmlError::InvalidXml(format!("Invalid attribute value: {}", e))
        })?;
        if let Some(prefix) = key.strip_prefix("xmlns:") {
            namespace_declarations.push((prefix.to_string(), value.to_string()));
        } else if key == "xmlns" {
            namespace_declarations.push(("".to_string(), value.to_string()));
        }
    }
    Ok(namespace_declarations)
}

/// Extract regular (non-namespace) attributes
fn extract_regular_attributes(e: &BytesStart) -> XmlResult<Vec<Attribute>> {
    let mut attributes = Vec::new();
    for attr in e.attributes() {
        let attr = attr
            .map_err(|e| XmlError::InvalidXml(format!("Invalid attribute: {}", e)))?;
        let key = std::str::from_utf8(attr.key.into_inner()).map_err(|e| {
            XmlError::InvalidXml(format!("Invalid UTF-8 in attribute name: {}", e))
        })?;
        let value = attr.unescape_value().map_err(|e| {
            XmlError::InvalidXml(format!("Invalid attribute value: {}", e))
        })?;
        if !key.starts_with("xmlns") {
            attributes.push(Attribute::new(key.to_string(), value.to_string()));
        }
    }
    Ok(attributes)
}

/// Resolve element namespace if it has a qualified name
fn resolve_element_namespace(
    doc: &Document,
    element: Element,
    original_name: &str,
    local_name: String,
) -> XmlResult<Element> {
    if let Some(colon_pos) = original_name.find(':') {
        let prefix = &original_name[..colon_pos];
        if let Some(uri) = element.get_namespace_uri(prefix) {
            // Create a new element with the resolved namespace
            let namespaced_element = doc.create_element_with_namespace(
                local_name,
                Namespace::prefixed(uri, prefix.to_string()),
            );

            // Copy namespace declarations and attributes
            for (ns_prefix, ns_uri) in element.namespace_declarations() {
                if ns_prefix.is_empty() {
                    namespaced_element.declare_default_namespace(ns_uri);
                } else {
                    namespaced_element.declare_namespace(ns_prefix, ns_uri);
                }
            }
            for attr in element.attributes() {
                namespaced_element.add_attribute(attr);
            }

            Ok(namespaced_element)
        } else {
            Ok(element)
        }
    } else {
        Ok(element)
    }
}

/// Write XML document to a file
pub fn write_file<P: AsRef<Path>>(doc: &Document, path: P) -> XmlResult<()> {
    let file = File::create(path)
        .map_err(|e| XmlError::InvalidXml(format!("Failed to create file: {}", e)))?;
    let writer = BufWriter::new(file);
    write_writer(doc, writer)
}

/// Write XML document to a string
pub fn write_string(doc: &Document) -> XmlResult<String> {
    let mut buffer = Vec::new();
    write_writer(doc, &mut buffer)?;
    String::from_utf8(buffer)
        .map_err(|e| XmlError::InvalidXml(format!("Invalid UTF-8 in output: {}", e)))
}

/// Write XML document to a generic writer
pub fn write_writer<W: Write>(doc: &Document, writer: W) -> XmlResult<()> {
    let mut xml_writer = Writer::new(writer);

    if let Some(root) = doc.root() {
        write_element(&mut xml_writer, &root)?;
    }

    Ok(())
}

/// Write a single element and its children
fn write_element<W: Write>(writer: &mut Writer<W>, element: &Element) -> XmlResult<()> {
    let mut attrs = Vec::new();
    for (prefix, uri) in element.namespace_declarations() {
        if prefix.is_empty() {
            attrs.push(("xmlns".to_string(), uri));
        } else {
            attrs.push((format!("xmlns:{}", prefix), uri));
        }
    }
    for attr in element.attributes() {
        attrs.push((attr.name.clone(), attr.value.clone()));
    }
    let start = BytesStart::new(element.name()).with_attributes(
        attrs.iter().map(|(k, v)| (k.as_bytes(), v.as_bytes())).collect::<Vec<_>>(),
    );
    writer.write_event(Event::Start(start))?;
    for node in element.children() {
        match node {
            crate::element::XmlNode::Element(ref child) => {
                write_element(writer, child)?;
            }
            crate::element::XmlNode::Text(ref text) => {
                if !text.is_empty() {
                    let text_event = BytesText::new(text);
                    writer.write_event(Event::Text(text_event))?;
                }
            }
        }
    }
    let end = BytesEnd::new(element.name());
    writer.write_event(Event::End(end))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_document;

    #[test]
    fn test_parse_and_write_simple_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<root>
    <child id="1">Hello, World!</child>
    <child id="2">Another child</child>
</root>"#;

        let doc = parse_string(xml).unwrap();
        let output = write_string(&doc).unwrap();

        // Parse again to verify round-trip
        let doc2 = parse_string(&output).unwrap();
        assert_eq!(doc.root().unwrap().name(), doc2.root().unwrap().name());
    }

    #[test]
    fn test_parse_with_namespaces() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<html:html xmlns:html="http://www.w3.org/1999/xhtml">
    <html:head>
        <html:title>Test Page</html:title>
    </html:head>
    <html:body>
        <html:p>Hello, World!</html:p>
    </html:body>
</html:html>"#;

        let doc = parse_string(xml).unwrap();
        let root = doc.root().unwrap();

        assert_eq!(root.name(), "html");
        assert!(root.namespace().is_some());
        assert_eq!(root.namespace().unwrap().uri, "http://www.w3.org/1999/xhtml");
        assert_eq!(root.qualified_name(), "html:html");
    }

    #[test]
    fn test_write_created_document() {
        let doc = create_document();

        let html_ns = Namespace::prefixed(
            "http://www.w3.org/1999/xhtml".to_string(),
            "html".to_string(),
        );
        let root = doc.create_element_with_namespace("html".to_string(), html_ns);
        root.declare_namespace(
            "html".to_string(),
            "http://www.w3.org/1999/xhtml".to_string(),
        );
        doc.set_root(root.clone()).unwrap();

        let head = doc.create_element("head".to_string());
        let title = doc.create_element("title".to_string());
        title.add_text("Test Page".to_string());
        head.add_child_element(title).unwrap();
        root.add_child_element(head).unwrap();

        let output = write_string(&doc).unwrap();
        assert!(output.contains("<html"));
        assert!(output.contains("<head"));
        assert!(output.contains("<title>Test Page</title>"));
    }

    #[test]
    fn test_scoped_namespaces() {
        // Test that namespaces are properly scoped to elements
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<root xmlns:default="http://default.com">
    <child xmlns:ex="http://example.com">
        <ex:element>Hello, <ex:s>World!</ex:s></ex:element>
        <nested xmlns:ex="http://example-another.com">
            <ex:element>Different namespace <ex:s>here!</ex:s></ex:element>
            <deep xmlns:ex="http://example-third.com">
                <ex:element>Third namespace <ex:s>here!</ex:s></ex:element>
            </deep>
        </nested>
        <back_to_original>
            <ex:element>Back to first namespace <ex:s>here!</ex:s></ex:element>
        </back_to_original>
    </child>
    <child xmlns:ex="http://example-another.com">
        <ex:element>Hello, <ex:s>World!</ex:s></ex:element>
        <nested xmlns:ex="http://example-fourth.com">
            <ex:element>Fourth namespace <ex:s>here!</ex:s></ex:element>
        </nested>
    </child>
    <default:element>Default namespace element</default:element>
</root>"#;

        let doc = parse_string(xml).unwrap();
        let root = doc.root().unwrap();

        assert_eq!(root.name(), "root");
        assert_eq!(root.namespace_declarations().get("default"), Some(&"http://default.com".to_string()));

        let first_child = root.element_children()[0].clone();
        assert_eq!(first_child.get_namespace_uri("ex"), Some("http://example.com".to_string()));

        let nested = first_child.element_children()[1].clone();
        assert_eq!(nested.get_namespace_uri("ex"), Some("http://example-another.com".to_string()));

        let deep = nested.element_children()[1].clone();
        assert_eq!(deep.get_namespace_uri("ex"), Some("http://example-third.com".to_string()));

        let back_to_original = first_child.element_children()[2].clone();
        assert_eq!(back_to_original.get_namespace_uri("ex"), Some("http://example.com".to_string()));

        let second_child = root.element_children()[1].clone();
        assert_eq!(second_child.get_namespace_uri("ex"), Some("http://example-another.com".to_string()));

        let output = write_string(&doc).unwrap();
        let doc2 = parse_string(&output).unwrap();
        let root2 = doc2.root().unwrap();
        assert_eq!(root2.get_namespace_uri("default"), Some("http://default.com".to_string()));
    }

    #[test]
    fn test_mixed_content() {
        let xml = r#"<a> some text <b> other text </b> more text <c> other text </c> </a>"#;
        let doc = parse_string(xml).unwrap();
        let root = doc.root().unwrap();
        assert_eq!(root.name(), "a");
        let children = root.children();
        let mut actual: Vec<String> = vec![];
        for node in children {
            match node {
                crate::element::XmlNode::Text(t) => actual.push(format!("text:{:?}", t)),
                crate::element::XmlNode::Element(e) => actual.push(format!("element:{}", e.name())),
            }
        }
        let expected = vec![
            "text:\" some text \"",
            "element:b",
            "text:\" more text \"",
            "element:c",
            "text:\" \"",
        ];
        assert_eq!(actual, expected, "Mixed content structure should be preserved");
    }
}


