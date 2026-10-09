//! Every documented `# Errors` entry, triggered at least once.
//!
//! The point of this file is to make the error documentation *checked* rather than asserted: for
//! each public fallible method it triggers each documented failure condition and asserts that the
//! documented [`XmlError`] variant is the one that actually comes back. If an implementation starts
//! returning a different (undocumented) variant, these tests fail.

mod common;

use biodivine_lib_xml_dom::xml_spec::nc_name;
use biodivine_lib_xml_dom::{Document, Namespace, QualifiedName, XmlError, parse_string};

use common::element;

/// A bare element with no declarations, used to trigger resolution errors.
fn bare(document: &Document) -> biodivine_lib_xml_dom::Element {
    element(document, "root")
}

/// Asserts that a result is `Err` and matches the expected variant.
fn assert_variant<T: std::fmt::Debug>(
    label: &str,
    result: Result<T, XmlError>,
    expected: &str,
    matches: impl FnOnce(&XmlError) -> bool,
) {
    let error = match result {
        Err(error) => error,
        Ok(value) => panic!("`{label}`: expected an error, but got Ok({value:?})"),
    };
    assert!(
        matches(&error),
        "`{label}`: expected {expected}, got {error:?} ({error})"
    );
}

// ---------------------------------------------------------------------------------------------
// Namespace construction
// ---------------------------------------------------------------------------------------------

#[test]
fn namespace_construction_errors() {
    assert_variant(
        "Namespace::without_prefix('')",
        Namespace::without_prefix(""),
        "InvalidNamespace",
        |error| matches!(error, XmlError::InvalidNamespace(_)),
    );
    assert_variant(
        "Namespace::prefixed('http://example.com'",
        Namespace::prefixed("http://example.com", "1bad"),
        "InvalidName",
        |error| matches!(error, XmlError::InvalidName(_)),
    );
    assert_variant(
        "Namespace::prefixed('http://example.com'",
        Namespace::prefixed("http://example.com", "xmlns"),
        "ReservedPrefix",
        |error| matches!(error, XmlError::ReservedPrefix(_)),
    );
    assert_variant(
        "Namespace::prefixed('http://www.w3.org/XML/1998/namespace'",
        Namespace::prefixed("http://www.w3.org/XML/1998/namespace", "ex"),
        "ReservedPrefix",
        |error| matches!(error, XmlError::ReservedPrefix(_)),
    );
    assert_variant(
        "Namespace::prefixed('http://example.com'",
        Namespace::prefixed("http://example.com", "xml"),
        "ReservedPrefix",
        |error| matches!(error, XmlError::ReservedPrefix(_)),
    );
    assert_variant(
        "Namespace::without_prefix('http://www.w3.org/2000/xmlns/')",
        Namespace::without_prefix("http://www.w3.org/2000/xmlns/"),
        "ReservedPrefix",
        |error| matches!(error, XmlError::ReservedPrefix(_)),
    );
    // The valid cases must still work.
    assert!(Namespace::without_prefix("http://example.com").is_ok());
    assert!(Namespace::prefixed("http://example.com", "ex").is_ok());
}

#[test]
fn qualified_name_construction_errors() {
    assert_variant(
        "QualifiedName::without_namespace('1bad')",
        QualifiedName::without_namespace("1bad"),
        "InvalidName",
        |error| matches!(error, XmlError::InvalidName(_)),
    );
    assert_variant(
        "QualifiedName::without_namespace('a:b')",
        QualifiedName::without_namespace("a:b"),
        "InvalidName",
        |error| matches!(error, XmlError::InvalidName(_)),
    );
    let namespace = Namespace::without_prefix("http://example.com").unwrap();
    assert_variant(
        "QualifiedName::with_namespace(''",
        QualifiedName::with_namespace("", &namespace),
        "InvalidName",
        |error| matches!(error, XmlError::InvalidName(_)),
    );
}

