//! Structural editing: attachment, detachment, insertion, replacement, roots, and the guarantees
//! that hold when an operation fails.
//!
//! The tests are written against the public API only. The panic-safety tests are the operational
//! proof of the claim in `docs/design/PLAN.md` §3.2: an operation validates everything it can fail
//! on *before* touching the arena, so a rejected (or panicking) call leaves the document exactly
//! as it was and fully consistent.

mod common;

use biodivine_lib_xml_dom::{Document, Element, Node, NodeKind, QualifiedName, XmlError};
use std::panic::{AssertUnwindSafe, catch_unwind};

use common::{assert_consistent, element, snapshot};

/// Runs `operation`, expecting it to panic, and returns the panic message.
fn panic_message(operation: impl FnOnce()) -> String {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = catch_unwind(AssertUnwindSafe(operation));
    std::panic::set_hook(previous);

    let payload = result.expect_err("the operation was expected to panic");
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| {
            payload
                .downcast_ref::<&'static str>()
                .map(|message| (*message).to_string())
        })
        .unwrap_or_else(|| "<non-string panic payload>".to_string())
}

// ---------------------------------------------------------------------------------------------
// Basic editing
// ---------------------------------------------------------------------------------------------

#[test]
fn attaching_and_detaching_keeps_the_tree_consistent() {
    let document = Document::empty();
    let root = element(&document, "root");
    let first = element(&document, "first");
    let second = element(&document, "second");

    root.append_child(first.clone());
    root.append_child(second.clone());
    document.set_root(root.clone());

    assert_eq!(root.child_elements(), vec![first.clone(), second.clone()]);
    assert!(first.is_attached());
    assert_eq!(first.parent().unwrap(), root.node());

    first.detach();
    assert!(!first.is_attached());
    assert_eq!(root.child_elements(), vec![second.clone()]);
    assert_consistent(&document);
}

#[test]
fn detaching_is_idempotent_and_returns_the_previous_parent() {
    let document = Document::empty();
    let root = element(&document, "root");
    let child = element(&document, "child");
    root.append_child(child.clone());

    assert_eq!(child.detach().unwrap(), root.node());
    assert!(child.detach().is_none());
    assert!(child.detach().is_none());
    assert_consistent(&document);
}

#[test]
fn remove_returns_the_detached_node_itself() {
    let document = Document::empty();
    let root = element(&document, "root");
    let child = element(&document, "child");
    root.append_child(child.clone());

    let removed = child.remove();
    assert!(removed.ptr_eq(&child));
    assert!(!removed.is_attached());
    assert!(root.children().is_empty());
}

#[test]
fn attaching_a_detached_subtree_restores_it() {
    let document = Document::empty();
    let root = element(&document, "root");
    let parent = element(&document, "parent");
    let grandchild = element(&document, "grandchild");
    parent.append_child(grandchild.clone());

    assert!(!parent.is_attached());
    root.append_child(parent.clone());
    document.set_root(root.clone());
    assert!(parent.is_attached());
    assert!(grandchild.is_attached());
    assert_consistent(&document);
}

#[test]
fn insertion_positions_are_stable() {
    let document = Document::empty();
    let root = element(&document, "root");
    let a = element(&document, "a");
    let b = element(&document, "b");
    let c = element(&document, "c");
    let d = element(&document, "d");
    root.append_child(a.clone());
    root.append_child(c.clone());

    root.insert_child(1, b.clone());
    assert_eq!(names(&root), ["a", "b", "c"]);

    root.insert_after(a.clone(), d.clone());
    assert_eq!(names(&root), ["a", "d", "b", "c"]);

    root.insert_before(c.clone(), a.clone());
    assert_eq!(names(&root), ["d", "b", "a", "c"]);

    // Moving an existing child interprets the index after detaching it.
    root.insert_child(0, c.clone());
    assert_eq!(names(&root), ["c", "d", "b", "a"]);
    assert_consistent(&document);
}

#[test]
fn replacing_swaps_the_node_in_place() {
    let document = Document::empty();
    let root = element(&document, "root");
    let old = element(&document, "old");
    let replacement = element(&document, "replacement");
    root.append_child(old.clone());

    let detached = old.replace_with(replacement.clone());
    assert!(detached.ptr_eq(&old));
    assert!(!detached.is_attached());
    assert_eq!(names(&root), ["replacement"]);
    assert_eq!(replacement.parent().unwrap(), root.node());
    assert_consistent(&document);
}

