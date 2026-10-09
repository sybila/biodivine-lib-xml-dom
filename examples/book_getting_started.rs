//! Book chapter "Getting started": build a document, give it a root, and serialize it.
//!
//! Run with `cargo run --example book_getting_started`. The assertions are what makes this file a
//! test as well as an example: the book includes this file verbatim, so a chapter cannot show code
//! that no longer works.

use biodivine_lib_xml_dom::{Document, Namespace, QualifiedName, write_string};

fn main() {
    let document = Document::empty();
    assert!(document.root().is_none());

    let ex = Namespace::prefixed("http://example.com", "ex").unwrap();
    let root = document.create_element(QualifiedName::with_namespace("root", &ex).unwrap());
    root.declare_namespace(ex.clone());
    document.set_root(root.clone());

    let child = document.create_element(QualifiedName::with_namespace("child", &ex).unwrap());
    child.set_attribute(QualifiedName::without_namespace("id").unwrap(), "first");
    child.append_child(document.create_text("Hello, World!").unwrap());
    root.append_child(child);

    assert_eq!(
        write_string(&document).unwrap(),
        r#"<ex:root xmlns:ex="http://example.com"><ex:child id="first">Hello, World!</ex:child></ex:root>"#
    );
    assert!(document.is_valid());

    // Element and attribute names are *expanded* names: the prefix is part of the namespace, not of
    // the local name.
    let child = &root.child_elements()[0];
    assert_eq!(child.local_name(), "child");
    assert_eq!(child.namespace().unwrap().uri(), "http://example.com");
    println!("{}", write_string(&document).unwrap());
}