#[test]
fn qualified_name_resolution_errors() {
    let document = Document::empty();
    let root = bare(&document);

    assert_variant(
        "QualifiedName::resolve_element(&root",
        QualifiedName::resolve_element(&root, "nope:item"),
        "UndeclaredPrefix",
        |error| matches!(error, XmlError::UndeclaredPrefix(prefix) if prefix == "nope"),
    );
    assert_variant(
        "QualifiedName::resolve_attribute(&root",
        QualifiedName::resolve_attribute(&root, "nope:item"),
        "UndeclaredPrefix",
        |error| matches!(error, XmlError::UndeclaredPrefix(_)),
    );
    assert_variant(
        "QualifiedName::resolve_element(&root",
        QualifiedName::resolve_element(&root, "xmlns:item"),
        "ReservedPrefix",
        |error| matches!(error, XmlError::ReservedPrefix(_)),
    );
    assert_variant(
        "QualifiedName::resolve_element(&root",
        QualifiedName::resolve_element(&root, "a:b:c"),
        "InvalidName",
        |error| matches!(error, XmlError::InvalidName(_)),
    );
    assert_variant(
        "QualifiedName::resolve_element(&root",
        QualifiedName::resolve_element(&root, "1bad"),
        "InvalidName",
        |error| matches!(error, XmlError::InvalidName(_)),
    );

    // The map-based helpers report the same variants.
    let empty = std::collections::HashMap::new();
    assert_variant(
        "QualifiedName::resolve_element_with_namespace_map('nope:item",
        QualifiedName::resolve_element_with_namespace_map("nope:item", &empty),
        "UndeclaredPrefix",
        |error| matches!(error, XmlError::UndeclaredPrefix(_)),
    );
    assert_variant(
        "QualifiedName::resolve_attribute_with_namespace_map('xmlns:i",
        QualifiedName::resolve_attribute_with_namespace_map("xmlns:item", &empty),
        "ReservedPrefix",
        |error| matches!(error, XmlError::ReservedPrefix(_)),
    );
    assert_variant(
        "QualifiedName::resolve_element_with_namespace_map('a:b:c'",
        QualifiedName::resolve_element_with_namespace_map("a:b:c", &empty),
        "InvalidName",
        |error| matches!(error, XmlError::InvalidName(_)),
    );
}

// ---------------------------------------------------------------------------------------------
// Document
// ---------------------------------------------------------------------------------------------

#[test]
fn node_construction_errors() {
    let document = Document::empty();

    assert_variant(
        "document.create_text('bad \u{1}')",
        document.create_text("bad \u{1}"),
        "InvalidText",
        |error| matches!(error, XmlError::InvalidText(_)),
    );
    assert_variant(
        "document.create_comment('bad -- comment')",
        document.create_comment("bad -- comment"),
        "InvalidComment",
        |error| matches!(error, XmlError::InvalidComment(_)),
    );
    assert_variant(
        "document.create_cdata('bad ]]> content')",
        document.create_cdata("bad ]]> content"),
        "InvalidCData",
        |error| matches!(error, XmlError::InvalidCData(_)),
    );
    assert_variant(
        "document.create_processing_instruction('xml'",
        document.create_processing_instruction("xml", "data"),
        "InvalidProcessingInstruction",
        |error| matches!(error, XmlError::InvalidProcessingInstruction(_)),
    );
    assert_variant(
        "document.create_processing_instruction('target'",
        document.create_processing_instruction("target", "bad ?> content"),
        "InvalidProcessingInstruction",
        |error| matches!(error, XmlError::InvalidProcessingInstruction(_)),
    );
    // Nothing was created.
    assert_eq!(document.node_count(), 0);
}

#[test]
fn set_root_errors() {
    let first = Document::empty();
    let second = Document::empty();
    let foreign = element(&second, "foreign");
    assert_variant(
        "first.set_root_checked(foreign)",
        first.set_root_checked(foreign),
        "ForeignDocument",
        |error| matches!(error, XmlError::ForeignDocument),
    );

    let root = element(&first, "root");
    let child = element(&first, "child");
    root.append_child(child.clone());
    assert_variant(
        "first.set_root_checked(child)",
        first.set_root_checked(child),
        "RootHasParent",
        |error| matches!(error, XmlError::RootHasParent),
    );
    assert!(first.root().is_none());
}

// ---------------------------------------------------------------------------------------------
// Element
// ---------------------------------------------------------------------------------------------

