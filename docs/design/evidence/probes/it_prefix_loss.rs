//! PROBE `it_prefix_loss` — the serializer drops the prefix of an element name.
//!
//! Reproduce:
//!   docs/design/evidence/probes/run_probe.sh docs/design/evidence/probes/it_prefix_loss.rs
//!
//! Expected (defect present): the test FAILS, because `<html:body>` is serialized as `<body>`
//! while the now-unused `xmlns:html` declaration is still emitted. Round-tripping any
//! prefixed document therefore loses namespace information.
//!
//! Cause: `src/io.rs` `write_element()` builds the start/end tag from
//! `qname.local_name()` only, never `qname.namespace()`'s prefix. The same function *does*
//! prepend the prefix for attribute names, so attributes and elements disagree.

use biodivine_lib_xml_dom::{parse_string, write_string};

#[test]
fn serializer_preserves_element_prefix() {
    let xml = r#"<html:html xmlns:html="http://www.w3.org/1999/xhtml"><html:body>hi</html:body></html:html>"#;
    let doc = parse_string(xml).expect("input must parse");
    let out = write_string(&doc).expect("document must serialize");

    println!("input : {xml}");
    println!("output: {out}");

    assert!(
        out.contains("<html:body"),
        "element prefix lost by the serializer: {out}"
    );
}
