#![allow(dead_code)] // Shared helpers: not every test binary uses every one of them.
//!
//! Helpers shared by the integration tests.
//!
//! These tests only use the public API, so they double as a check that the API is actually usable
//! from outside the crate.

use biodivine_lib_xml_dom::{Document, QualifiedName, write_string};

/// Creates a childless element with the given local name.
pub fn element(document: &Document, name: &str) -> biodivine_lib_xml_dom::Element {
    document.create_element(QualifiedName::without_namespace(name).unwrap())
}

/// A structural snapshot of one node: `(id, kind, parent)`, used to compare states.
pub type NodeState = (String, String, Option<String>);

/// A structural snapshot of the whole document, sorted by node id.
pub fn snapshot(document: &Document) -> Vec<NodeState> {
    let mut result: Vec<NodeState> = document
        .nodes()
        .iter()
        .map(|node| {
            (
                node.id().to_string(),
                format!("{:?}", node.kind()),
                node.parent().map(|parent| parent.id().to_string()),
            )
        })
        .collect();
    result.sort();
    result
}

/// Asserts every structural invariant the arena promises, from the outside.
///
/// * every node has the parent its parent claims (and appears in that parent's child list exactly
///   once);
/// * no node reaches itself by walking up the parent chain, and the walk always terminates;
/// * the root has no parent, and no node has the root as a child.
///
/// This is what the panic-safety tests use to show that a failed operation leaves the document
/// consistent rather than half-modified.
pub fn assert_consistent(document: &Document) {
    let nodes = document.nodes();
    let total = nodes.len();

    for node in &nodes {
        let parent = node.parent();
        if let Some(parent) = &parent {
            let occurrences = parent
                .children()
                .iter()
                .filter(|child| child.ptr_eq(node))
                .count();
            assert_eq!(
                occurrences,
                1,
                "node {} occurs {occurrences} times in the child list of {}",
                node.id(),
                parent.id()
            );
        }

        // Walking up must terminate and must not revisit a node. `total + 1` steps are enough
        // because a node can have at most `total` ancestors in a consistent tree.
        let mut steps = 0usize;
        let mut current = parent.clone();
        let mut seen = vec![node.id()];
        while let Some(ancestor) = current {
            assert!(
                !seen.contains(&ancestor.id()),
                "the parent chain of node {} contains a cycle",
                node.id()
            );
            seen.push(ancestor.id());
            steps += 1;
            assert!(
                steps <= total,
                "the parent chain of node {} is longer than the document",
                node.id()
            );
            current = ancestor.parent();
        }
    }

    if let Some(root) = document.root() {
        assert!(
            root.parent().is_none(),
            "the document root must not have a parent"
        );
        for node in &nodes {
            if node.ptr_eq(&root.node()) {
                continue;
            }
            assert!(
                !node.is_ancestor(&root.node()),
                "node {} is an ancestor of the document root",
                node.id()
            );
        }
    }

    // The tree must still be serializable.
    write_string(document).expect("a consistent document must serialize");
}
