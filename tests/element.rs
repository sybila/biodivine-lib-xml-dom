//! The element and namespace API (requirement (3)).
//!
//! The contract under test is "no magic": names are *expanded* names that carry their namespace,
//! and no editing operation ever adds, removes or rewrites a namespace declaration. Making the
//! document consistent again is the job of whole-document validation, not of the editing API, so
//! deleting a declaration that children still rely on is allowed and silent.

mod common;

use biodivine_lib_xml_dom::xml_spec::nc_name;
use biodivine_lib_xml_dom::{
    Document, Namespace, QualifiedName, XmlError, parse_string, write_string,
};

use common::element;

/// A namespaced name built from `(uri, prefix, local)`.
fn name(uri: &str, prefix: &str, local: &str) -> QualifiedName {
    QualifiedName::with_namespace(local, &Namespace::prefixed(uri, prefix).unwrap()).unwrap()
}

#[test]
fn names_accessors_agree() {
    let document = Document::empty();
    let ex = Namespace::prefixed("http://example.com", "ex").unwrap();
    let element = document.create_element(QualifiedName::with_namespace("item", &ex).unwrap());

    assert_eq!(element.local_name(), "item");
    assert_eq!(element.namespace().unwrap().uri(), "http://example.com");
    assert_eq!(element.namespace().unwrap().prefix_str(), Some("ex"));
    assert_eq!(element.qualified_name().to_string(), "ex:item");

    element.set_qualified_name(QualifiedName::without_namespace("plain").unwrap());
    assert_eq!(element.local_name(), "plain");
    assert!(element.namespace().is_none());
    assert_eq!(element.qualified_name().to_string(), "plain");
}

#[test]
fn attribute_crud_is_keyed_by_expanded_name() {
    let document = Document::empty();
    let element = element(&document, "root");

    let plain = QualifiedName::without_namespace("attr").unwrap();
    let namespaced = name("http://example.com", "ex", "attr");
    let other_namespace = name("http://other.com", "o", "attr");

    element.set_attribute(plain.clone(), "plain");
    element.set_attribute(namespaced.clone(), "namespaced");
    element.set_attribute(other_namespace.clone(), "other");
    assert_eq!(element.attributes().len(), 3);

    assert_eq!(element.attribute(&plain).unwrap().as_ref(), "plain");
    assert_eq!(
        element.attribute_local(&nc_name("attr")).unwrap().as_ref(),
        "plain"
    );
    assert_eq!(
        element.attribute(&namespaced).unwrap().as_ref(),
        "namespaced"
    );
    assert!(element.has_attribute(&other_namespace));

    // Setting an existing expanded name overwrites, which is the documented default behaviour.
    element.set_attribute(plain.clone(), "updated");
    assert_eq!(element.attribute(&plain).unwrap().as_ref(), "updated");
    assert_eq!(element.attributes().len(), 3);

    assert_eq!(
        element.remove_attribute(&plain).unwrap().as_ref(),
        "updated"
    );
    assert!(element.attribute(&plain).is_none());
    assert!(element.remove_attribute(&plain).is_none());

    element.clear_attributes();
    assert!(element.attributes().is_empty());
}

#[test]
fn invalid_attribute_values_are_rejected() {
    let document = Document::empty();
    let element = element(&document, "root");
    let plain = QualifiedName::without_namespace("attr").unwrap();

    assert!(matches!(
        element.set_attribute_checked(plain.clone(), "control \u{1}"),
        Err(XmlError::InvalidText(_))
    ));
    assert!(element.attributes().is_empty());

    // Markup characters are fine: the serializer escapes them.
    element.set_attribute(plain, "a < b & c");
    assert_eq!(
        element.attribute_local(&nc_name("attr")).unwrap().as_ref(),
        "a < b & c"
    );
}

