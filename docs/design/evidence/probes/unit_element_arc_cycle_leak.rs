//! PROBE `unit_arc_cycle_leak` — the parent <-> child `Arc` pair is a reference cycle.
//!
//! Reproduce:
//!   docs/design/evidence/probes/run_probe.sh docs/design/evidence/probes/unit_element_arc_cycle_leak.rs
//!
//! Expected (defect present): the test FAILS with `strong_count(child) == 2`, i.e. the
//! whole subtree stays alive after the `Document` and the last user handle are dropped.
//!
//! Cause: `ElementData` (src/element.rs) stores `parent: Option<Element>` while the parent
//! stores `children: Vec<XmlNode>` containing `XmlNode::Element(Element)`, and
//! `Element = Arc<RwLock<ElementData>>`. Both directions are strong references.
//!
//! This probe needs crate internals (`Arc::strong_count(&child.0)`), so it is injected as a
//! temporary `mod __probe` into `src/lib.rs`; the runner restores the file afterwards.
//! It documents the pre-rewrite implementation at commit 76beb74 and is not meant to be
//! kept in the crate's own test suite.

use super::*;
use crate::document::Document;
use crate::qualified_name::QualifiedName;
use std::sync::Arc;

#[test]
fn dropped_document_does_not_free_its_nodes() {
    let doc = Document::empty();
    let parent = doc.create_element(QualifiedName::without_namespace("p").unwrap());
    let child = doc.create_element(QualifiedName::without_namespace("c").unwrap());
    parent.add_child_element(child.clone()).unwrap();
    doc.set_root(parent.clone()).unwrap();

    println!(
        "before drop: strong_count(child) = {}, strong_count(parent) = {}",
        Arc::strong_count(&child.0),
        Arc::strong_count(&parent.0)
    );

    drop(doc);
    drop(parent);

    let count = Arc::strong_count(&child.0);
    println!("after dropping the document and the parent handle:");
    println!("  strong_count(child) = {count}");
    println!("  expected            1 (only the `child` handle above)");
    assert_eq!(
        count, 1,
        "reference cycle keeps the whole subtree alive (memory leak)"
    );
}
