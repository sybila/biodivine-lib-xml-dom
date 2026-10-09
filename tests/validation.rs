//! Whole-document validation (requirement (4)(2)).
//!
//! The tests are organised by the group of rules they exercise: structure, namespace scope,
//! namespace declarations, and values. They also pin the *contract* of the editing API: editing
//! never checks namespace integrity, and the problems show up only here (requirement (3)).

mod common;

use biodivine_lib_xml_dom::xml_spec::{NCName, nc_name, rules};
use biodivine_lib_xml_dom::{
    Document, Element, Namespace, QualifiedName, ValidationErrorKind, XmlValidationError,
    parse_string, write_string,
};

use common::element;

/// The reserved XML namespace URI.
const XML_NS: &str = "http://www.w3.org/XML/1998/namespace";

/// An `xml:*` qualified name.
fn xml_name(local: &str) -> QualifiedName {
    QualifiedName::with_namespace(local, &Namespace::prefixed(XML_NS, "xml").unwrap()).unwrap()
}

/// Collects the kinds of the issues found, in order.
fn kinds(document: &Document) -> Vec<ValidationErrorKind> {
    document
        .validate()
        .expect_err("the document was expected to be invalid")
        .into_errors()
        .into_iter()
        .map(|error| error.kind().clone())
        .collect()
}

/// Asserts that the document is valid.
fn assert_valid(document: &Document) {
    if let Err(errors) = document.validate() {
        panic!("the document was expected to be valid, but:\n{errors}");
    }
}

// ---------------------------------------------------------------------------------------------
// Structure
// ---------------------------------------------------------------------------------------------

#[test]
fn an_empty_document_has_no_root() {
    // rule: rule.well-formedness.document-production.md
    rules::assert_rule_exists("rule.well-formedness.document-production.md");
    let document = Document::empty();
    assert!(!document.is_valid());
    let errors = document.validate().unwrap_err();
    assert_eq!(errors.len(), 1);
    let error = &errors.as_slice()[0];
    assert_eq!(error.kind(), &ValidationErrorKind::MissingRoot);
    assert_eq!(error.node(), None, "a missing root has no node to point at");
    assert!(error.to_string().contains("no root element"));
}

#[test]
fn a_consistent_document_has_no_structural_problems() {
    let document = Document::empty();
    let root = element(&document, "root");
    let child = element(&document, "child");
    let grandchild = element(&document, "grandchild");
    root.append_child(child.clone());
    child.append_child(grandchild.clone());
    document.set_root(root.clone());

    // Detached nodes are a normal state, not an error (requirement (2)).
    let _detached = element(&document, "detached");
    assert_valid(&document);

    // Re-attaching the subtree keeps it valid.
    child.detach();
    assert_valid(&document);
    root.append_child(child);
    assert_valid(&document);
}

#[test]
fn every_parsed_fixture_is_valid() {
    let fixtures = [
        r#"<a/>"#,
        r#"<?xml version="1.0" encoding="UTF-8"?><a/>"#,
        r#"<h:root xmlns:h="http://h" xmlns="http://d"><child h:attr="v"/></h:root>"#,
        r#"<root xmlns="http://d"><inner xmlns=""><leaf/></inner></root>"#,
        r#"<a xml:lang="en-GB" xml:space="preserve"><b xml:lang="fr"/></a>"#,
        r#"<a xml:id="one"><b xml:id="two"/></a>"#,
        r#"<a xmlns:ex="http://e"><ex:b>text</ex:b><!-- c --><![CDATA[raw]]><?pi data?></a>"#,
    ];
    for fixture in fixtures {
        let document = parse_string(fixture).unwrap();
        assert_valid(&document);
        // ... and the round trip stays valid.
        assert_valid(&parse_string(&write_string(&document).unwrap()).unwrap());
    }
}

// ---------------------------------------------------------------------------------------------
// Namespace scope
// ---------------------------------------------------------------------------------------------

#[test]
fn an_element_in_a_namespace_needs_it_declared() {
    // rule: rule.namespace-usage.prefix-declared.md
    rules::assert_rule_exists("rule.namespace-usage.prefix-declared.md");

    let document = Document::empty();
    let ex = Namespace::prefixed("http://example.com", "ex").unwrap();
    let root = document.create_element(QualifiedName::with_namespace("item", &ex).unwrap());
    document.set_root(root.clone());

    assert_eq!(
        kinds(&document),
        vec![ValidationErrorKind::UndeclaredPrefix {
            prefix: nc_name("ex")
        }]
    );

    // Declaring it fixes the document.
    root.declare_namespace(ex);
    assert_valid(&document);
}

