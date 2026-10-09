//! The XML serializer.
//!
//! # Fidelity
//!
//! Serialization is intended to be *lossless* for everything the data model can represent:
//!
//! * element and attribute names are written **with their prefix**, so
//!   `<html:body>` stays `<html:body>` (this was a data-loss bug in the pre-rewrite
//!   implementation, REVIEW D2);
//! * namespace declarations are written exactly as they are stored on the nodes. The serializer
//!   never adds, removes or rewrites one — an inconsistent document stays inconsistent and is
//!   reported by whole-document validation instead (requirement (3));
//! * text is escaped so that `&`, `<`, ``]]>`` and a literal carriage return survive a round trip;
//! * attribute values are escaped so that `&`, `<`, `"` survive, and tab, line feed and carriage
//!   return are written as character references, because attribute-value normalisation would
//!   otherwise turn them into spaces (XML 1.0 §3.3.3);
//! * comments, CDATA sections and processing instructions are written verbatim. Their content
//!   cannot contain an escape sequence, so line ends are normalised when they are *constructed*
//!   (see [`crate::xml_spec`]), which makes the stored value exactly what a re-parse produces;
//! * adjacent text nodes are merged, so the output does not depend on how the tree was built.
//!
//! # Namespaces
//!
//! The serializer is deliberately not "smart": it writes the prefixes and declarations that are
//! stored in the tree. A document whose names use a prefix that is not declared anywhere is
//! serialized as such, and re-parsing it fails. That is the contract required by the task
//! description — run [`crate::Document::validate`] (or `cargo doc`-documented validation once it
//! lands) before serializing a document you assembled by hand.
//!
//! # Depth
//!
//! Serialization is iterative (an explicit work stack, not recursion), so a deeply nested document
//! cannot overflow the stack: a stack overflow aborts the process and could not be reported as a
//! typed error (see `docs/design/PLAN.md` §16, advisor condition C2).

use quick_xml::Writer;
use quick_xml::events::{BytesCData, BytesDecl, BytesEnd, BytesPI, BytesStart, BytesText, Event};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::document::Document;
use crate::element::Element;
use crate::error::{XmlError, XmlResult};
use crate::node::{Node, NodeContent};
use crate::qualified_name::QualifiedName;
use crate::xml_spec::XmlDeclaration;

/// When to write the `<?xml …?>` declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DeclarationStyle {
    /// Never write a declaration.
    Never,
    /// Write a declaration if the document has one (the default).
    ///
    /// A document gets one when it is parsed from input that had one, or when
    /// [`Document::set_xml_declaration`] is called explicitly. This is what makes
    /// `write(parse(x))` preserve the declaration.
    #[default]
    IfPresent,
    /// Always write a declaration, using [`XmlDeclaration::utf8`] when the document has none.
    Always,
}

/// How to write an element that has no children.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EmptyElementStyle {
    /// `<a/>` (the default).
    #[default]
    SelfClosing,
    /// `<a></a>`.
    ExplicitEndTag,
}

/// Options for [`write_string_with`], [`write_file_with`] and [`write_writer_with`].
///
/// [`WriteOptions::default`] is "write the declaration when the document has one" plus
/// "self-closing empty elements", which reproduces the input of a parsed document as closely as
/// the data model allows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WriteOptions {
    /// When to write the XML declaration.
    pub declaration: DeclarationStyle,
    /// How to write elements without children.
    pub empty_elements: EmptyElementStyle,
}

/// Serializes an XML document into a string, using the default options.
///
/// A document without a root element and without a declaration serializes to the empty string.
///
/// # Errors
///
/// Returns [`XmlError::InvalidUtf8`] if the serialized output is not valid UTF-8. This cannot
/// happen for a document built from `str` (every payload of the data model is valid UTF-8 by
/// construction), so the variant is reported rather than panicking.
pub fn write_string(doc: &Document) -> XmlResult<String> {
    write_string_with(doc, &WriteOptions::default())
}

/// Serializes an XML document into a string with explicit options.
///
/// # Errors
///
/// As [`write_string`].
pub fn write_string_with(doc: &Document, options: &WriteOptions) -> XmlResult<String> {
    let mut buffer = Vec::new();
    write_writer_with(doc, &mut buffer, options)?;
    String::from_utf8(buffer)
        .map_err(|error| XmlError::InvalidUtf8(format!("invalid UTF-8 in output: {error}")))
}

/// Writes an XML document to a file, using the default options.
///
/// # Errors
///
/// Returns [`XmlError::Io`] if the file cannot be created or written, plus the errors of
/// [`write_writer_with`].
pub fn write_file<P: AsRef<Path>>(doc: &Document, path: P) -> XmlResult<()> {
    write_file_with(doc, path, &WriteOptions::default())
}

/// Writes an XML document to a file with explicit options.
///
/// # Errors
///
/// As [`write_file`].
pub fn write_file_with<P: AsRef<Path>>(
    doc: &Document,
    path: P,
    options: &WriteOptions,
) -> XmlResult<()> {
    let file = File::create(path)?;
    write_writer_with(doc, BufWriter::new(file), options)
}

