use biodivine_lib_xml_dom::{
    create_document, parse_string, write_string, Namespace, QualifiedName,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("XML DOM Library Example");
    println!("=======================");

    // Example 1: Create XML programmatically
    println!("\n1. Creating XML programmatically:");
    let doc = create_document();

    let html_ns = Namespace::prefixed("http://www.w3.org/1999/xhtml", "html").unwrap();
    let root = doc.create_element(QualifiedName::with_namespace("html", &html_ns).unwrap());

    // Declare namespaces on the root element
    root.declare_namespace("html".to_string(), html_ns.clone());
    let svg_ns = Namespace::prefixed("http://www.w3.org/2000/svg", "svg").unwrap();
    root.declare_namespace("svg".to_string(), svg_ns.clone());

    doc.set_root(root.clone())?;

    let head = doc.create_element(QualifiedName::without_namespace("head").unwrap());
    let title = doc.create_element(QualifiedName::without_namespace("title").unwrap());
    title.add_text("My XML Document".to_string());
    head.add_child_element(title)?;
    root.add_child_element(head.clone())?;

    let body = doc.create_element(QualifiedName::without_namespace("body").unwrap());
    let p = doc.create_element(QualifiedName::without_namespace("p").unwrap());
    p.add_attribute(
        QualifiedName::without_namespace("class").unwrap(),
        "example".to_string(),
    );
    p.add_attribute(
        QualifiedName::without_namespace("id").unwrap(),
        "intro".to_string(),
    );
    p.add_text("This is an example XML document created with our DOM library.".to_string());
    body.add_child_element(p)?;
    root.add_child_element(body.clone())?;

    let svg = doc.create_element(QualifiedName::with_namespace("svg", &svg_ns).unwrap());
    svg.add_attribute(
        QualifiedName::without_namespace("width").unwrap(),
        "100".to_string(),
    );
    svg.add_attribute(
        QualifiedName::without_namespace("height").unwrap(),
        "100".to_string(),
    );
    body.add_child_element(svg.clone())?;

    let circle = doc.create_element(QualifiedName::without_namespace("circle").unwrap());
    circle.add_attribute(
        QualifiedName::without_namespace("cx").unwrap(),
        "50".to_string(),
    );
    circle.add_attribute(
        QualifiedName::without_namespace("cy").unwrap(),
        "50".to_string(),
    );
    circle.add_attribute(
        QualifiedName::without_namespace("r").unwrap(),
        "40".to_string(),
    );
    circle.add_attribute(
        QualifiedName::without_namespace("fill").unwrap(),
        "blue".to_string(),
    );
    svg.add_child_element(circle)?;

    let xml_output = write_string(&doc)?;
    println!("Generated XML:");
    println!("{}", xml_output);

    // Example 2: Parse XML from string
    println!("\n2. Parsing XML from string:");
    let xml_input = r#"<?xml version="1.0" encoding="UTF-8"?>
<bookstore>
    <book category="fiction">
        <title>Harry Potter</title>
        <author>J.K. Rowling</author>
        <year>1997</year>
        <price>29.99</price>
    </book>
    <book category="non-fiction">
        <title>Learning Rust</title>
        <author>Steve Klabnik</author>
        <year>2018</year>
        <price>39.99</price>
    </book>
</bookstore>"#;

    let parsed_doc = parse_string(xml_input)?;
    let root = parsed_doc
        .root()
        .ok_or("Parsed document has no root element")?;
    println!("Parsed document root: {}", root.name());
    let books: Vec<_> = root.element_children();
    println!("Number of books: {}", books.len());

    for book in books {
        println!("  - Book: {}", book.name());
        let children = book.element_children();
        let title = children.iter().find(|e| e.name() == "title").unwrap();
        println!("    Title: {}", title.text_children().join(""));
    }

    // Example 3: Working with comments
    println!("\n3. Working with XML comments:");
    let xml_with_comments = r#"<?xml version="1.0" encoding="UTF-8"?>
<document>
    <!-- This is a header comment -->
    <header>
        <title>My Document</title>
        <!-- Author information -->
        <author>John Doe</author>
    </header>
    <!-- Main content section -->
    <content>
        <p>This is the main content.</p>
        <!-- TODO: Add more content here -->
    </content>
</document>"#;

    let comment_doc = parse_string(xml_with_comments)?;
    let comment_root = comment_doc.root().unwrap();

    println!("Parsed document with comments:");
    println!("Root element: {}", comment_root.name());

    // Get all comments
    let comments = comment_root.comment_children();
    println!("Comments in root: {:?}", comments);

    // Get comments from child elements
    let header_children = comment_root.element_children();
    let header = header_children
        .iter()
        .find(|e| e.name() == "header")
        .unwrap();
    let header_comments = header.comment_children();
    println!("Comments in header: {:?}", header_comments);

    let content_children = comment_root.element_children();
    let content = content_children
        .iter()
        .find(|e| e.name() == "content")
        .unwrap();
    let content_comments = content.comment_children();
    println!("Comments in content: {:?}", content_comments);

    // Create a new document with comments
    println!("\n4. Creating document with comments:");
    let new_doc = create_document();
    let new_root = new_doc.create_element(QualifiedName::without_namespace("root").unwrap());
    new_doc.set_root(new_root.clone())?;

    new_root.add_comment(" This is a programmatically added comment ".to_string());
    new_root.add_text("Some text content".to_string());

    let child = new_doc.create_element(QualifiedName::without_namespace("child").unwrap());
    child.add_comment(" Comment inside child element ".to_string());
    child.add_text("Child content".to_string());
    new_root.add_child_element(child)?;

    new_root.add_comment(" Final comment ".to_string());

    let new_xml = write_string(&new_doc)?;
    println!("Generated XML with comments:");
    println!("{}", new_xml);

    Ok(())
}