#[test]
fn a_prefix_bound_to_a_different_uri_is_reported() {
    // rule: rule.namespace-usage.prefix-declaration-scope.md
    rules::assert_rule_exists("rule.namespace-usage.prefix-declaration-scope.md");

    let document = Document::empty();
    let root = element(&document, "root");
    document.set_root(root.clone());
    // The declaration binds `ex` to one URI ...
    root.declare_namespace(Namespace::prefixed("http://one", "ex").unwrap());
    // ... while the child's name claims another.
    let child = document.create_element(
        QualifiedName::with_namespace("child", &Namespace::prefixed("http://two", "ex").unwrap())
            .unwrap(),
    );
    root.append_child(child);

    assert_eq!(
        kinds(&document),
        vec![ValidationErrorKind::PrefixBoundToDifferentUri {
            prefix: nc_name("ex"),
            expected: "http://two".to_string(),
            actual: "http://one".to_string(),
        }]
    );
}

#[test]
fn default_namespace_scope_must_match_the_name() {
    // rule: rule.namespace-usage.default-namespace-scope.md
    rules::assert_rule_exists("rule.namespace-usage.default-namespace-scope.md");

    // (a) A name in the default namespace with no declaration in scope.
    let document = Document::empty();
    let root = document.create_element(
        QualifiedName::with_namespace("a", &Namespace::without_prefix("http://d").unwrap())
            .unwrap(),
    );
    document.set_root(root.clone());
    assert_eq!(
        kinds(&document),
        vec![ValidationErrorKind::MissingDefaultNamespace {
            expected: "http://d".to_string()
        }]
    );
    root.declare_namespace(Namespace::without_prefix("http://d").unwrap());
    assert_valid(&document);

    // (b) A name in a different default namespace than the one in scope. Note that the declaration
    //     applies to the element that carries it, so the root's own name has to match as well.
    let document = Document::empty();
    let root = document.create_element(
        QualifiedName::with_namespace("a", &Namespace::without_prefix("http://d").unwrap())
            .unwrap(),
    );
    document.set_root(root.clone());
    root.declare_namespace(Namespace::without_prefix("http://d").unwrap());
    let child = document.create_element(
        QualifiedName::with_namespace("b", &Namespace::without_prefix("http://other").unwrap())
            .unwrap(),
    );
    root.append_child(child);
    assert_eq!(
        kinds(&document),
        vec![ValidationErrorKind::DefaultNamespaceMismatch {
            expected: "http://other".to_string(),
            actual: "http://d".to_string(),
        }]
    );

    // (c) A name with *no* namespace inside a default-namespace scope: writing it out would put it
    //     in `http://d`, which is exactly the "moving an element breaks its declarations" case.
    let document = Document::empty();
    let root = document.create_element(
        QualifiedName::with_namespace("a", &Namespace::without_prefix("http://d").unwrap())
            .unwrap(),
    );
    document.set_root(root.clone());
    root.declare_namespace(Namespace::without_prefix("http://d").unwrap());
    root.append_child(element(&document, "b"));
    assert_eq!(
        kinds(&document),
        vec![ValidationErrorKind::UnprefixedNameTakesDefaultNamespace {
            default_uri: "http://d".to_string()
        }]
    );
    // Removing the default declaration fixes the child (which now really has no namespace) but
    // breaks the root, whose name *is* in that namespace. Validation reports the remaining problem
    // instead of silently "repairing" either of them.
    root.remove_namespace_declaration(None);
    assert_eq!(
        kinds(&document),
        vec![ValidationErrorKind::MissingDefaultNamespace {
            expected: "http://d".to_string()
        }]
    );
}

#[test]
fn attributes_are_not_affected_by_the_default_namespace() {
    // rule: rule.namespace-usage.default-namespace-not-attributes.md
    rules::assert_rule_exists("rule.namespace-usage.default-namespace-not-attributes.md");

    // An unprefixed attribute inside a default-namespace scope is perfectly fine.
    let document = Document::empty();
    let root = document.create_element(
        QualifiedName::with_namespace("a", &Namespace::without_prefix("http://d").unwrap())
            .unwrap(),
    );
    document.set_root(root.clone());
    root.declare_namespace(Namespace::without_prefix("http://d").unwrap());
    root.set_attribute(QualifiedName::without_namespace("plain").unwrap(), "v");
    assert_valid(&document);

    // An attribute that carries a namespace but no prefix cannot be written at all.
    root.set_attribute(
        QualifiedName::new(
            nc_name("odd"),
            Some(Namespace::without_prefix("http://d").unwrap()),
        ),
        "v",
    );
    assert_eq!(
        kinds(&document),
        vec![ValidationErrorKind::AttributeNamespaceWithoutPrefix {
            uri: "http://d".to_string()
        }]
    );
}

