use biodivine_lib_xml_dom::{create_document, Attribute, Namespace};

fn main() {
    println!("XML DOM Library Example");
    println!("=======================");

    // Create a new XML document
    let doc = create_document();

    // Declare some namespaces
    doc.declare_namespace("html".to_string(), "http://www.w3.org/1999/xhtml".to_string());
    doc.declare_namespace("svg".to_string(), "http://www.w3.org/2000/svg".to_string());

    // Create the root element
    let html_ns = Namespace::prefixed("http://www.w3.org/1999/xhtml".to_string(), "html".to_string());
    let root = doc.create_element_with_namespace("html".to_string(), html_ns);
    
    // Set it as the document root
    doc.set_root(root.clone()).unwrap();

    // Create head element
    let head = doc.create_element("head".to_string());
    root.add_child(head.clone()).unwrap();

    // Create title element with text content
    let title = doc.create_element("title".to_string());
    title.set_text_content("My XML Document".to_string());
    head.add_child(title).unwrap();

    // Create body element
    let body = doc.create_element("body".to_string());
    root.add_child(body.clone()).unwrap();

    // Create a paragraph with attributes
    let p = doc.create_element("p".to_string());
    p.add_attribute(Attribute::new("class".to_string(), "example".to_string()));
    p.add_attribute(Attribute::new("id".to_string(), "intro".to_string()));
    p.set_text_content("This is an example XML document created with our DOM library.".to_string());
    body.add_child(p).unwrap();

    // Create an SVG element with namespace
    let svg_ns = Namespace::prefixed("http://www.w3.org/2000/svg".to_string(), "svg".to_string());
    let svg = doc.create_element_with_namespace("svg".to_string(), svg_ns);
    svg.add_attribute(Attribute::new("width".to_string(), "100".to_string()));
    svg.add_attribute(Attribute::new("height".to_string(), "100".to_string()));
    body.add_child(svg.clone()).unwrap();

    // Create a circle inside SVG
    let circle = doc.create_element("circle".to_string());
    circle.add_attribute(Attribute::new("cx".to_string(), "50".to_string()));
    circle.add_attribute(Attribute::new("cy".to_string(), "50".to_string()));
    circle.add_attribute(Attribute::new("r".to_string(), "40".to_string()));
    circle.add_attribute(Attribute::new("fill".to_string(), "blue".to_string()));
    svg.add_child(circle).unwrap();

    // Print some information about the document
    println!("Document created successfully!");
    println!("Root element: {}", root.qualified_name());
    println!("Number of children in root: {}", root.children().len());
    
    if let Some(body_elem) = root.get_children_by_name("body").first() {
        println!("Body has {} children", body_elem.children().len());
        
        if let Some(p_elem) = body_elem.get_children_by_name("p").first() {
            println!("Paragraph text: {}", p_elem.text_content().unwrap());
            println!("Paragraph attributes: {:?}", p_elem.attributes());
        }
    }

    println!("\nLibrary is ready for use!");
}