#[test]
fn mixed_content_node_kinds_are_preserved() {
    let document = Document::empty();
    let root = element(&document, "root");
    root.append_child(document.create_text("before").unwrap());
    root.append_child(document.create_comment(" note ").unwrap());
    root.append_child(document.create_cdata("raw <content>").unwrap());
    root.append_child(
        document
            .create_processing_instruction("target", "data")
            .unwrap(),
    );
    root.append_child(element(&document, "child"));

    let kinds: Vec<NodeKind> = root.children().iter().map(Node::kind).collect();
    assert_eq!(
        kinds,
        vec![
            NodeKind::Text,
            NodeKind::Comment,
            NodeKind::CData,
            NodeKind::ProcessingInstruction,
            NodeKind::Element,
        ]
    );
    assert_consistent(&document);
}

// ---------------------------------------------------------------------------------------------
// Rejections: cycles, foreign documents, roots, indices
// ---------------------------------------------------------------------------------------------

#[test]
fn cycles_are_rejected() {
    let document = Document::empty();
    let a = element(&document, "a");
    let b = element(&document, "b");
    let c = element(&document, "c");
    a.append_child(b.clone());
    b.append_child(c.clone());

    assert!(matches!(
        c.append_child_checked(a.clone()),
        Err(XmlError::CycleDetected)
    ));
    assert!(matches!(
        a.append_child_checked(a.clone()),
        Err(XmlError::CycleDetected)
    ));
    assert!(matches!(
        b.append_child_checked(a.clone()),
        Err(XmlError::CycleDetected)
    ));
    assert_eq!(names(&a), ["b"]);
    assert_eq!(names(&b), ["c"]);
    assert_consistent(&document);
}

#[test]
fn non_elements_cannot_be_parents() {
    let document = Document::empty();
    let text = document.create_text("hello").unwrap();
    let child = element(&document, "child");
    assert!(matches!(
        text.append_child_checked(child),
        Err(XmlError::NotAnElement(_))
    ));
}

#[test]
fn foreign_documents_are_rejected() {
    let first = Document::empty();
    let second = Document::empty();
    let parent = element(&first, "parent");
    let foreign = element(&second, "foreign");

    assert!(matches!(
        parent.append_child_checked(foreign.clone()),
        Err(XmlError::ForeignDocument)
    ));
    assert!(matches!(
        parent.insert_child_checked(0, foreign.clone()),
        Err(XmlError::ForeignDocument)
    ));
    assert!(matches!(
        parent.insert_before_checked(foreign.clone(), foreign.clone()),
        Err(XmlError::ForeignDocument)
    ));
    let other_parent = element(&second, "other");
    assert!(matches!(
        other_parent.replace_with_checked(foreign.clone()),
        Err(XmlError::NodeHasNoParent(_)) // `other_parent` is detached
    ));
    assert_consistent(&first);
    assert_consistent(&second);
}

#[test]
fn the_root_cannot_be_attached_as_a_child() {
    let document = Document::empty();
    let root = element(&document, "root");
    let child = element(&document, "child");
    document.set_root(root.clone());

    assert!(matches!(
        child.append_child_checked(root.clone()),
        Err(XmlError::CannotAttachRoot)
    ));
    assert!(!root.is_ancestor(&child.node()));
    assert_consistent(&document);
}

#[test]
fn an_attached_element_cannot_become_the_root() {
    let document = Document::empty();
    let root = element(&document, "root");
    let child = element(&document, "child");
    root.append_child(child.clone());

    assert!(matches!(
        document.set_root_checked(child.clone()),
        Err(XmlError::RootHasParent)
    ));
    assert_eq!(document.root(), None);
    assert_eq!(root.child_elements(), vec![child]);
    assert_consistent(&document);
}

#[test]
fn a_foreign_element_cannot_become_the_root() {
    let first = Document::empty();
    let second = Document::empty();
    let foreign = element(&second, "foreign");

    assert!(matches!(
        first.set_root_checked(foreign),
        Err(XmlError::ForeignDocument)
    ));
    assert!(first.root().is_none());
}

#[test]
fn out_of_range_insertions_are_rejected() {
    let document = Document::empty();
    let root = element(&document, "root");
    let child = element(&document, "child");
    root.append_child(child.clone());

    let extra = element(&document, "extra");
    assert!(matches!(
        root.insert_child_checked(2, extra.clone()),
        Err(XmlError::IndexOutOfBounds { index: 2, len: 1 })
    ));
    assert_eq!(names(&root), ["child"]);
    assert!(!extra.is_attached());
}

#[test]
fn relative_insertion_requires_a_sibling_of_the_same_parent() {
    let document = Document::empty();
    let root = element(&document, "root");
    let other = element(&document, "other");
    let child = element(&document, "child");
    let unrelated = element(&document, "unrelated");

    assert!(matches!(
        root.insert_before_checked(child, other.clone()),
        Err(XmlError::NotAChild(_, _))
    ));
    assert!(matches!(
        root.insert_after_checked(unrelated, other),
        Err(XmlError::NotAChild(_, _))
    ));
    assert!(root.children().is_empty());
}

