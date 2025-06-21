use std::fs::File;
use std::io::{BufReader, BufWriter, BufRead, Read, Write};
use std::path::Path;
use std::sync::Arc;

use quick_xml::events::{BytesStart, BytesEnd, BytesText, Event};
use quick_xml::Reader;
use quick_xml::Writer;

use crate::error::{XmlError, XmlResult};
use crate::namespace::{Attribute, Namespace};
use crate::element::Element;
use crate::document::Document;

/// Parse XML from a file path
pub fn parse_file<P: AsRef<Path>>(path: P) -> XmlResult<Document> {
    let file = File::open(path).map_err(|e| XmlError::InvalidXml(format!("Failed to open file: {}", e)))?;
    let reader = BufReader::new(file);
    parse_reader(reader)
}

/// Parse XML from a string
pub fn parse_string(xml: &str) -> XmlResult<Document> {
    parse_reader(BufReader::new(xml.as_bytes()))
}

/// Parse XML from a generic reader
pub fn parse_reader<R: BufRead>(reader: R) -> XmlResult<Document> {
    let mut xml_reader = Reader::from_reader(reader);
    xml_reader.trim_text(true);

    let doc = Document::new();
    let mut stack: Vec<Arc<Element>> = Vec::new();
    let mut buf = Vec::new();

    loop {
        match xml_reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = std::str::from_utf8(e.name().into_inner())
                    .map_err(|e| XmlError::InvalidXml(format!("Invalid UTF-8 in element name: {}", e)))?;
                
                // Parse namespace declarations from attributes
                let mut namespace = None;
                let mut attributes = Vec::new();
                
                for attr in e.attributes() {
                    let attr = attr.map_err(|e| XmlError::InvalidXml(format!("Invalid attribute: {}", e)))?;
                    let key = std::str::from_utf8(attr.key.into_inner())
                        .map_err(|e| XmlError::InvalidXml(format!("Invalid UTF-8 in attribute name: {}", e)))?;
                    let value = attr.unescape_value()
                        .map_err(|e| XmlError::InvalidXml(format!("Invalid attribute value: {}", e)))?;
                    
                    if key.starts_with("xmlns:") {
                        // Prefixed namespace declaration
                        let prefix = &key[6..]; // Remove "xmlns:"
                        doc.declare_namespace(prefix.to_string(), value.to_string());
                    } else if key == "xmlns" {
                        // Default namespace declaration
                        doc.declare_default_namespace(value.to_string());
                        namespace = Some(Namespace::default(value.to_string()));
                    } else {
                        // Regular attribute
                        attributes.push(Attribute::new(key.to_string(), value.to_string()));
                    }
                }
                
                // Handle qualified names (prefix:local_name)
                let (local_name, element_namespace) = if let Some(colon_pos) = name.find(':') {
                    let prefix = &name[..colon_pos];
                    let local_name = &name[colon_pos + 1..];
                    
                    if let Some(uri) = doc.get_namespace_uri(prefix) {
                        let ns = Namespace::prefixed(uri, prefix.to_string());
                        (local_name.to_string(), Some(ns))
                    } else {
                        (name.to_string(), None)
                    }
                } else {
                    (name.to_string(), namespace)
                };
                
                // Create element
                let element = if let Some(ns) = element_namespace {
                    doc.create_element_with_namespace(local_name, ns)
                } else {
                    doc.create_element(local_name)
                };
                
                // Add attributes
                for attr in attributes {
                    element.add_attribute(attr);
                }
                
                // Add to parent or set as root
                if let Some(parent) = stack.last() {
                    parent.add_child(element.clone())?;
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
                    let text = e.unescape()
                        .map_err(|e| XmlError::InvalidXml(format!("Invalid text content: {}", e)))?;
                    current.set_text_content(text.to_string());
                }
            }
            
            Ok(Event::Eof) => break,
            
            Ok(Event::Comment(_)) => {
                // Ignore comments for now
            }
            
            Ok(Event::Decl(_)) => {
                // Ignore XML declaration for now
            }
            
            Ok(Event::PI(_)) => {
                // Ignore processing instructions for now
            }
            
            Ok(Event::CData(e)) => {
                // Treat CDATA as text for now
                if let Some(current) = stack.last() {
                    let text = std::str::from_utf8(&e)
                        .map_err(|e| XmlError::InvalidXml(format!("Invalid CDATA content: {}", e)))?;
                    current.set_text_content(text.to_string());
                }
            }
            
            Ok(Event::DocType(_)) => {
                // Ignore DOCTYPE declarations for now
            }
            
            Ok(Event::Empty(ref e)) => {
                // Handle self-closing elements
                let name = std::str::from_utf8(e.name().into_inner())
                    .map_err(|e| XmlError::InvalidXml(format!("Invalid UTF-8 in element name: {}", e)))?;
                
                let mut namespace = None;
                let mut attributes = Vec::new();
                
                for attr in e.attributes() {
                    let attr = attr.map_err(|e| XmlError::InvalidXml(format!("Invalid attribute: {}", e)))?;
                    let key = std::str::from_utf8(attr.key.into_inner())
                        .map_err(|e| XmlError::InvalidXml(format!("Invalid UTF-8 in attribute name: {}", e)))?;
                    let value = attr.unescape_value()
                        .map_err(|e| XmlError::InvalidXml(format!("Invalid attribute value: {}", e)))?;
                    
                    if key.starts_with("xmlns:") {
                        let prefix = &key[6..];
                        doc.declare_namespace(prefix.to_string(), value.to_string());
                    } else if key == "xmlns" {
                        doc.declare_default_namespace(value.to_string());
                        namespace = Some(Namespace::default(value.to_string()));
                    } else {
                        attributes.push(Attribute::new(key.to_string(), value.to_string()));
                    }
                }
                
                // Handle qualified names (prefix:local_name)
                let (local_name, element_namespace) = if let Some(colon_pos) = name.find(':') {
                    let prefix = &name[..colon_pos];
                    let local_name = &name[colon_pos + 1..];
                    
                    if let Some(uri) = doc.get_namespace_uri(prefix) {
                        let ns = Namespace::prefixed(uri, prefix.to_string());
                        (local_name.to_string(), Some(ns))
                    } else {
                        (name.to_string(), None)
                    }
                } else {
                    (name.to_string(), namespace)
                };
                
                let element = if let Some(ns) = element_namespace {
                    doc.create_element_with_namespace(local_name, ns)
                } else {
                    doc.create_element(local_name)
                };
                
                for attr in attributes {
                    element.add_attribute(attr);
                }
                
                if let Some(parent) = stack.last() {
                    parent.add_child(element)?;
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

/// Write XML document to a file
pub fn write_file<P: AsRef<Path>>(doc: &Document, path: P) -> XmlResult<()> {
    let file = File::create(path).map_err(|e| XmlError::InvalidXml(format!("Failed to create file: {}", e)))?;
    let writer = BufWriter::new(file);
    write_writer(doc, writer)
}

/// Write XML document to a string
pub fn write_string(doc: &Document) -> XmlResult<String> {
    let mut buffer = Vec::new();
    write_writer(doc, &mut buffer)?;
    String::from_utf8(buffer).map_err(|e| XmlError::InvalidXml(format!("Invalid UTF-8 in output: {}", e)))
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
    
    // Add element attributes - collect them first to avoid lifetime issues
    for attr in element.attributes() {
        attrs.push((attr.name.clone(), attr.value.clone()));
    }
    
    // Create start tag
    let start = BytesStart::new(element.name()).with_attributes(
        attrs.iter().map(|(k, v)| (k.as_bytes(), v.as_bytes())).collect::<Vec<_>>()
    );
    writer.write_event(Event::Start(start))?;
    
    // Write text content or children
    if let Some(text) = element.text_content() {
        if !text.is_empty() {
            let text_event = BytesText::new(&text);
            writer.write_event(Event::Text(text_event))?;
        }
    } else {
        // Write children
        for child in element.children() {
            write_element(writer, &child)?;
        }
    }
    
    // Write end tag
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
        doc.declare_namespace("html".to_string(), "http://www.w3.org/1999/xhtml".to_string());
        
        let html_ns = Namespace::prefixed("http://www.w3.org/1999/xhtml".to_string(), "html".to_string());
        let root = doc.create_element_with_namespace("html".to_string(), html_ns);
        doc.set_root(root.clone()).unwrap();
        
        let head = doc.create_element("head".to_string());
        let title = doc.create_element("title".to_string());
        title.set_text_content("Test Page".to_string());
        head.add_child(title).unwrap();
        root.add_child(head).unwrap();
        
        let output = write_string(&doc).unwrap();
        assert!(output.contains("<html"));
        assert!(output.contains("<head"));
        assert!(output.contains("<title>Test Page</title>"));
    }
} 