#[test]
fn namespace_declarations_can_be_added_changed_and_removed() {
    let document = Document::empty();
    let root = element(&document, "root");
    let ex = Namespace::prefixed("http://example.com", "ex").unwrap();
    let default = Namespace::without_prefix("http://default").unwrap();

    root.declare_namespace(ex.clone());
    root.declare_namespace(default.clone());

    let declarations = root.namespace_declarations();
    assert_eq!(declarations.len(), 2);
    assert_eq!(
        declarations.get(&Some(nc_name("ex"))),
        Some(&Some(ex.clone()))
    );
    assert_eq!(declarations.get(&None), Some(&Some(default.clone())));

    // Overwriting is allowed and documented ...
    let other = Namespace::prefixed("http://other.com", "ex").unwrap();
    root.declare_namespace(other.clone());
    assert_eq!(
        root.get_namespace(Some(&nc_name("ex"))),
        Some(other.clone())
    );

    // ... whereas the checked variant refuses to change an existing binding.
    assert!(matches!(
        root.declare_namespace_checked(ex.clone()),
        Err(XmlError::InvalidNamespace(_))
    ));
    assert_eq!(
        root.get_namespace(Some(&nc_name("ex"))),
        Some(other.clone())
    );
    // Re-declaring the identical binding is a no-op.
    assert!(root.declare_namespace_checked(other.clone()).is_ok());

    assert_eq!(
        root.remove_namespace_declaration(Some(&nc_name("ex"))),
        Some(Some(other))
    );
    assert!(
        root.remove_namespace_declaration(Some(&nc_name("ex")))
            .is_none()
    );
    assert_eq!(root.namespace_declarations().len(), 1);
}

#[test]
fn declarations_are_inherited_and_shadowed() {
    let document = Document::empty();
    let outer = element(&document, "outer");
    let inner = element(&document, "inner");
    let deeper = element(&document, "deeper");
    outer.append_child(inner.clone());
    inner.append_child(deeper.clone());

    let outer_ns = Namespace::prefixed("http://outer", "ex").unwrap();
    outer.declare_namespace(outer_ns.clone());
    let default = Namespace::without_prefix("http://default").unwrap();
    outer.declare_namespace(default.clone());

    // Inheritance.
    assert_eq!(
        inner.get_namespace(Some(&nc_name("ex"))),
        Some(outer_ns.clone())
    );
    assert_eq!(deeper.get_namespace(None), Some(default.clone()));

    // Shadowing.
    let inner_ns = Namespace::prefixed("http://inner", "ex").unwrap();
    inner.declare_namespace(inner_ns.clone());
    assert_eq!(
        inner.get_namespace(Some(&nc_name("ex"))),
        Some(inner_ns.clone())
    );
    assert_eq!(
        deeper.get_namespace(Some(&nc_name("ex"))),
        Some(inner_ns.clone())
    );
    // The outer element still sees its own binding.
    assert_eq!(
        outer.get_namespace(Some(&nc_name("ex"))),
        Some(outer_ns.clone())
    );
    // And the outer one is still in scope, just shadowed.
    let scope = deeper.namespaces_in_scope();
    let innermost = scope
        .iter()
        .find(|(prefix, _)| {
            prefix
                .as_ref()
                .is_some_and(|prefix| prefix.as_str() == "ex")
        })
        .expect("`ex` must be in scope");
    assert_eq!(innermost.1, Some(inner_ns));

    // An unknown prefix is simply not bound.
    assert_eq!(inner.get_namespace(Some(&nc_name("nope"))), None);
}

#[test]
fn an_empty_default_declaration_removes_the_default_namespace() {
    let document = Document::empty();
    let outer = element(&document, "outer");
    let inner = element(&document, "inner");
    outer.append_child(inner.clone());

    let default = Namespace::without_prefix("http://default").unwrap();
    outer.declare_namespace(default.clone());
    inner.undeclare_default_namespace();

    assert_eq!(outer.get_namespace(None), Some(default));
    assert_eq!(inner.get_namespace(None), None);
    assert_eq!(
        inner.namespace_declarations().get(&None),
        Some(&None),
        "the empty declaration must be recorded explicitly"
    );
}

