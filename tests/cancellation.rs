//! Cancellation of the long-running operations.
//!
//! A caller registers a `cancel_this` trigger around an operation; the operation checks it
//! periodically and returns [`XmlError::Cancelled`] instead of running to completion. The trigger
//! scope is thread-local, so no token is threaded through any signature.

use std::time::Duration;

use biodivine_lib_xml_dom::cancel_this;
use biodivine_lib_xml_dom::{Document, XmlError};

/// A document large enough that the operations below have something to walk.
fn source() -> String {
    let mut xml = String::from("<root>");
    for index in 0..500 {
        xml.push_str(&format!("<item id=\"{index}\">text</item>"));
    }
    xml.push_str("</root>");
    xml
}

/// Runs `action` under a trigger that is already cancelled and returns the resulting error.
fn cancelled<T: std::fmt::Debug>(
    action: impl FnOnce() -> Result<T, XmlError>,
) -> XmlError {
    let trigger = cancel_this::CancelAtomic::new();
    trigger.cancel();
    cancel_this::on_atomic(trigger, action).unwrap_err()
}

fn assert_cancelled(error: XmlError) {
    assert!(
        matches!(error, XmlError::Cancelled(_)),
        "expected a cancellation, got {error:?}"
    );
}

#[test]
fn every_long_operation_can_be_cancelled() {
    let xml = source();
    assert_cancelled(cancelled(|| Document::parse_string(&xml)));

    let document = Document::parse_string(&xml).unwrap();
    assert_cancelled(cancelled(|| document.write_string()));
    assert_cancelled(cancelled(|| document.validate()));
    assert_cancelled(cancelled(|| document.validation_errors()));
    assert_cancelled(cancelled(|| document.is_valid()));
    assert_cancelled(cancelled(|| document.collect_garbage()));

    let root = document.root().unwrap();
    assert_cancelled(cancelled(|| root.deep_clone()));
    assert_cancelled(cancelled(|| root.shallow_clone()));

    let target = Document::empty();
    assert_cancelled(cancelled(|| root.deep_clone_into(&target)));
    assert_cancelled(cancelled(|| root.shallow_clone_into(&target)));
}

#[test]
fn a_generous_timeout_does_not_cancel() {
    let xml = source();
    let result = cancel_this::on_timeout(Duration::from_secs(3600), || {
        let document = Document::parse_string(&xml)?;
        document.write_string()?;
        document.validate()?;
        let _ = document.collect_garbage()?;
        root_round_trip(&document)
    });
    assert!(result.is_ok(), "{result:?}");
}

fn root_round_trip(document: &Document) -> Result<(), XmlError> {
    let target = Document::empty();
    document.root().unwrap().deep_clone_into(&target)?;
    Ok(())
}

#[test]
fn a_cancelled_collection_leaves_the_document_usable() {
    let document = Document::parse_string(&source()).unwrap();
    // A handle keeps a detached node alive across the cancelled collection.
    let kept = document.create_text("kept").unwrap();
    let before = document.node_count();

    assert_cancelled(cancelled(|| document.collect_garbage()));
    assert_eq!(document.node_count(), before);

    // The collection still works once the trigger is gone, and the handle is still live.
    assert_eq!(document.collect_garbage().unwrap().collected, 0);
    assert!(!kept.is_attached());
}
