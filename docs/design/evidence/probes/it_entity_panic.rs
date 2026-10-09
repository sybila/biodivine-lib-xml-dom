//! PROBE `it_entity_panic` — the parser aborts on the five predefined entities.
//!
//! Reproduce:
//!   docs/design/evidence/probes/run_probe.sh docs/design/evidence/probes/it_entity_panic.rs
//!
//! Expected (defect present): the test FAILS with "parser panicked", because
//! `src/io.rs:127` handles `quick_xml::events::Event::GeneralRef` with
//! `unimplemented!("Custom entities are currently not supported.")`.
//!
//! `&amp;` and friends are *predefined* entities that every XML processor must expand
//! (XML 1.0 §4.6). They are not "custom entities", so a plain, perfectly well-formed
//! document such as `<a>AT&amp;T</a>` kills the process. A library must never panic on
//! user input.
//!
//! The second test shows the intended behaviour for a genuinely undeclared entity
//! reference: a typed error, not a panic.

use biodivine_lib_xml_dom::{parse_string, write_string};
use std::panic;

/// A well-formed document using a predefined entity must parse without panicking.
#[test]
fn parser_does_not_panic_on_predefined_entities() {
    // Silence the default panic hook so the probe output stays readable; the panic is
    // still reported by `catch_unwind` below.
    let previous = panic::take_hook();
    panic::set_hook(Box::new(|info| {
        println!("!!! panic intercepted: {info}");
    }));
    let result = panic::catch_unwind(|| parse_string("<a>AT&amp;T</a>"));
    panic::set_hook(previous);

    match result {
        Ok(Ok(doc)) => println!(
            "OK: parsed and re-serialized as {:?}",
            write_string(&doc).expect("serialize")
        ),
        Ok(Err(e)) => println!("OK: typed error returned: {e}"),
        Err(_) => panic!("parser panicked instead of returning Ok(_) or Err(_)"),
    }
}

/// An undeclared general entity reference (no DTD processing in scope) must be a typed
/// error, never a panic.
#[test]
fn undeclared_entity_is_a_typed_error() {
    let previous = panic::take_hook();
    panic::set_hook(Box::new(|info| {
        println!("!!! panic intercepted: {info}");
    }));
    let result = panic::catch_unwind(|| parse_string("<a>&undefined;</a>"));
    panic::set_hook(previous);

    match result {
        Ok(Ok(_)) => println!("unexpectedly accepted `&undefined;`"),
        Ok(Err(e)) => println!("OK: typed error returned: {e}"),
        Err(_) => panic!("parser panicked instead of returning a typed error"),
    }
}