#[test]
fn setting_the_root_returns_the_previous_one() {
    let document = Document::empty();
    let first = element(&document, "first");
    let second = element(&document, "second");

    assert!(document.set_root(first.clone()).is_none());
    let previous = document.set_root(second.clone()).unwrap();
    assert!(previous.ptr_eq(&first));
    assert!(!first.is_attached());
    assert!(second.is_attached());
    assert!(document.clear_root().unwrap().ptr_eq(&second));
    assert!(document.root().is_none());
    assert_consistent(&document);
}

// ---------------------------------------------------------------------------------------------
// Panic safety: a panicking ergonomic method must not corrupt the document
// ---------------------------------------------------------------------------------------------

/// Runs `operation` (which must panic) and checks that the document is byte-for-byte unchanged
/// and still consistent, and that a subsequent edit still works.
fn assert_panics_without_side_effects(
    document: &Document,
    expected: &str,
    operation: impl FnOnce(),
) {
    let before = snapshot(document);
    let message = panic_message(operation);
    assert!(
        message.contains(expected),
        "panic message `{message}` does not mention `{expected}`"
    );
    assert_eq!(
        snapshot(document),
        before,
        "a failed operation modified the document"
    );
    assert_consistent(document);

    // The document is still editable afterwards.
    let probe = element(document, "probe");
    document.root().unwrap().append_child(probe);
    assert_consistent(document);
}

#[test]
fn panicking_append_child_leaves_the_document_unchanged() {
    let document = Document::empty();
    let root = element(&document, "root");
    let child = element(&document, "child");
    let grandchild = element(&document, "grandchild");
    root.append_child(child.clone());
    child.append_child(grandchild.clone());
    document.set_root(root.clone());

    // The root cannot become a child at all.
    assert_panics_without_side_effects(&document, "cannot be attached", || {
        grandchild.append_child(root.clone());
    });

    // A genuine cycle: attaching an ancestor of the receiver.
    assert_panics_without_side_effects(&document, "would create a cycle", || {
        grandchild.append_child(child.clone());
    });
}

#[test]
fn panicking_append_child_on_a_foreign_document_leaves_everything_unchanged() {
    let first = Document::empty();
    let second = Document::empty();
    let root = element(&first, "root");
    first.set_root(root.clone());
    let foreign = element(&second, "foreign");

    assert_panics_without_side_effects(&first, "different document", || {
        root.append_child(foreign.clone());
    });
    assert_consistent(&second);
}

#[test]
fn panicking_insert_child_leaves_the_document_unchanged() {
    let document = Document::empty();
    let root = element(&document, "root");
    let child = element(&document, "child");
    root.append_child(child.clone());
    document.set_root(root.clone());

    // Note: creating `extra` before the snapshot is taken, because creating a node is itself a
    // (legitimate) modification of the document.
    let extra = element(&document, "extra");
    assert_panics_without_side_effects(&document, "out of range", || {
        root.insert_child(5, extra.clone());
    });
}

#[test]
fn panicking_replace_with_leaves_the_document_unchanged() {
    let document = Document::empty();
    let root = element(&document, "root");
    let detached = element(&document, "detached");
    let replacement = element(&document, "replacement");
    document.set_root(root.clone());

    // `detached` has no parent, so there is nothing to replace it in.
    assert_panics_without_side_effects(&document, "has no parent", || {
        detached.replace_with(replacement.clone());
    });

    // Replacing a node by its own ancestor would break the tree.
    let parent = element(&document, "parent");
    let child = element(&document, "child");
    root.append_child(parent.clone());
    parent.append_child(child.clone());
    assert_panics_without_side_effects(&document, "would create a cycle", || {
        child.replace_with(parent.clone());
    });
}

#[test]
fn panicking_set_root_leaves_the_document_unchanged() {
    let document = Document::empty();
    let root = element(&document, "root");
    let child = element(&document, "child");
    root.append_child(child.clone());
    document.set_root(root.clone());

    assert_panics_without_side_effects(&document, "already attached", || {
        document.set_root(child.clone());
    });
}

#[test]
fn panicking_set_attribute_leaves_the_document_unchanged() {
    let document = Document::empty();
    let root = element(&document, "root");
    root.set_attribute(QualifiedName::without_namespace("a").unwrap(), "b");
    document.set_root(root.clone());

    assert_panics_without_side_effects(&document, "not a valid XML text", || {
        root.set_attribute(QualifiedName::without_namespace("c").unwrap(), "bad \u{1}");
    });
    assert_eq!(root.attributes().len(), 1);
}