#[test]
fn removing_a_declaration_is_silent_even_though_the_subtree_relies_on_it() {
    // Requirement (3): editing never checks namespace integrity; validation reports it instead.
    let document = Document::empty();
    let root = element(&document, "root");
    let child = document.create_element(name("http://example.com", "ex", "item"));
    root.append_child(child.clone());
    document.set_root(root.clone());

    let ex = Namespace::prefixed("http://example.com", "ex").unwrap();
    root.declare_namespace(ex);
    assert_eq!(
        child.get_namespace(Some(&nc_name("ex"))),
        Some(Namespace::prefixed("http://example.com", "ex").unwrap())
    );

    root.remove_namespace_declaration(Some(&nc_name("ex")));
    // Nothing complains, and the child keeps its (now undeclared) namespace.
    assert_eq!(child.get_namespace(Some(&nc_name("ex"))), None);
    assert_eq!(child.namespace().unwrap().uri(), "http://example.com");
    assert_eq!(child.qualified_name().to_string(), "ex:item");
    common::assert_consistent(&document);
}

#[test]
fn resolution_uses_the_in_scope_declarations() {
    let document = Document::empty();
    let root = element(&document, "root");
    let child = element(&document, "child");
    root.append_child(child.clone());

    let default = Namespace::without_prefix("http://default").unwrap();
    root.declare_namespace(default.clone());
    let ex = Namespace::prefixed("http://example.com", "ex").unwrap();
    root.declare_namespace(ex.clone());

    // Elements inherit the default namespace for unprefixed names ...
    let resolved = child.resolve_qualified_name("item").unwrap();
    assert_eq!(resolved.namespace().unwrap().uri(), "http://default");
    // ... attributes never do.
    let resolved = child.resolve_attribute_name("item").unwrap();
    assert!(resolved.namespace().is_none());

    assert_eq!(
        child
            .resolve_qualified_name("ex:item")
            .unwrap()
            .namespace()
            .unwrap()
            .uri(),
        "http://example.com"
    );
    assert_eq!(
        child
            .resolve_attribute_name("ex:item")
            .unwrap()
            .namespace()
            .unwrap()
            .uri(),
        "http://example.com"
    );

    // The predefined `xml` prefix needs no declaration.
    assert_eq!(
        child
            .resolve_attribute_name("xml:lang")
            .unwrap()
            .namespace()
            .unwrap()
            .uri(),
        "http://www.w3.org/XML/1998/namespace"
    );

    // Errors are typed.
    assert!(matches!(
        child.resolve_qualified_name("nope:item"),
        Err(XmlError::UndeclaredPrefix(prefix)) if prefix == "nope"
    ));
    assert!(matches!(
        child.resolve_qualified_name("a:b:c"),
        Err(XmlError::InvalidName(_))
    ));
    assert!(matches!(
        child.resolve_qualified_name("xmlns:item"),
        Err(XmlError::ReservedPrefix(_))
    ));
}

#[test]
fn namespaced_documents_round_trip_through_the_parser() {
    // Parsing assigns the expanded names, so the tree already carries everything the editing API
    // needs; see G3 for full round-trip fidelity of the serializer.
    let document = parse_string(
        r#"<ex:root xmlns:ex="http://example.com" xmlns="http://default"><child ex:attr="v"/></ex:root>"#,
    )
    .unwrap();
    let root = document.root().unwrap();
    assert_eq!(root.local_name(), "root");
    assert_eq!(root.namespace().unwrap().uri(), "http://example.com");

    let child = root.child_elements()[0].clone();
    assert_eq!(child.local_name(), "child");
    assert_eq!(child.namespace().unwrap().uri(), "http://default");
    assert_eq!(
        child
            .attribute(&name("http://example.com", "ex", "attr"))
            .unwrap()
            .as_ref(),
        "v"
    );
    assert_eq!(child.namespace_declarations().len(), 0);

    // Serializing and re-parsing preserves the expanded names (the declaration is inherited from
    // the root, which is why no declaration is stored on the child).
    let reparsed = parse_string(&write_string(&document).unwrap()).unwrap();
    let reparsed_child = reparsed.root().unwrap().child_elements()[0].clone();
    assert_eq!(
        reparsed_child
            .attribute(&name("http://example.com", "ex", "attr"))
            .unwrap()
            .as_ref(),
        "v"
    );
}
