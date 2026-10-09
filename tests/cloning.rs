//! Clone semantics and cross-document copies.
//!
//! Requirement (1) distinguishes three different "clone" operations, and requirement (2) adds the
//! cross-document copy. All four are covered here, together with the consistency guarantee for
//! copies taken while the source is being mutated concurrently.

mod common;

use biodivine_lib_xml_dom::{
    Document, Element, Namespace, Node, NodeContent, QualifiedName, XmlError, write_string,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use common::{assert_consistent, element};

/// Builds a namespaced tree:
///
/// ```text
/// ex:root (xmlns:ex="http://example.com", xmlns="http://default")
/// ├── ex:child (ex:attr="v", attr="plain")
/// │   └── "text"
/// └── "tail"
/// ```
fn namespaced_tree(document: &Document) -> (Element, Element) {
    let ex = Namespace::prefixed("http://example.com", "ex").unwrap();
    let default = Namespace::without_prefix("http://default").unwrap();

    let root = document.create_element(QualifiedName::with_namespace("root", &ex).unwrap());
    root.declare_namespace(ex.clone());
    root.declare_namespace(default.clone());

    let child = document.create_element(QualifiedName::with_namespace("child", &ex).unwrap());
    child.set_attribute(QualifiedName::with_namespace("attr", &ex).unwrap(), "v");
    child.set_attribute(QualifiedName::without_namespace("attr").unwrap(), "plain");
    child.append_child(document.create_text("text").unwrap());
    root.append_child(child.clone());
    root.append_child(document.create_text("tail").unwrap());

    (root, child)
}

#[test]
fn cloning_a_handle_refers_to_the_same_node() {
    let document = Document::empty();
    let node = element(&document, "node");
    let handle = node.clone();

    assert!(handle.ptr_eq(&node.node()));
    assert_eq!(handle, node);
    assert_eq!(node.id(), handle.id());
    assert_eq!(document.node_count(), 1);
}

#[test]
fn shallow_clone_copies_the_payload_but_not_the_children() {
    let document = Document::empty();
    let (_root, child) = namespaced_tree(&document);
    child.detach();
    let before = document.node_count();

    let clone = child.shallow_clone();
    assert!(!clone.is_attached());
    assert!(!clone.ptr_eq(&child));
    assert_eq!(clone.qualified_name(), child.qualified_name());
    assert_eq!(clone.attributes(), child.attributes());
    assert_eq!(
        clone.namespace_declarations(),
        child.namespace_declarations()
    );
    assert!(clone.children().is_empty());
    assert_eq!(child.children().len(), 1);
    // Exactly one new node: the clone does not drag its children along.
    assert_eq!(document.node_count(), before + 1);
}

#[test]
fn shallow_clone_of_a_text_node_copies_the_content() {
    let document = Document::empty();
    let text = document.create_text("hello").unwrap();
    let clone = text.shallow_clone();
    assert!(!clone.ptr_eq(&text));
    assert_eq!(clone.text().unwrap().as_str(), "hello");
}

#[test]
fn deep_clone_copies_the_whole_subtree() {
    let document = Document::empty();
    let (root, _) = namespaced_tree(&document);
    document.set_root(root.clone());

    let clone = root.deep_clone();
    assert!(!clone.is_attached());
    assert!(!clone.ptr_eq(&root));
    assert_eq!(clone.qualified_name(), root.qualified_name());
    assert_eq!(
        clone.namespace_declarations(),
        root.namespace_declarations()
    );
    assert_eq!(clone.children().len(), root.children().len());

    let original_child = root.child_elements()[0].clone();
    let copied_child = clone.child_elements()[0].clone();
    assert!(!copied_child.ptr_eq(&original_child));
    assert_eq!(
        copied_child.qualified_name(),
        original_child.qualified_name()
    );
    assert_eq!(copied_child.attributes(), original_child.attributes());
    assert_eq!(
        copied_child.children()[0].text().unwrap(),
        original_child.children()[0].text().unwrap()
    );

    // The original tree is untouched and both trees are consistent.
    assert_eq!(root.children().len(), 2);
    assert_consistent(&document);
    assert_subtree_consistent(&clone);
}

#[test]
fn deep_clone_into_another_document_rebuilds_the_tree_there() {
    let source = Document::empty();
    let target = Document::empty();
    let (root, _) = namespaced_tree(&source);
    source.set_root(root.clone());
    let source_nodes = source.node_count();

    let copy = root.deep_clone_into(&target);
    assert!(copy.belongs_to(&target));
    assert!(!copy.belongs_to(&source));
    assert!(!copy.is_attached());
    assert_eq!(copy.qualified_name(), root.qualified_name());
    assert_eq!(copy.namespace_declarations(), root.namespace_declarations());
    assert_eq!(copy.children().len(), 2);

    let copied_child = copy.child_elements()[0].clone();
    assert_eq!(copied_child.local_name(), "child");
    assert_eq!(
        copied_child
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
    assert_eq!(
        copied_child
            .attribute(&QualifiedName::without_namespace("attr").unwrap())
            .unwrap()
            .as_ref(),
        "plain"
    );

    // The source is unchanged, and the copy lives in the target.
    assert_eq!(source.node_count(), source_nodes);
    assert!(target.nodes().iter().any(|node| node.ptr_eq(&copy.node())));
    assert_consistent(&source);
    assert_subtree_consistent(&copy);
}

#[test]
fn deep_clone_into_the_same_document_behaves_like_deep_clone() {
    let document = Document::empty();
    let (root, _) = namespaced_tree(&document);

    let via_clone = root.deep_clone();
    let via_into = root.deep_clone_into(&document);
    assert!(!via_into.is_attached());
    assert_eq!(via_into.children().len(), via_clone.children().len());
    assert_eq!(via_into.qualified_name(), via_clone.qualified_name());
    assert!(!via_into.ptr_eq(&via_clone));
}

#[test]
fn shallow_clone_into_another_document_keeps_only_the_node() {
    let source = Document::empty();
    let target = Document::empty();
    let (root, _) = namespaced_tree(&source);

    let copy = root.shallow_clone_into(&target);
    assert!(copy.belongs_to(&target));
    assert!(copy.children().is_empty());
    assert_eq!(copy.qualified_name(), root.qualified_name());
    assert_eq!(copy.namespace_declarations(), root.namespace_declarations());
}

#[test]
fn a_copied_subtree_can_be_attached_in_the_target_document() {
    let source = Document::empty();
    let target = Document::empty();
    let (root, _) = namespaced_tree(&source);

    let target_root = element(&target, "wrapper");
    target.set_root(target_root.clone());
    let copy = root.deep_clone_into(&target);
    target_root.append_child(copy.clone());

    assert!(copy.is_attached());
    assert_eq!(target_root.child_elements().len(), 1);

    // Attaching it to the *source* is still an error: the handle belongs to the target now.
    assert!(matches!(
        root.append_child_checked(copy.clone()),
        Err(XmlError::ForeignDocument)
    ));
    assert_consistent(&source);
    assert_consistent(&target);
}

#[test]
fn non_element_nodes_can_be_copied_between_documents() {
    let source = Document::empty();
    let target = Document::empty();

    let text = source.create_text("hello").unwrap();
    let comment = source.create_comment(" note ").unwrap();
    let cdata = source.create_cdata("raw <x>").unwrap();
    let pi = source
        .create_processing_instruction("target", "data")
        .unwrap();

    for node in [&text, &comment, &cdata, &pi] {
        let copy = node.deep_clone_into(&target);
        assert!(copy.belongs_to(&target));
        assert_eq!(copy.kind(), node.kind());
        assert_eq!(copy.content(), node.content());
    }
}

#[test]
fn a_deep_copy_can_be_edited_independently_of_its_source() {
    let document = Document::empty();
    let (root, child) = namespaced_tree(&document);
    document.set_root(root.clone());

    let copy = root.deep_clone();
    copy.child_elements()[0]
        .set_attribute(QualifiedName::without_namespace("attr").unwrap(), "changed");

    assert_eq!(
        child
            .attribute(&QualifiedName::without_namespace("attr").unwrap())
            .unwrap()
            .as_ref(),
        "plain"
    );
    assert_eq!(
        copy.child_elements()[0]
            .attribute(&QualifiedName::without_namespace("attr").unwrap())
            .unwrap()
            .as_ref(),
        "changed"
    );
}

#[test]
fn a_deep_clone_is_cheap_because_names_are_shared() {
    // Not a timing test: this checks that copying reuses the shared `Arc` payloads by comparing
    // the values, which is what makes the copy cheap (the allocation identity is asserted by the
    // crate-internal `re_interns_names_in_the_target_document` test).
    let document = Document::empty();
    let (root, _) = namespaced_tree(&document);
    let clone = root.deep_clone();

    assert_eq!(clone.qualified_name(), root.qualified_name());
    assert!(
        clone
            .namespace_declarations()
            .values()
            .all(|namespace| namespace.is_some())
    );
}

#[test]
fn display_renders_the_subtree_as_xml() {
    let document = Document::empty();
    let root = element(&document, "root");
    root.append_child(document.create_text("hello").unwrap());
    root.append_child(document.create_comment(" note ").unwrap());
    root.append_child(document.create_cdata("raw <content>").unwrap());
    document.set_root(root.clone());

    let rendered = root.to_string();
    assert!(rendered.contains("<root>"), "{rendered}");
    assert!(rendered.contains("hello"), "{rendered}");
    assert!(rendered.contains("<!-- note -->"), "{rendered}");
    assert!(rendered.contains("<![CDATA[raw <content>]]>"), "{rendered}");
    assert_eq!(rendered, write_string(&document).unwrap());
}

#[test]
fn display_of_non_element_nodes() {
    let document = Document::empty();
    assert_eq!(document.create_text("hi").unwrap().to_string(), "hi");
    assert_eq!(
        document.create_comment(" c ").unwrap().to_string(),
        "<!-- c -->"
    );
    assert!(
        document.create_cdata("]]>not-allowed").is_err(),
        "CDATA must reject `]]>`"
    );
    assert_eq!(
        document.create_cdata("raw").unwrap().to_string(),
        "<![CDATA[raw]]>"
    );
    assert_eq!(
        document
            .create_processing_instruction("target", "data")
            .unwrap()
            .to_string(),
        "<?target data?>"
    );
    assert_eq!(
        document
            .create_processing_instruction("target", "")
            .unwrap()
            .to_string(),
        "<?target?>"
    );
}

#[test]
fn node_content_exposes_every_kind() {
    let document = Document::empty();
    let root = element(&document, "root");
    let text = document.create_text("hi").unwrap();
    root.append_child(text.clone());

    assert_eq!(text.kind(), biodivine_lib_xml_dom::NodeKind::Text);
    assert!(matches!(text.content(), NodeContent::Text(_)));
    assert!(text.as_element().is_none());
    assert!(text.text().is_some());
    assert!(text.comment().is_none());

    assert!(matches!(root.content(), NodeContent::Element(_)));
    assert_eq!(root.as_element().unwrap(), root);
}

/// A copy taken while the source is mutated concurrently must never be corrupted.
///
/// `deep_clone_into` snapshots the source under a read lock, releases it, and only then inserts
/// into the target, so the source can change in between without affecting the copy's consistency.
#[test]
fn copying_while_the_source_is_mutated_produces_consistent_copies() {
    let source = Document::empty();
    let target = Document::empty();

    let root = element(&source, "root");
    source.set_root(root.clone());
    for index in 0..8 {
        let child = element(&source, &format!("child{index}"));
        child.append_child(source.create_text("payload").unwrap());
        root.append_child(child);
    }

    let stop = Arc::new(AtomicBool::new(false));
    let mutator = {
        let source = source.clone();
        let root = root.clone();
        let stop = Arc::clone(&stop);
        std::thread::spawn(move || {
            let mut counter = 0usize;
            while !stop.load(Ordering::Relaxed) {
                let child = element(&source, &format!("mutated{counter}"));
                child.append_child(source.create_text("payload").unwrap());
                root.append_child(child);
                if let Some(first) = root.first_child() {
                    first.detach();
                }
                counter += 1;
            }
            counter
        })
    };

    let mut copies = 0usize;
    for _ in 0..200 {
        let copy = root.deep_clone_into(&target);
        assert!(copy.belongs_to(&target));
        assert!(!copy.is_attached());
        assert_subtree_consistent(&copy.as_element().unwrap());
        copies += 1;
    }

    stop.store(true, Ordering::Relaxed);
    let mutator_iterations = mutator.join().unwrap();
    assert!(mutator_iterations > 0, "the mutator never ran");
    assert_eq!(copies, 200);

    assert_consistent(&source);
    assert_consistent(&target);
}

/// Asserts that a (possibly detached) subtree is internally consistent.
fn assert_subtree_consistent(root: &Element) {
    let mut stack: Vec<Node> = vec![root.node()];
    let mut visited: Vec<String> = Vec::new();
    while let Some(node) = stack.pop() {
        assert!(
            !visited.contains(&node.id().to_string()),
            "the copied subtree contains node {} twice",
            node.id()
        );
        visited.push(node.id().to_string());
        for child in node.children() {
            assert_eq!(
                child.parent().map(|parent| parent.id()),
                Some(node.id()),
                "a child of the copy points to a different parent"
            );
            stack.push(child);
        }
    }
}