#[test]
fn element_errors() {
    let document = Document::empty();
    let root = element(&document, "root");

    assert_variant(
        "root.set_attribute_checked(QualifiedName::without_namespace(",
        root.set_attribute_checked(QualifiedName::without_namespace("a").unwrap(), "bad \u{1}"),
        "InvalidText",
        |error| matches!(error, XmlError::InvalidText(_)),
    );
    assert!(root.attributes().is_empty());

    let ex = Namespace::prefixed("http://example.com", "ex").unwrap();
    root.declare_namespace(ex.clone());
    assert_variant(
        "root.declare_namespace_checked( Namespace::prefixed('http://",
        root.declare_namespace_checked(Namespace::prefixed("http://other.com", "ex").unwrap()),
        "InvalidNamespace",
        |error| matches!(error, XmlError::InvalidNamespace(_)),
    );
    // The original binding is untouched.
    assert_eq!(root.get_namespace(Some(&nc_name("ex"))), Some(ex));

    assert_variant(
        "root.resolve_qualified_name('nope:item')",
        root.resolve_qualified_name("nope:item"),
        "UndeclaredPrefix",
        |error| matches!(error, XmlError::UndeclaredPrefix(_)),
    );
    assert_variant(
        "root.resolve_attribute_name('xmlns:item')",
        root.resolve_attribute_name("xmlns:item"),
        "ReservedPrefix",
        |error| matches!(error, XmlError::ReservedPrefix(_)),
    );
}

// ---------------------------------------------------------------------------------------------
// Node
// ---------------------------------------------------------------------------------------------

#[test]
fn append_child_errors() {
    let first = Document::empty();
    let second = Document::empty();
    let root = element(&first, "root");
    let child = element(&first, "child");
    let grandchild = element(&first, "grandchild");
    root.append_child(child.clone());
    child.append_child(grandchild.clone());
    first.set_root(root.clone());

    assert_variant(
        "root.append_child_checked(element(&second",
        root.append_child_checked(element(&second, "foreign")),
        "ForeignDocument",
        |error| matches!(error, XmlError::ForeignDocument),
    );
    assert_variant(
        "root.append_child_checked(root.clone())",
        root.append_child_checked(root.clone()),
        "CycleDetected",
        |error| matches!(error, XmlError::CycleDetected),
    );
    assert_variant(
        "grandchild.append_child_checked(child.clone())",
        grandchild.append_child_checked(child.clone()),
        "CycleDetected",
        |error| matches!(error, XmlError::CycleDetected),
    );
    assert_variant(
        "child.append_child_checked(root.clone())",
        child.append_child_checked(root.clone()),
        "CannotAttachRoot",
        |error| matches!(error, XmlError::CannotAttachRoot),
    );
    assert_variant(
        "first .create_text('text') .unwrap() .append_child_checked(c",
        first
            .create_text("text")
            .unwrap()
            .append_child_checked(child.clone()),
        "NotAnElement",
        |error| matches!(error, XmlError::NotAnElement(_)),
    );
}

#[test]
fn insert_child_errors() {
    let document = Document::empty();
    let root = element(&document, "root");
    let child = element(&document, "child");
    root.append_child(child.clone());

    assert_variant(
        "root.insert_child_checked(5",
        root.insert_child_checked(5, element(&document, "extra")),
        "IndexOutOfBounds",
        |error| matches!(error, XmlError::IndexOutOfBounds { index: 5, len: 1 }),
    );
    // In-range indices succeed, so that only the out-of-range case is an error.
    assert!(
        root.insert_child_checked(0, element(&document, "extra"))
            .is_ok()
    );
}

#[test]
fn relative_insertion_errors() {
    let document = Document::empty();
    let root = element(&document, "root");
    let child = element(&document, "child");
    let unrelated = element(&document, "unrelated");

    assert_variant(
        "root.insert_before_checked(unrelated.clone()",
        root.insert_before_checked(unrelated.clone(), child.clone()),
        "NotAChild",
        |error| matches!(error, XmlError::NotAChild(_, _)),
    );
    assert_variant(
        "root.insert_after_checked(unrelated.clone()",
        root.insert_after_checked(unrelated.clone(), child.clone()),
        "NotAChild",
        |error| matches!(error, XmlError::NotAChild(_, _)),
    );
}

