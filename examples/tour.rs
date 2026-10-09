//! A guided tour of the library: building, parsing, traversing and editing XML documents.
//!
//! Run with:
//!
//! ```sh
//! cargo run --example tour
//! ```

use biodivine_lib_xml_dom::{
    Document, Namespace, NodeContent, QualifiedName, parse_string, write_string,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("XML DOM library tour");
    println!("====================");

    // --- 1: build a document ------------------------------------------------------------------
    println!("\n1. Building a document programmatically");
    let document = Document::empty();

    let html = Namespace::prefixed("http://www.w3.org/1999/xhtml", "html")?;
    let svg = Namespace::prefixed("http://www.w3.org/2000/svg", "svg")?;

    let root = document.create_element(QualifiedName::with_namespace("html", &html)?);
    root.declare_namespace(html.clone());
    root.declare_namespace(svg.clone());
    document.set_root(root.clone());

    let head = document.create_element(QualifiedName::with_namespace("head", &html)?);
    let title = document.create_element(QualifiedName::with_namespace("title", &html)?);
    title.append_child(document.create_text("My XML Document")?);
    head.append_child(title);
    root.append_child(head);

    let body = document.create_element(QualifiedName::with_namespace("body", &html)?);
    let paragraph = document.create_element(QualifiedName::with_namespace("p", &html)?);
    paragraph.set_attribute(QualifiedName::without_namespace("class")?, "example");
    paragraph.set_attribute(QualifiedName::without_namespace("id")?, "intro");
    paragraph.append_child(
        document.create_text(
            "This document was created with the DOM API rather than parsed from text.",
        )?,
    );
    body.append_child(paragraph);

    let circle = document.create_element(QualifiedName::with_namespace("circle", &svg)?);
    circle.set_attribute(QualifiedName::without_namespace("r")?, "40");
    circle.set_attribute(QualifiedName::without_namespace("fill")?, "blue");
    let svg_element = document.create_element(QualifiedName::with_namespace("svg", &svg)?);
    svg_element.append_child(circle);
    body.append_child(svg_element);
    root.append_child(body);

    println!("{root}");

    // --- 2: parse ---------------------------------------------------------------------------
    println!("\n2. Parsing XML text");
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<bookstore>
    <book category="fiction">
        <title>Harry Potter</title>
        <author>J.K. Rowling</author>
    </book>
    <book category="non-fiction">
        <title>Learning Rust</title>
        <author>Steve Klabnik</author>
    </book>
</bookstore>"#;

    let bookstore = parse_string(xml)?;
    let root = bookstore.root().ok_or("the document has no root element")?;
    println!("root: {}", root.qualified_name());

    for book in root.child_elements() {
        let category = book.attribute(&QualifiedName::without_namespace("category")?);
        let title = book
            .child_elements()
            .into_iter()
            .find(|child| child.local_name() == "title")
            .map(|child| text_of(&child))
            .unwrap_or_default();
        println!(
            "  {} [{}]",
            title,
            category.as_deref().unwrap_or("uncategorised")
        );
    }

    // --- 3: traverse with node contents -----------------------------------------------------
    println!("\n3. Inspecting child nodes");
    let mixed = parse_string(r#"<p>Hello <b>world</b>, welcome!</p>"#)?;
    let paragraph = mixed.root().unwrap();
    for (index, child) in paragraph.children().iter().enumerate() {
        match child.content() {
            NodeContent::Element(element) => {
                println!("  {index}: element <{}>", element.local_name())
            }
            NodeContent::Text(text) => println!("  {index}: text {:?}", text.as_str()),
            NodeContent::Comment(comment) => println!("  {index}: comment {:?}", comment.as_str()),
            NodeContent::CData(cdata) => println!("  {index}: cdata {:?}", cdata.as_str()),
            NodeContent::ProcessingInstruction(target, data) => {
                println!("  {index}: <?{} {}?>", target.as_str(), data.as_str());
            }
        }
    }

    // --- 4: edit an existing document -------------------------------------------------------
    println!("\n4. Editing: move, clone and detach");
    let first_book = root.child_elements()[0].clone();
    let copy = first_book.deep_clone();
    println!("deep clone is detached: {}", !copy.is_attached());
    copy.set_attribute(QualifiedName::without_namespace("category")?, "copy");
    root.append_child(copy);
    println!("bookstore now has {} books", root.child_elements().len());

    let removed = first_book.remove();
    println!(
        "removed {} -> {} books left, removed node still usable: {}",
        removed.local_name(),
        root.child_elements().len(),
        removed.qualified_name()
    );

    // --- 5: copy a subtree into another document --------------------------------------------
    println!("\n5. Copying between documents");
    let other = Document::empty();
    let imported = root.deep_clone_into(&other);
    let _ = other.set_root(imported.clone());
    println!(
        "imported subtree has {} children in the target document",
        imported.child_elements().len()
    );

    // --- 6: a comment-bearing document ------------------------------------------------------
    println!("\n6. Comments, CDATA and processing instructions");
    let annotated = Document::empty();
    let annotated_root = annotated.create_element(QualifiedName::without_namespace("root")?);
    annotated.set_root(annotated_root.clone());
    annotated_root.append_child(annotated.create_comment(" a header comment ")?);
    annotated_root.append_child(annotated.create_cdata("raw <content> & text")?);
    annotated_root.append_child(
        annotated
            .create_processing_instruction("xml-stylesheet", "type=\"text/css\" href=\"a.css\"")?,
    );
    annotated_root.append_child(annotated.create_text("plain text")?);

    let serialized = write_string(&annotated)?;
    println!("{serialized}");
    assert!(serialized.contains("<![CDATA[raw <content> & text]]>"));
    assert!(serialized.contains("<?xml-stylesheet type=\"text/css\" href=\"a.css\"?>"));

    Ok(())
}

/// Concatenates the direct text children of an element.
fn text_of(element: &biodivine_lib_xml_dom::Element) -> String {
    element
        .children()
        .iter()
        .filter_map(|child| child.text())
        .map(|text| text.as_str().to_string())
        .collect()
}