#[test]
fn the_xml_prefix_is_always_available() {
    // rule: rule.namespace-basics.xml-prefix-fixed-binding.md
    rules::assert_rule_exists("rule.namespace-basics.xml-prefix-fixed-binding.md");

    let document = Document::empty();
    let root = element(&document, "root");
    document.set_root(root.clone());
    root.set_attribute(xml_name("lang"), "en");
    root.set_attribute(xml_name("space"), "preserve");
    root.set_attribute(xml_name("id"), "one");
    // No declaration is needed, and none is stored.
    assert!(root.namespace_declarations().is_empty());
    assert_valid(&document);
}

#[test]
fn detached_subtrees_are_validated_against_their_own_scope() {
    let document = Document::empty();
    let root = element(&document, "root");
    document.set_root(root);
    let detached = document.create_element(
        QualifiedName::with_namespace(
            "item",
            &Namespace::prefixed("http://example.com", "ex").unwrap(),
        )
        .unwrap(),
    );
    // Not attached, and the prefix is not declared: still reported, so a subtree can be prepared
    // and validated before it is used.
    assert_eq!(
        kinds(&document),
        vec![ValidationErrorKind::UndeclaredPrefix {
            prefix: nc_name("ex")
        }]
    );
    detached.declare_namespace(Namespace::prefixed("http://example.com", "ex").unwrap());
    assert_valid(&document);
}

// ---------------------------------------------------------------------------------------------
// Values
// ---------------------------------------------------------------------------------------------

#[test]
fn xml_id_values_must_be_names_and_unique() {
    // rule: rule.attributes.id-must-be-name.md
    // rule: rule.attributes.id-must-be-unique.md
    rules::assert_rule_exists("rule.attributes.id-must-be-name.md");
    rules::assert_rule_exists("rule.attributes.id-must-be-unique.md");

    let document = Document::empty();
    let root = element(&document, "root");
    document.set_root(root.clone());
    root.set_attribute(xml_name("id"), "first");
    let child = element(&document, "child");
    root.append_child(child.clone());
    child.set_attribute(xml_name("id"), "second");
    assert_valid(&document);

    // A duplicate is reported once, on the second element that uses the value.
    child.set_attribute(xml_name("id"), "first");
    assert_eq!(
        kinds(&document),
        vec![ValidationErrorKind::DuplicateXmlId {
            id: nc_name("first")
        }]
    );

    // A value that is not an NCName is reported as such.
    child.set_attribute(xml_name("id"), "not a name");
    assert_eq!(
        kinds(&document),
        vec![ValidationErrorKind::XmlIdIsNotAName {
            value: "not a name".to_string()
        }]
    );
}

#[test]
fn a_detached_copy_does_not_conflict_with_its_original() {
    let document = Document::empty();
    let root = element(&document, "root");
    document.set_root(root.clone());
    root.set_attribute(xml_name("id"), "one");

    // A detached clone carries the same `xml:id`. Uniqueness is a property of the *document*, i.e.
    // of the tree reachable from the root, so a detached copy is not a duplicate - which is what
    // makes "clone a subtree, edit it, attach it later" work.
    let clone = root.deep_clone();
    assert_valid(&document);

    // Attaching it does create a duplicate, and that is reported.
    root.append_child(clone);
    assert_eq!(
        kinds(&document),
        vec![ValidationErrorKind::DuplicateXmlId { id: nc_name("one") }]
    );
}

#[test]
fn xml_lang_and_xml_space_values_are_checked() {
    // rule: rule.document-structure.xml-lang-must-be-bcp47-or-empty.md
    // rule: rule.document-structure.xml-space-must-be-enumerated-default-preserve.md
    rules::assert_rule_exists("rule.document-structure.xml-lang-must-be-bcp47-or-empty.md");
    rules::assert_rule_exists(
        "rule.document-structure.xml-space-must-be-enumerated-default-preserve.md",
    );

    let document = Document::empty();
    let root = element(&document, "root");
    document.set_root(root.clone());
    root.set_attribute(xml_name("lang"), "en-GB");
    root.set_attribute(xml_name("space"), "preserve");
    assert_valid(&document);

    // The empty string is a legal `xml:lang`.
    root.set_attribute(xml_name("lang"), "");
    assert_valid(&document);

    root.set_attribute(xml_name("lang"), "en_US_very_long_invalid");
    root.set_attribute(xml_name("space"), "preserved");
    assert_eq!(
        kinds(&document),
        vec![
            ValidationErrorKind::InvalidXmlLang {
                value: "en_US_very_long_invalid".to_string()
            },
            ValidationErrorKind::InvalidXmlSpace {
                value: "preserved".to_string()
            },
        ]
    );
}

