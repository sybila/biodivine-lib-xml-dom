//! Book chapter "The Python layer": what the two APIs look like side by side.
//!
//! This is the Rust half of a chapter that also shows the Python layer, so it demonstrates the Rust
//! API that the Pythonic conveniences map onto: iterating children, reading attributes by local
//! name, and validating.
//!
//! Run with `cargo run --example book_pythonic`.

use biodivine_lib_xml_dom::parse_string;

fn main() {
    let document = parse_string(r#"<root><child class="a"/><child class="b"/></root>"#).unwrap();
    let root = document.root().unwrap();

    // The Python layer offers `for child in node` and `node["class"]`; the Rust API spells the same
    // operations as an explicit child list and an attribute lookup by local name.
    let children = root.child_elements();
    assert_eq!(children.len(), 2);
    let classes: Vec<String> = children
        .iter()
        .map(|child| {
            child
                .attribute_local(&biodivine_lib_xml_dom::xml_spec::nc_name("class"))
                .unwrap()
                .to_string()
        })
        .collect();
    assert_eq!(classes, ["a", "b"]);

    // Indexing by position is `children.get(i)`.
    assert_eq!(children.get(1).unwrap().local_name(), "child");

    // Validation is a whole-document call in both languages.
    assert!(document.is_valid());
    println!("{} children", children.len());
}
