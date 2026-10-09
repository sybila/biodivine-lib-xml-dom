//! PROBE `cf_missing_api` — compile-fail probe listing API surface that does not exist.
//!
//! Reproduce:
//!   docs/design/evidence/probes/run_probe.sh docs/design/evidence/probes/cf_missing_api.rs
//!
//! Expected (defect present): compilation FAILS with `E0599`/`E0425` errors for every line
//! below. This is the machine-checkable half of the "what is missing" audit: instead of
//! claiming a method is absent, we ask the compiler.
//!
//! Every line is a requirement from the task:
//!   * (1) arena handles + deep/shallow clone
//!   * (2) detached nodes, cross-document copies
//!   * (4)(2) whole-document validation
//!   * (3) ergonomic namespace queries/edits

use biodivine_lib_xml_dom::{Document, Element};

#[allow(unused, clippy::no_effect_underscore_binding)]
fn missing_api(doc: &Document, other: &Document, element: &Element) {
    // (1) Node/Element handles: there is no generic `Node` type at all.
    let _node: biodivine_lib_xml_dom::Node;

    // (1) deep / shallow clone (only `Clone` exists, which copies the handle).
    let _deep = element.deep_clone();
    let _shallow = element.shallow_clone();

    // (2) copying a subtree into another document.
    let _copied = element.deep_clone_into(other);

    // (2) explicit detach / remove / replace (there is no way to detach a node).
    let _removed = element.remove();
    let _detached = element.detach();
    let _replaced = element.replace_with(element.clone());

    // (2) `Document::create_element` exists, but there is no typed creation of other node kinds
    // and no way to insert an already created node at a position.
    let _inserted = element.insert_child(0, element.clone());

    // (3) ergonomic namespace access: in-scope resolution and declaration removal.
    let _in_scope = element.namespaces_in_scope();
    let _removed_decl = element.remove_namespace_declaration(None);

    // (4)(2) whole-document validation reporting all issues at once.
    let _issues = doc.validate();

    // (2) document identity/`PartialEq` for nodes across documents.
    let _same = element.belongs_to(other);
}