// ---------------------------------------------------------------------------------------------
// Detached nodes stay fully usable
// ---------------------------------------------------------------------------------------------

#[test]
fn a_detached_subtree_stays_editable_clonable_and_reattachable() {
    let document = Document::empty();
    let root = element(&document, "root");
    document.set_root(root.clone());

    let parent = element(&document, "parent");
    let child = element(&document, "child");
    parent.append_child(child.clone());
    root.append_child(parent.clone());

    // Detach the whole subtree.
    parent.detach();
    assert!(!parent.is_attached());
    assert!(!child.is_attached());
    assert!(parent.is_ancestor(&child.node()));

    // Still editable while detached.
    let extra = element(&document, "extra");
    child.append_child(extra.clone());
    parent.set_attribute(QualifiedName::without_namespace("a").unwrap(), "b");
    assert_eq!(
        parent
            .attribute(&QualifiedName::without_namespace("a").unwrap())
            .unwrap(),
        "b".into()
    );
    assert_eq!(child.child_elements(), vec![extra.clone()]);

    // Still deeply clonable while detached.
    let clone = parent.deep_clone();
    assert_eq!(clone.child_elements().len(), 1);
    assert_eq!(clone.child_elements()[0].child_elements().len(), 1);
    assert!(!clone.is_attached());

    // Still re-attachable, including under a different parent.
    let other = element(&document, "other");
    root.append_child(other.clone());
    other.append_child(parent.clone());
    assert!(parent.is_attached());
    assert!(extra.is_attached());
    assert_consistent(&document);
}

#[test]
fn attachment_state_is_only_about_the_root_of_the_document() {
    let document = Document::empty();
    let root = element(&document, "root");
    let child = element(&document, "child");
    root.append_child(child.clone());

    // Without a root nothing is attached, even though the nodes are linked to each other.
    assert!(!root.is_attached());
    assert!(!child.is_attached());
    document.set_root(root.clone());
    assert!(root.is_attached());
    assert!(child.is_attached());
    document.clear_root();
    assert!(!root.is_attached());
    assert!(!child.is_attached());
}

// ---------------------------------------------------------------------------------------------
// Tree queries
// ---------------------------------------------------------------------------------------------

#[test]
fn sibling_and_index_queries_agree_with_the_child_list() {
    let document = Document::empty();
    let root = element(&document, "root");
    let a = element(&document, "a");
    let b = element(&document, "b");
    let c = element(&document, "c");
    root.append_child(a.clone());
    root.append_child(b.clone());
    root.append_child(c.clone());

    assert_eq!(a.index_in_parent(), Some(0));
    assert_eq!(b.index_in_parent(), Some(1));
    assert_eq!(c.index_in_parent(), Some(2));
    assert!(root.index_in_parent().is_none());

    assert_eq!(root.first_child().unwrap().as_element().unwrap(), a);
    assert_eq!(root.last_child().unwrap().as_element().unwrap(), c);
    assert!(a.previous_sibling().is_none());
    assert_eq!(a.next_sibling().unwrap().as_element().unwrap(), b);
    assert_eq!(c.previous_sibling().unwrap().as_element().unwrap(), b);
    assert!(c.next_sibling().is_none());
}

#[test]
fn descendants_are_returned_in_document_pre_order() {
    let document = Document::empty();
    let root = element(&document, "root");
    let a = element(&document, "a");
    let a1 = element(&document, "a1");
    let b = element(&document, "b");
    root.append_child(a.clone());
    a.append_child(a1.clone());
    root.append_child(b.clone());

    let names: Vec<String> = root
        .descendants()
        .iter()
        .filter_map(|node| node.as_element())
        .map(|element| element.local_name().to_string())
        .collect();
    assert_eq!(names, ["a", "a1", "b"]);
}

#[test]
fn a_deeply_nested_document_does_not_overflow_the_stack() {
    // The previous implementation recursed through `Arc` handles for parent lookups, so deep
    // documents were limited by the stack size.
    let document = Document::empty();
    let root = element(&document, "root");
    document.set_root(root.clone());

    let mut current = root.clone();
    for _ in 0..5_000 {
        let child = element(&document, "child");
        current.append_child(child.clone());
        current = child;
    }

    assert_eq!(root.descendants().len(), 5_000);
    assert!(current.is_attached());
    assert!(root.is_ancestor(&current.node()));
    assert!(!current.is_ancestor(&root.node()));
}

/// The local names of an element's element children, in order.
fn names(element: &Element) -> Vec<String> {
    element
        .child_elements()
        .iter()
        .map(|child| child.local_name().to_string())
        .collect()
}