#[test]
fn replace_with_errors() {
    let first = Document::empty();
    let second = Document::empty();
    let root = element(&first, "root");
    let parent = element(&first, "parent");
    let child = element(&first, "child");
    root.append_child(parent.clone());
    parent.append_child(child.clone());
    first.set_root(root.clone());

    let detached = element(&first, "detached");
    assert_variant(
        "detached.replace_with_checked(element(&first",
        detached.replace_with_checked(element(&first, "other")),
        "NodeHasNoParent",
        |error| matches!(error, XmlError::NodeHasNoParent(_)),
    );
    assert_variant(
        "child.replace_with_checked(element(&second",
        child.replace_with_checked(element(&second, "foreign")),
        "ForeignDocument",
        |error| matches!(error, XmlError::ForeignDocument),
    );
    assert_variant(
        "child.replace_with_checked(parent.clone())",
        child.replace_with_checked(parent.clone()),
        "CycleDetected",
        |error| matches!(error, XmlError::CycleDetected),
    );
    assert_variant(
        "child.replace_with_checked(root.clone())",
        child.replace_with_checked(root.clone()),
        "CannotAttachRoot",
        |error| matches!(error, XmlError::CannotAttachRoot),
    );
    // Replacing a node by itself is documented as a successful no-op.
    assert!(child.replace_with_checked(child.clone()).is_ok());
}

// ---------------------------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------------------------

#[test]
fn parse_errors() {
    assert_variant(
        "parse_string('<a a='1' a='2'/>')",
        parse_string("<a a='1' a='2'/>"),
        "DuplicateAttribute",
        |error| matches!(error, XmlError::DuplicateAttribute(_)),
    );
    assert_variant(
        "parse_string('<a><b></a>')",
        parse_string("<a><b></a>"),
        "MalformedXml",
        |error| matches!(error, XmlError::MalformedXml(_)),
    );
    assert_variant(
        "parse_string('<a xmlns:p=''/>')",
        parse_string("<a xmlns:p=''/>"),
        "InvalidNamespace",
        |error| matches!(error, XmlError::InvalidNamespace(_)),
    );
    assert_variant(
        "parse_string('<p:a/>')",
        parse_string("<p:a/>"),
        "UndeclaredPrefix",
        |error| matches!(error, XmlError::UndeclaredPrefix(_)),
    );
    assert_variant(
        "parse_string('<a xmlns:xmlns='http://example.com'/>')",
        parse_string("<a xmlns:xmlns='http://example.com'/>"),
        "ReservedPrefix",
        |error| matches!(error, XmlError::ReservedPrefix(_)),
    );
    assert_variant(
        "parse_string('<a><!-- bad -- comment --></a>')",
        parse_string("<a><!-- bad -- comment --></a>"),
        "InvalidComment",
        |error| matches!(error, XmlError::InvalidComment(_)),
    );
    assert_variant(
        "parse_string('<1bad/>')",
        parse_string("<1bad/>"),
        "InvalidName",
        |error| matches!(error, XmlError::InvalidName(_)),
    );
}

#[test]
fn parse_file_reports_io_errors() {
    assert_variant(
        "biodivine_lib_xml_dom::parse_file('/nonexistent/path/to/file",
        biodivine_lib_xml_dom::parse_file("/nonexistent/path/to/file.xml"),
        "Io",
        |error| matches!(error, XmlError::Io(_)),
    );
    assert_variant(
        "biodivine_lib_xml_dom::write_file(&Document::empty()",
        biodivine_lib_xml_dom::write_file(&Document::empty(), "/nonexistent/dir/file.xml"),
        "Io",
        |error| matches!(error, XmlError::Io(_)),
    );
}

/// The two parser gaps the G2 audit found are now errors (fixed in G3).
///
/// This test used to be a characterisation test that pinned the *wrong* behaviour, precisely so
/// that the fix could not happen silently. It is now the positive counterpart; the corresponding
/// rows in `docs/design/REVIEW.md` (D8) record the fix.
#[test]
fn unclosed_elements_and_xml_target_pis_are_rejected() {
    assert_variant(
        "parse_string('<a><b></a>')",
        parse_string("<a><b></a>"),
        "MalformedXml",
        |error| matches!(error, XmlError::MalformedXml(_)),
    );
    assert_variant(
        "parse_string('<a>')",
        parse_string("<a>"),
        "MalformedXml",
        |error| matches!(error, XmlError::MalformedXml(_)),
    );
    assert_variant(
        "parse_string('<a><?xml target?></a>')",
        parse_string("<a><?xml target?></a>"),
        "InvalidProcessingInstruction",
        |error| matches!(error, XmlError::InvalidProcessingInstruction(_)),
    );
}
