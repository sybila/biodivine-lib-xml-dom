//! Book chapter "Namespaces": declarations, scope, and the implicit `xml` prefix.
//!
//! Run with `cargo run --example book_namespaces`.

use biodivine_lib_xml_dom::{Namespace, QualifiedName, parse_string, write_string};

fn main() {
    // Parsing assigns every name its expanded form, resolving prefixes against the declarations
    // that are in scope.
    let document = parse_string(
        r#"<root xmlns:ex="http://example.com" xmlns="http://default"><child ex:attr="v"/></root>"#,
    )
    .unwrap();
    let root = document.root().unwrap();
    let child = root.child_elements()[0].clone();

    assert_eq!(root.qualified_name().to_string(), "root");
    assert_eq!(child.namespace().unwrap().uri(), "http://default");
    assert_eq!(
        child
            .attribute(
                &QualifiedName::with_namespace(
                    "attr",
                    &Namespace::prefixed("http://example.com", "ex").unwrap()
                )
                .unwrap()
            )
            .unwrap()
            .as_ref(),
        "v"
    );

    // Resolution helpers work in the scope of any node.
    assert_eq!(
        child
            .resolve_qualified_name("item")
            .unwrap()
            .namespace()
            .unwrap()
            .uri(),
        "http://default"
    );
    // ... and an unprefixed *attribute* never takes the default namespace.
    assert!(
        child
            .resolve_attribute_name("item")
            .unwrap()
            .namespace()
            .is_none()
    );
    // The `xml` prefix needs no declaration.
    assert_eq!(
        child
            .resolve_attribute_name("xml:lang")
            .unwrap()
            .namespace()
            .unwrap()
            .uri(),
        "http://www.w3.org/XML/1998/namespace"
    );

    // `xmlns=""` removes the default namespace from the element it is declared on.
    let document =
        parse_string(r#"<root xmlns="http://d"><inner xmlns=""><leaf/></inner></root>"#).unwrap();
    let inner = document.root().unwrap().child_elements()[0].clone();
    assert!(inner.namespace().is_none());
    assert!(inner.child_elements()[0].namespace().is_none());
    assert_eq!(inner.namespace_declarations().get(&None), Some(&None));

    // Editing never synchronises declarations for you: removing one that a subtree relies on
    // succeeds silently, and `validate` is what reports the resulting inconsistency.
    let document =
        parse_string(r#"<ex:root xmlns:ex="http://example.com"><ex:child/></ex:root>"#).unwrap();
    let root = document.root().unwrap();
    root.remove_namespace_declaration(Some(&biodivine_lib_xml_dom::xml_spec::nc_name("ex")));
    assert_eq!(document.validate().unwrap_err().len(), 2);
    assert_eq!(
        write_string(&document).unwrap(),
        "<ex:root><ex:child/></ex:root>"
    );
    println!("{}", write_string(&document).unwrap());
}