// ---------------------------------------------------------------------------------------------
// Aggregation, ordering and presentation
// ---------------------------------------------------------------------------------------------

#[test]
fn all_problems_are_reported_in_one_call() {
    // rule: rule.attributes.id-must-be-unique.md
    let document = Document::empty();
    // Problem 1: no root element (yet).
    assert_eq!(kinds(&document).len(), 1);

    let root = element(&document, "root");
    document.set_root(root.clone());
    assert_valid(&document);

    // Now introduce four independent problems at once.
    // 2: an undeclared prefix on the root's name.
    root.set_qualified_name(
        QualifiedName::with_namespace("root", &Namespace::prefixed("http://x", "missing").unwrap())
            .unwrap(),
    );
    // 3: an invalid `xml:space` value.
    root.set_attribute(xml_name("space"), "nope");
    // 4: an invalid `xml:lang` value.
    root.set_attribute(xml_name("lang"), "de_DE");
    // 5: an attribute that carries a namespace without a prefix.
    root.set_attribute(
        QualifiedName::new(
            nc_name("bad"),
            Some(Namespace::without_prefix("http://d").unwrap()),
        ),
        "v",
    );

    let errors = document.validate().unwrap_err();
    assert_eq!(errors.len(), 4, "{errors}");
    assert_eq!(
        errors
            .as_slice()
            .iter()
            .map(|error| error.kind().rule())
            .collect::<Vec<_>>(),
        vec![
            "rule.namespace-usage.prefix-declared.md",
            "rule.namespace-usage.default-namespace-not-attributes.md",
            "rule.document-structure.xml-lang-must-be-bcp47-or-empty.md",
            "rule.document-structure.xml-space-must-be-enumerated-default-preserve.md",
        ]
    );
    // Every issue points at the node it belongs to.
    assert!(
        errors
            .as_slice()
            .iter()
            .all(|error| error.node() == Some(root.id()))
    );
    // The display form lists them all.
    let rendered = errors.to_string();
    assert!(rendered.starts_with("4 validation problems"), "{rendered}");
    assert_eq!(rendered.lines().count(), 5);
}

#[test]
fn validation_is_deterministic() {
    let document = Document::empty();
    let root = element(&document, "root");
    document.set_root(root.clone());
    root.set_attribute(xml_name("space"), "nope");
    root.append_child(element(&document, "a"));
    root.append_child(element(&document, "b"));

    let first = document.validate().unwrap_err();
    let second = document.validate().unwrap_err();
    assert_eq!(first, second);
    // Two siblings that are both unprefixed inside a default-namespace scope would both be
    // reported, in arena order.
    root.declare_namespace(Namespace::without_prefix("http://d").unwrap());
    let third = document.validate().unwrap_err();
    assert_eq!(third.len(), 4, "{third}");
}

#[test]
fn every_error_kind_names_an_existing_rule_file() {
    // The rule ids used in messages must point at real files, so that a reader can always find the
    // rule an issue came from.
    let document = Document::empty();
    let root = element(&document, "root");
    document.set_root(root.clone());
    root.set_qualified_name(
        QualifiedName::with_namespace("root", &Namespace::prefixed("http://x", "missing").unwrap())
            .unwrap(),
    );
    root.set_attribute(xml_name("space"), "nope");
    let child = element(&document, "child");
    root.append_child(child.clone());
    child.set_attribute(xml_name("id"), "not a name");

    let errors = document.validate().unwrap_err();
    assert!(!errors.is_empty());
    for error in &errors {
        rules::assert_rule_exists(error.kind().rule());
    }
    // And the set of rules referenced covers the layer-C rules this goal implements.
    let referenced: Vec<&str> = errors.iter().map(|error| error.kind().rule()).collect();
    assert!(referenced.contains(&"rule.namespace-usage.prefix-declared.md"));
    assert!(referenced.contains(&"rule.attributes.id-must-be-name.md"));
    assert!(
        referenced
            .contains(&"rule.document-structure.xml-space-must-be-enumerated-default-preserve.md")
    );

    // Missing-root is the only kind that can be reported without a node.
    let empty = Document::empty().validate().unwrap_err();
    assert_eq!(
        empty.as_slice()[0].kind(),
        &ValidationErrorKind::MissingRoot
    );
    rules::assert_rule_exists(empty.as_slice()[0].kind().rule());
}

