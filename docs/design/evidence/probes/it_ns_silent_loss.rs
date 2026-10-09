//! PROBE `it_ns_silent_loss` — namespaces are silently dropped on serialization.
//!
//! Reproduce:
//!   docs/design/evidence/probes/run_probe.sh docs/design/evidence/probes/it_ns_silent_loss.rs
//!
//! Expected (defect present): both tests FAIL, because there is no whole-document
//! validation, so a document whose element/attribute names carry a namespace that is not
//! declared anywhere in scope is happily serialized as if the namespace did not exist.
//! Re-parsing the output therefore yields a *different* document.
//!
//! Two variants are covered:
//!   (a) an element carrying a default (unprefixed) namespace with no declaration, and
//!   (b) an element/attribute carrying a *prefixed* namespace with no declaration of that prefix.
//!
//! `src/io.rs::write_element()` writes `xmlns:*` only for declarations that are actually
//! stored on the node, and silently ignores `qname.namespace().prefix()` when writing the
//! tag name (see `it_prefix_loss`). There is no `Document::validate()` to catch this.

use biodivine_lib_xml_dom::{Document, Namespace, QualifiedName, parse_string, write_string};

#[test]
fn default_namespace_without_declaration_survives_round_trip() {
    let doc = Document::empty();
    let ns = Namespace::without_prefix("http://example.com").unwrap();
    let root = doc.create_element(QualifiedName::with_namespace("r", &ns).unwrap());
    doc.set_root(root);

    let out = write_string(&doc).expect("serialize");
    println!("serialized: {out}");
    assert!(
        out.contains("xmlns=\"http://example.com\""),
        "namespace not declared in the output: {out}"
    );

    let reparsed = parse_string(&out).expect("reparse");
    let uri = reparsed
        .root()
        .unwrap()
        .qualified_name()
        .namespace()
        .map(|ns| ns.uri().to_string());
    println!("namespace after round trip: {uri:?}");
    assert_eq!(uri.as_deref(), Some("http://example.com"));
}

#[test]
fn prefixed_namespace_without_declaration_survives_round_trip() {
    let doc = Document::empty();
    let ns = Namespace::prefixed("http://example.com", "ex").unwrap();
    let root = doc.create_element(QualifiedName::with_namespace("r", &ns).unwrap());
    doc.set_root(root.clone());
    root.set_attribute(QualifiedName::with_namespace("a", &ns).unwrap(), "v");

    let out = write_string(&doc).expect("serialize");
    println!("serialized: {out}");
    assert!(
        out.contains("xmlns:ex=\"http://example.com\""),
        "prefix `ex` not declared in the output: {out}"
    );

    // Re-parsing must not fail and must preserve the namespace.
    let reparsed = match parse_string(&out) {
        Ok(d) => d,
        Err(e) => panic!("output is not even valid XML ({e}): {out}"),
    };
    let uri = reparsed
        .root()
        .unwrap()
        .qualified_name()
        .namespace()
        .map(|ns| ns.uri().to_string());
    assert_eq!(uri.as_deref(), Some("http://example.com"));
}
