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
    root.declare_namespace(
        "html".to_string(),
        "http://www.w3.org/1999/xhtml".to_string(),
    );
    root.declare_namespace("svg".to_string(), "http://www.w3.org/2000/svg".to_string());

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

    let svg_ns = Namespace::prefixed("http://www.w3.org/2000/svg", "svg").unwrap();
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

    for (i, book) in books.iter().enumerate() {
        let category = book
            .get_attribute(&QualifiedName::without_namespace("category").unwrap())
            .ok_or("Book element missing 'category' attribute")?;
        println!("Book {}: {}", i + 1, category);
        let titles: Vec<_> = book
            .element_children()
            .into_iter()
            .filter(|e| e.name() == "title")
            .collect();
        if let Some(title) = titles.first() {
            let text = title.text_children().join("");
            println!("  Title: {}", text);
        }
    }

    // Example 3: Round-trip test
    println!("\n3. Round-trip test (parse -> modify -> write):");
    let round_trip_xml = write_string(&parsed_doc)?;
    let round_trip_doc = parse_string(&round_trip_xml)?;
    println!(
        "Round-trip XML root: {}",
        round_trip_doc.root().unwrap().name()
    );

    Ok(())
}