/// Writes an XML document to a generic writer, using the default options.
///
/// # Errors
///
/// Returns [`XmlError::Io`] if the underlying writer fails.
pub fn write_writer<W: Write>(doc: &Document, writer: W) -> XmlResult<()> {
    write_writer_with(doc, writer, &WriteOptions::default())
}

/// Writes an XML document to a generic writer with explicit options.
///
/// # Errors
///
/// Returns [`XmlError::Io`] if the underlying writer fails.
pub fn write_writer_with<W: Write>(
    doc: &Document,
    writer: W,
    options: &WriteOptions,
) -> XmlResult<()> {
    let mut writer = Writer::new(writer);
    write_declaration(&mut writer, doc, options)?;
    if let Some(root) = doc.root() {
        write_subtree(&mut writer, &root.node(), options)?;
    }
    Ok(())
}

/// Serializes one subtree (an element or any other node kind) to a string.
///
/// Used by the `Display` implementations of [`Element`] and [`Node`]; the declaration is never
/// written, because a subtree is not a document.
///
/// # Panics
///
/// Writing into an in-memory buffer cannot fail; the intermediate `Result` is unwrapped with a
/// message saying so.
pub(crate) fn write_element_to_string(element: &Element) -> String {
    let mut buffer = Vec::new();
    let mut writer = Writer::new(&mut buffer);
    write_subtree(&mut writer, &element.node(), &WriteOptions::default())
        .expect("writing XML into a memory buffer cannot fail");
    String::from_utf8(buffer).expect("the serializer only ever emits UTF-8")
}

/// Writes the `<?xml …?>` declaration according to `options`.
fn write_declaration<W: Write>(
    writer: &mut Writer<W>,
    document: &Document,
    options: &WriteOptions,
) -> XmlResult<()> {
    let stored = document.xml_declaration();
    let (write, declaration) = match options.declaration {
        DeclarationStyle::Never => (false, None),
        DeclarationStyle::IfPresent => (stored.is_some(), stored),
        DeclarationStyle::Always => (true, stored.or_else(|| Some(XmlDeclaration::utf8()))),
    };
    let Some(declaration) = declaration else {
        debug_assert!(!write);
        return Ok(());
    };
    if !write {
        return Ok(());
    }
    let content = format!("xml {}", declaration.pseudo_attributes());
    // `name_len` is the length of `xml`, which `BytesDecl` uses to expose the target.
    writer.write_event(Event::Decl(BytesDecl::from_start(
        BytesStart::from_content(content, 3),
    )))?;
    Ok(())
}

/// One step of the iterative subtree traversal.
enum Task {
    /// Write the node (start tag, leaf content, or an empty-element tag).
    Enter(Node),
    /// Write the end tag of an element whose name is carried here.
    Exit(String),
    /// Write one text node whose content is the concatenation of a run of adjacent text children.
    Text(String),
}

/// Writes a subtree iteratively.
fn write_subtree<W: Write>(
    writer: &mut Writer<W>,
    root: &Node,
    options: &WriteOptions,
) -> XmlResult<()> {
    let mut tasks: Vec<Task> = vec![Task::Enter(root.clone())];
    let mut buffer = String::new();

    while let Some(task) = tasks.pop() {
        match task {
            Task::Exit(name) => {
                writer.write_event(Event::End(BytesEnd::new(name)))?;
            }
            Task::Text(text) => {
                buffer.clear();
                escape_text(&text, &mut buffer);
                if !buffer.is_empty() {
                    writer.write_event(Event::Text(BytesText::from_escaped(buffer.as_str())))?;
                }
            }
            Task::Enter(node) => {
                buffer.clear();
                match node.content() {
                    NodeContent::Element(element) => {
                        let name = write_start_tag(&mut buffer, &element);
                        let mut pending_tasks: Vec<Task> = Vec::new();
                        push_children(&element, &mut pending_tasks);
                        let empty = pending_tasks.is_empty();
                        let self_closing =
                            empty && options.empty_elements == EmptyElementStyle::SelfClosing;
                        if self_closing {
                            writer.write_event(Event::Empty(BytesStart::from_content(
                                buffer.as_str(),
                                name.len(),
                            )))?;
                        } else {
                            writer.write_event(Event::Start(BytesStart::from_content(
                                buffer.as_str(),
                                name.len(),
                            )))?;
                            tasks.push(Task::Exit(name));
                            tasks.append(&mut pending_tasks);
                        }
                    }
                    NodeContent::Text(text) => {
                        escape_text(text.as_str(), &mut buffer);
                        if !buffer.is_empty() {
                            writer.write_event(Event::Text(BytesText::from_escaped(
                                buffer.as_str(),
                            )))?;
                        }
                    }
                    NodeContent::Comment(comment) => {
                        writer.write_event(Event::Comment(BytesText::from_escaped(
                            comment.as_str(),
                        )))?;
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
                        writer.write_event(Event::PI(BytesPI::new(content)))?;
                    }
                }
            }
        }
    }
    Ok(())
}

