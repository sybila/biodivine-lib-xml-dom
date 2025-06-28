use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Reader;
use quick_xml::Writer;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use crate::document::Document;
use crate::element::Element;
use crate::error::{XmlError, XmlResult};
use crate::Namespace;
use crate::QualifiedName;

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
    let mut ns_stack: Vec<std::collections::HashMap<String, String>> =
        vec![std::collections::HashMap::new()];
    let mut buf = Vec::new();

    loop {
        match xml_reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                // Clone the current namespace map and update with new declarations
                let mut ns_map = ns_stack.last().unwrap().clone();
                let namespace_declarations = extract_namespace_declarations(e)?;
                for (prefix, uri) in &namespace_declarations {
                    ns_map.insert(prefix.clone(), uri.clone());
                }
                ns_stack.push(ns_map.clone());
                let parent = stack.last();
                let element = parse_element(&doc, e, &ns_map)?;
                if let Some(parent) = parent {
                    parent.add_child_element(element.clone())?;
                } else {
                    doc.set_root(element.clone())?;
                }
                stack.push(element);
            }
            Ok(Event::End(_)) => {
                stack.pop();
                ns_stack.pop();
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
            Ok(Event::Comment(e)) => {
                if let Some(current) = stack.last() {
                    let comment = std::str::from_utf8(&e).map_err(|e| {
                        XmlError::InvalidXml(format!("Invalid UTF-8 in comment: {}", e))
                    })?;
                    current.add_comment(comment.to_string());
                }
            }
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
                // Clone the current namespace map and update with new declarations
                let mut ns_map = ns_stack.last().unwrap().clone();
                let namespace_declarations = extract_namespace_declarations(e)?;
                for (prefix, uri) in &namespace_declarations {
                    ns_map.insert(prefix.clone(), uri.clone());
                }
                let parent = stack.last();
                let element = parse_element(&doc, e, &ns_map)?;
                if let Some(parent) = parent {
                    parent.add_child_element(element.clone())?;
                } else {
                    doc.set_root(element.clone())?;
                }
            }
            Err(e) => return Err(XmlError::InvalidXml(format!("XML parsing error: {}", e))),
        }
        buf.clear();
    }
    Ok(doc)
}

fn parse_element(
    doc: &Document,
    e: &BytesStart,
    ns_map: &std::collections::HashMap<String, String>,
) -> XmlResult<Element> {
    // 1. Extract namespace declarations (already done in caller)
    // 2. Use the provided ns_map for resolution
    // 3. Resolve the qualified name of the tag
    let name = std::str::from_utf8(e.name().into_inner())
        .map_err(|e| XmlError::InvalidXml(format!("Invalid UTF-8 in element name: {}", e)))?;
    let qname = match QualifiedName::resolve_with_namespace_map(name, ns_map) {
        Ok(q) => q,
        Err(e) => {
            eprintln!(
                "[DEBUG] Failed to resolve element name '{}'. Namespace map: {:?}",
                name, ns_map
            );
            return Err(e);
        }
    };
    // 4. Create the element with the correct qualified name
    let element = doc.create_element(qname.clone());
    // 5. Apply namespace declarations to the element
    let namespace_declarations = extract_namespace_declarations(e)?;
    for (prefix, uri) in namespace_declarations {
        if prefix.is_empty() {
            element.declare_default_namespace(Namespace::default(&uri).unwrap());
        } else {
            element.declare_namespace(prefix.clone(), Namespace::prefixed(&uri, &prefix).unwrap());
        }
    }
    // 6. Add all attributes, resolving their qualified names using the provided ns_map
    let mut attributes = BTreeMap::new();
    for attr in e.attributes() {
        let attr = attr.map_err(|e| XmlError::InvalidXml(format!("Invalid attribute: {}", e)))?;
        let key = std::str::from_utf8(attr.key.into_inner())
            .map_err(|e| XmlError::InvalidXml(format!("Invalid UTF-8 in attribute name: {}", e)))?;
        let value = attr
            .unescape_value()
            .map_err(|e| XmlError::InvalidXml(format!("Invalid attribute value: {}", e)))?;
        if key.starts_with("xmlns") {
            continue;
        }
        let qattr = match QualifiedName::resolve_with_namespace_map(key, ns_map) {
            Ok(q) => q,
            Err(e) => {
                eprintln!(
                    "[DEBUG] Failed to resolve attribute name '{}'. Namespace map: {:?}",
                    key, ns_map
                );
                return Err(e);
            }
        };
        attributes.insert(qattr, value.to_string());
    }
    element.set_attributes(attributes);
    Ok(element)
}