// ---------------------------------------------------------------------------------------------
// The editing API does not check namespace integrity
// ---------------------------------------------------------------------------------------------

#[test]
fn editing_is_silent_and_validation_is_what_reports() {
    // Requirement (3): "I can remove the namespace declaration attribute, and it will not complain
    // even if some child nodes rely on that declaration."
    let document =
        parse_string(r#"<ex:root xmlns:ex="http://example.com"><ex:child/></ex:root>"#).unwrap();
    let root = document.root().unwrap();
    assert_valid(&document);

    let child = root.child_elements()[0].clone();
    // Removing the declaration succeeds silently, and nothing else changes.
    root.remove_namespace_declaration(Some(&nc_name("ex")));
    assert_eq!(child.qualified_name().to_string(), "ex:child");
    assert_eq!(child.namespace().unwrap().uri(), "http://example.com");

    // The breakage is reported by validation, once per affected name: the root and the child both
    // use the `ex` prefix.
    let errors = document.validate().unwrap_err();
    assert_eq!(errors.len(), 2, "{errors}");
    assert!(
        errors
            .iter()
            .all(|error| matches!(error.kind(), ValidationErrorKind::UndeclaredPrefix { .. }))
    );

    // Restoring the declaration makes the document valid again.
    root.declare_namespace(Namespace::prefixed("http://example.com", "ex").unwrap());
    assert_valid(&document);
}

#[test]
fn moving_an_element_can_break_its_namespace_and_validation_says_so() {
    // Moving an element into a different scope is exactly the case the task description calls out.
    let document = Document::empty();
    let root = element(&document, "root");
    document.set_root(root.clone());
    let plain = root.shallow_clone();
    plain.set_qualified_name(QualifiedName::without_namespace("plain").unwrap());

    let elsewhere = document.create_element(
        QualifiedName::with_namespace("elsewhere", &Namespace::without_prefix("http://d").unwrap())
            .unwrap(),
    );
    elsewhere.declare_namespace(Namespace::without_prefix("http://d").unwrap());
    root.append_child(elsewhere.clone());

    // `elsewhere` declares a default namespace, so `plain` inside it would change meaning.
    assert_valid(&document);
    elsewhere.append_child(plain.clone());
    assert_eq!(
        kinds(&document),
        vec![ValidationErrorKind::UnprefixedNameTakesDefaultNamespace {
            default_uri: "http://d".to_string()
        }]
    );

    // Moving it back fixes it - and the element itself was never modified by the move.
    plain.detach();
    assert_valid(&document);
    assert_eq!(plain.qualified_name().to_string(), "plain");
}

#[test]
fn validation_does_not_change_the_document() {
    let document =
        parse_string(r#"<ex:root xmlns:ex="http://example.com"><ex:b/></ex:root>"#).unwrap();
    let before = write_string(&document).unwrap();
    let _ = document.validate();
    let _ = document.is_valid();
    assert_eq!(write_string(&document).unwrap(), before);
}

/// A small helper so that the tests can build `xml:*` attributes without repeating the namespace.
#[allow(dead_code)]
fn xml_namespace() -> Namespace {
    Namespace::prefixed(XML_NS, "xml").unwrap()
}

/// A helper for building an element with an `xml:id`.
#[allow(dead_code)]
fn with_xml_id(document: &Document, id: &str) -> Element {
    let element = element(document, "element");
    element.set_attribute(xml_name("id"), id);
    element
}

/// The `NCName` type is used by the assertions above; keep the import explicit.
#[allow(dead_code)]
fn assert_ncname(name: &NCName) -> &str {
    name.as_str()
}

/// Confirms that the error type is usable as a boxed error.
#[test]
fn the_error_type_is_a_std_error() {
    fn as_error(error: XmlValidationError) -> Box<dyn std::error::Error + Send + Sync> {
        Box::new(error)
    }
    let document = Document::empty();
    let errors = document.validate().unwrap_err();
    let first = errors.into_errors().into_iter().next().unwrap();
    let boxed = as_error(first);
    assert!(boxed.to_string().contains("no root element"));
}