/// Pushes the children of `element` onto `tasks`, in document order.
///
/// Runs of adjacent text nodes are concatenated into a single [`Task::Text`]. The data model allows
/// two adjacent text nodes (a caller can create them), but XML has no way to express the boundary,
/// so writing them separately would produce output that re-parses into a *different* tree. Merging
/// them makes `write → parse` idempotent and the output independent of how the tree was built.
fn push_children(element: &Element, tasks: &mut Vec<Task>) {
    let mut grouped: Vec<Task> = Vec::new();
    let mut pending_text = String::new();
    for child in element.children() {
        match child.text() {
            Some(text) => pending_text.push_str(text.as_str()),
            None => {
                if !pending_text.is_empty() {
                    grouped.push(Task::Text(std::mem::take(&mut pending_text)));
                }
                grouped.push(Task::Enter(child));
            }
        }
    }
    if !pending_text.is_empty() {
        grouped.push(Task::Text(pending_text));
    }
    for task in grouped.into_iter().rev() {
        tasks.push(task);
    }
}

/// Writes the start tag of `element` (name plus namespace declarations plus attributes) into
/// `buffer`, and returns the tag name.
///
/// The tag is built as text and handed to `quick-xml` verbatim, so this function is the single
/// place that decides how names and values appear in the output.
fn write_start_tag(buffer: &mut String, element: &Element) -> String {
    let name = element.qualified_name();
    let tag_name = qualified_name_to_string(&name);
    buffer.push_str(&tag_name);

    for (prefix, namespace) in element.namespace_declarations() {
        match (prefix, namespace) {
            (Some(prefix), Some(namespace)) => {
                push_attribute(buffer, &format!("xmlns:{prefix}"), namespace.uri());
            }
            (None, Some(namespace)) => push_attribute(buffer, "xmlns", namespace.uri()),
            // An empty default declaration: `xmlns=""`.
            // rule: rule.namespace-usage.empty-default-namespace.md
            (None, None) => push_attribute(buffer, "xmlns", ""),
            // `xmlns:prefix=""` cannot be stored (the parser rejects it), so it cannot occur.
            (Some(_), None) => {}
        }
    }
    for (attribute, value) in element.attributes() {
        push_attribute(
            buffer,
            &qualified_name_to_string(&attribute),
            value.as_ref(),
        );
    }
    tag_name
}

/// Appends ` name="value"` to `buffer`, escaping the value.
fn push_attribute(buffer: &mut String, name: &str, value: &str) {
    buffer.push(' ');
    buffer.push_str(name);
    buffer.push_str("=\"");
    escape_attribute(value, buffer);
    buffer.push('"');
}

/// Renders a qualified name with its prefix, e.g. `html:body`.
fn qualified_name_to_string(name: &QualifiedName) -> String {
    match name.namespace().and_then(|namespace| namespace.prefix()) {
        Some(prefix) => format!("{prefix}:{}", name.local_name()),
        None => name.local_name().to_string(),
    }
}

/// Escapes text content.
///
/// `&` and `<` must be escaped (`rule.well-formedness.escape-ampersand-and-lt`). `>` is escaped
/// too, so that a `]]>` can never be formed — the `CharData` production excludes it. A literal
/// carriage return is written as `&#xD;`, because the processor's mandatory line-end normalisation
/// (`rule.document-structure.processor-must-normalize-line-breaks`) would otherwise turn it into a
/// line feed on the next parse, making the round trip lossy. Tab and line feed need no escaping in
/// content.
fn escape_text(input: &str, buffer: &mut String) {
    for character in input.chars() {
        match character {
            '&' => buffer.push_str("&amp;"),
            '<' => buffer.push_str("&lt;"),
            '>' => buffer.push_str("&gt;"),
            '\r' => buffer.push_str("&#xD;"),
            other => buffer.push(other),
        }
    }
}

/// Escapes an attribute value for a double-quoted attribute.
///
/// `&`, `<` and `"` must be escaped (`rule.attributes.no-lt-in-values`,
/// `rule.well-formedness.escape-ampersand-and-lt`). Tab, line feed and carriage return are written
/// as character references because attribute-value normalisation replaces a *literal* whitespace
/// character with a space (XML 1.0 §3.3.3,
/// `rule.attributes.values-must-be-normalized`); a character reference is not affected by that
/// normalisation, which is what makes `a="x&#x9;y"` round-trip byte-exactly while `a="x\ty"`
/// becomes `x y`.
fn escape_attribute(input: &str, buffer: &mut String) {
    for character in input.chars() {
        match character {
            '&' => buffer.push_str("&amp;"),
            '<' => buffer.push_str("&lt;"),
            '"' => buffer.push_str("&quot;"),
            '\t' => buffer.push_str("&#x9;"),
            '\n' => buffer.push_str("&#xA;"),
            '\r' => buffer.push_str("&#xD;"),
            other => buffer.push(other),
        }
    }
}