/// Extract namespace declarations from attributes
fn extract_namespace_declarations(e: &BytesStart) -> XmlResult<Vec<(String, String)>> {
    let mut namespace_declarations = Vec::new();
    for attr in e.attributes() {
        let attr = attr.map_err(|e| XmlError::InvalidXml(format!("Invalid attribute: {}", e)))?;
        let key = std::str::from_utf8(attr.key.into_inner())
            .map_err(|e| XmlError::InvalidXml(format!("Invalid UTF-8 in attribute name: {}", e)))?;
        let value = attr
            .unescape_value()
            .map_err(|e| XmlError::InvalidXml(format!("Invalid attribute value: {}", e)))?;
        if let Some(prefix) = key.strip_prefix("xmlns:") {
            namespace_declarations.push((prefix.to_string(), value.to_string()));
        } else if key == "xmlns" {
            namespace_declarations.push(("".to_string(), value.to_string()));
        }
    }
    Ok(namespace_declarations)
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
    for (prefix, ns) in element.namespace_declarations() {
        if prefix.is_empty() {
            attrs.push(("xmlns".to_string(), ns.uri().to_string()));
        } else {
            attrs.push((format!("xmlns:{}", prefix), ns.uri().to_string()));
        }
    }
    for (qname, value) in element.attributes().iter() {
        if let Some(ns) = qname.namespace() {
            if let Some(prefix) = ns.prefix() {
                attrs.push((format!("{}:{}", prefix, qname.name()), value.clone()));
            } else {
                attrs.push((qname.name().to_string(), value.clone()));
            }
        } else {
            attrs.push((qname.name().to_string(), value.clone()));
        }
    }
    let start = BytesStart::new(element.name()).with_attributes(
        attrs
            .iter()
            .map(|(k, v)| (k.as_bytes(), v.as_bytes()))
            .collect::<Vec<_>>(),
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
            crate::element::XmlNode::Comment(ref comment) => {
                let comment_event = BytesText::new(comment);
                writer.write_event(Event::Comment(comment_event))?;
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
    use crate::Namespace;

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
        assert_eq!(
            root.namespace().unwrap().uri(),
            "http://www.w3.org/1999/xhtml"
        );
        assert_eq!(root.qualified_name().qualified_name_string(), "html:html");
    }

    #[test]
    fn test_write_created_document() {
        let doc = create_document();

        let html_ns = Namespace::prefixed("http://www.w3.org/1999/xhtml", "html").unwrap();
        let root = doc.create_element(QualifiedName::with_namespace("html", &html_ns).unwrap());
        root.declare_namespace("html".to_string(), html_ns.clone());
        doc.set_root(root.clone()).unwrap();

        let head = doc.create_element(QualifiedName::without_namespace("head").unwrap());
        let title = doc.create_element(QualifiedName::without_namespace("title").unwrap());
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
        assert_eq!(
            root.namespace_declarations().get("default"),
            Some(&Namespace::prefixed("http://default.com", "default").unwrap())
        );

        let first_child = root.element_children()[0].clone();
        assert_eq!(
            first_child.get_namespace("ex"),
            Some(Namespace::prefixed("http://example.com", "ex").unwrap())
        );

        let nested = first_child.element_children()[1].clone();
        assert_eq!(
            nested.get_namespace("ex"),
            Some(Namespace::prefixed("http://example-another.com", "ex").unwrap())
        );

        let deep = nested.element_children()[1].clone();
        assert_eq!(
            deep.get_namespace("ex"),
            Some(Namespace::prefixed("http://example-third.com", "ex").unwrap())
        );

        let back_to_original = first_child.element_children()[2].clone();
        assert_eq!(
            back_to_original.get_namespace("ex"),
            Some(Namespace::prefixed("http://example.com", "ex").unwrap())
        );

        let second_child = root.element_children()[1].clone();
        assert_eq!(
            second_child.get_namespace("ex"),
            Some(Namespace::prefixed("http://example-another.com", "ex").unwrap())
        );

        let output = write_string(&doc).unwrap();
        let doc2 = parse_string(&output).unwrap();
        let root2 = doc2.root().unwrap();
        assert_eq!(
            root2.get_namespace("default"),
            Some(Namespace::prefixed("http://default.com", "default").unwrap())
        );
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
                crate::element::XmlNode::Comment(c) => actual.push(format!("comment:{:?}", c)),
            }
        }
        let expected = vec![
            "text:\" some text \"",
            "element:b",
            "text:\" more text \"",
            "element:c",
            "text:\" \"",
        ];
        assert_eq!(
            actual, expected,
            "Mixed content structure should be preserved"
        );
    }

    #[test]
    fn test_namespaced_attributes() {
        let xml = r#"<root xmlns:ex="http://example.com" ex:attr="value" attr2="other" />"#;
        let doc = parse_string(xml).unwrap();
        let root = doc.root().unwrap();
        let attrs = root.attributes();
        // Find the namespaced attribute
        let ns_attr = attrs
            .iter()
            .find(|(q, _)| q.name() == "attr" && q.namespace().is_some())
            .expect("Missing namespaced attribute");
        assert_eq!(ns_attr.1, "value");
        assert_eq!(
            ns_attr.0.namespace().as_ref().unwrap().uri(),
            "http://example.com"
        );
        assert_eq!(ns_attr.0.namespace().as_ref().unwrap().prefix(), Some("ex"));
        // Find the non-namespaced attribute
        let attr2 = attrs
            .iter()
            .find(|(q, _)| q.name() == "attr2")
            .expect("Missing attr2");
        assert_eq!(attr2.1, "other");
        assert!(attr2.0.namespace().is_none());
        // Round-trip
        let output = write_string(&doc).unwrap();
        let doc2 = parse_string(&output).unwrap();
        let root2 = doc2.root().unwrap();
        let attrs2 = root2.attributes();
        let ns_attr2 = attrs2
            .iter()
            .find(|(q, _)| q.name() == "attr" && q.namespace().is_some())
            .expect("Missing namespaced attribute after round-trip");
        assert_eq!(ns_attr2.1, "value");
        assert_eq!(
            ns_attr2.0.namespace().as_ref().unwrap().uri(),
            "http://example.com"
        );
        assert_eq!(
            ns_attr2.0.namespace().as_ref().unwrap().prefix(),
            Some("ex")
        );
    }

    #[test]
    fn test_namespaced_attributes_on_parent() {
        let xml =
            r#"<root xmlns:ex="http://example.com"><child ex:attr="value" attr2="other" /></root>"#;
        let doc = parse_string(xml).unwrap();
        let root = doc.root().unwrap();
        let child = root.element_children()[0].clone();
        let attrs = child.attributes();
        // Find the namespaced attribute
        let ns_attr = attrs
            .iter()
            .find(|(q, _)| q.name() == "attr" && q.namespace().is_some())
            .expect("Missing namespaced attribute");
        assert_eq!(ns_attr.1, "value");
        assert_eq!(
            ns_attr.0.namespace().as_ref().unwrap().uri(),
            "http://example.com"
        );
        assert_eq!(ns_attr.0.namespace().as_ref().unwrap().prefix(), Some("ex"));
        // Find the non-namespaced attribute
        let attr2 = attrs
            .iter()
            .find(|(q, _)| q.name() == "attr2")
            .expect("Missing attr2");
        assert_eq!(attr2.1, "other");
        assert!(attr2.0.namespace().is_none());
        // Round-trip
        let output = write_string(&doc).unwrap();
        let doc2 = parse_string(&output).unwrap();
        let root2 = doc2.root().unwrap();
        let child2 = root2.element_children()[0].clone();
        let attrs2 = child2.attributes();
        let ns_attr2 = attrs2
            .iter()
            .find(|(q, _)| q.name() == "attr" && q.namespace().is_some())
            .expect("Missing namespaced attribute after round-trip");
        assert_eq!(ns_attr2.1, "value");
        assert_eq!(
            ns_attr2.0.namespace().as_ref().unwrap().uri(),
            "http://example.com"
        );
        assert_eq!(
            ns_attr2.0.namespace().as_ref().unwrap().prefix(),
            Some("ex")
        );
    }

    #[test]
    fn test_comment_parsing_and_serialization() {
        let xml = r#"<root>
            <!-- This is a comment -->
            <child>Hello, World!</child>
            <!-- Another comment -->
            <child>Another child</child>
            <!-- Final comment -->
        </root>"#;

        let doc = parse_string(xml).unwrap();
        let root = doc.root().unwrap();

        // Check that comments are parsed
        let comments = root.comment_children();
        assert_eq!(comments.len(), 3);
        assert_eq!(comments[0], " This is a comment ");
        assert_eq!(comments[1], " Another comment ");
        assert_eq!(comments[2], " Final comment ");

        // Check that elements are still parsed correctly
        let elements = root.element_children();
        assert_eq!(elements.len(), 2);
        assert_eq!(elements[0].name(), "child");
        assert_eq!(elements[1].name(), "child");

        // Check round-trip serialization
        let output = write_string(&doc).unwrap();
        let doc2 = parse_string(&output).unwrap();
        let root2 = doc2.root().unwrap();

        let comments2 = root2.comment_children();
        assert_eq!(comments2.len(), 3);
        assert_eq!(comments2[0], " This is a comment ");
        assert_eq!(comments2[1], " Another comment ");
        assert_eq!(comments2[2], " Final comment ");
    }

    #[test]
    fn test_comment_creation() {
        let doc = create_document();
        let root = doc.create_element(QualifiedName::without_namespace("root").unwrap());
        doc.set_root(root.clone()).unwrap();

        // Add comments programmatically
        root.add_comment(" This is a test comment ".to_string());
        root.add_comment(" Another test comment ".to_string());

        let comments = root.comment_children();
        assert_eq!(comments.len(), 2);
        assert_eq!(comments[0], " This is a test comment ");
        assert_eq!(comments[1], " Another test comment ");

        // Test serialization
        let output = write_string(&doc).unwrap();
        assert!(output.contains("<!-- This is a test comment -->"));
        assert!(output.contains("<!-- Another test comment -->"));
    }
}
