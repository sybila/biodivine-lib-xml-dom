//! Cancellation of the long-running operations.
//!
//! A caller registers a `cancel_this` trigger around an operation; the operation checks it
//! periodically and returns [`XmlError::Cancelled`] instead of running to completion. The trigger
//! scope is thread-local, so no token is threaded through any signature.

mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use biodivine_lib_xml_dom::cancel_this::{self, CancellationTrigger};
use biodivine_lib_xml_dom::{Document, XmlError};
use common::assert_consistent;

/// A document large enough that every walk below consults the triggers many times.
fn source(items: usize) -> String {
    let mut xml = String::from("<root>");
    for index in 0..items {
        xml.push_str(&format!("<item id=\"{index}\">text</item>"));
    }
    xml.push_str("</root>");
    xml
}

/// A trigger that fires on its `n`-th consultation, so that an operation is cancelled part-way
/// through rather than before it starts.
#[derive(Clone)]
struct CancelAfter {
    remaining: Arc<AtomicUsize>,
}

impl CancelAfter {
    fn new(consultations: usize) -> Self {
        Self {
            remaining: Arc::new(AtomicUsize::new(consultations)),
        }
    }
}

impl CancellationTrigger for CancelAfter {
    fn is_cancelled(&self) -> bool {
        // Saturating: once the countdown reaches zero, the trigger stays cancelled.
        self.remaining
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |value| {
                Some(value.saturating_sub(1))
            })
            .map(|previous| previous <= 1)
            .unwrap_or(true)
    }

    fn type_name(&self) -> &'static str {
        "CancelAfter"
    }
}

/// Runs `action` under a trigger that is already cancelled and returns the resulting error.
fn cancelled<T: std::fmt::Debug>(
    action: impl FnOnce() -> Result<T, XmlError>,
) -> XmlError {
    let trigger = cancel_this::CancelAtomic::new();
    trigger.cancel();
    cancel_this::on_atomic(trigger, action).unwrap_err()
}

/// Runs `action` under a trigger that fires on its second consultation, i.e. after the operation
/// has started and made progress.
fn cancelled_midway<T: std::fmt::Debug>(
    action: impl FnOnce() -> Result<T, XmlError>,
) -> XmlError {
    cancel_this::on_trigger(CancelAfter::new(2), action).unwrap_err()
}

fn assert_cancelled(error: XmlError) {
    assert!(
        matches!(error, XmlError::Cancelled(_)),
        "expected a cancellation, got {error:?}"
    );
}

#[test]
fn every_long_operation_can_be_cancelled() {
    let xml = source(500);
    assert_cancelled(cancelled(|| Document::parse_string(&xml)));

    let document = Document::parse_string(&xml).unwrap();
    assert_cancelled(cancelled(|| document.write_string()));
    assert_cancelled(cancelled(|| document.validate()));
    assert_cancelled(cancelled(|| document.validation_errors()));
    assert_cancelled(cancelled(|| document.is_valid()));
    assert_cancelled(cancelled(|| document.collect_garbage()));

    let root = document.root().unwrap();
    assert_cancelled(cancelled(|| root.deep_clone()));

    let target = Document::new();
    assert_cancelled(cancelled(|| root.deep_clone_into(&target)));
}

#[test]
fn every_long_operation_can_be_cancelled_part_way() {
    let xml = source(5_000);
    assert_cancelled(cancelled_midway(|| Document::parse_string(&xml)));

    let document = Document::parse_string(&xml).unwrap();
    assert_cancelled(cancelled_midway(|| document.write_string()));
    assert_cancelled(cancelled_midway(|| document.validate()));
    assert_cancelled(cancelled_midway(|| document.collect_garbage()));

    let root = document.root().unwrap();
    assert_cancelled(cancelled_midway(|| root.deep_clone()));
    let target = Document::new();
    assert_cancelled(cancelled_midway(|| root.deep_clone_into(&target)));
}

#[test]
fn cheap_operations_are_never_cancelled() {
    let document = Document::parse_string(&source(10)).unwrap();
    let root = document.root().unwrap();
    let target = Document::new();

    let trigger = cancel_this::CancelAtomic::new();
    trigger.cancel();
    let result: Result<(), XmlError> = cancel_this::on_atomic(trigger, || {
        // A shallow copy touches one node, and `Display` cannot report an error.
        let copy = root.shallow_clone();
        assert!(copy.children().is_empty());
        assert!(root.shallow_clone_into(&target).belongs_to(&target));
        assert!(format!("{root}").starts_with("<root>"));
        assert!(format!("{}", root.node()).starts_with("<root>"));
        Ok(())
    });
    assert!(result.is_ok());
}

#[test]
fn a_generous_timeout_does_not_cancel() {
    let xml = source(500);
    let result = cancel_this::on_timeout(Duration::from_secs(3600), || {
        let document = Document::parse_string(&xml)?;
        document.write_string()?;
        document.validate()?;
        let _ = document.collect_garbage()?;
        let target = Document::new();
        document.root().unwrap().deep_clone_into(&target)?;
        Ok::<_, XmlError>(())
    });
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn a_cancelled_collection_changes_nothing() {
    // Large enough for the collector to consult the trigger more than once, small enough for the
    // quadratic consistency check below.
    let document = Document::parse_string(&source(1_000)).unwrap();
    // A handle keeps a detached node alive; a detached subtree without a handle is garbage.
    let kept = document.create_text("kept").unwrap();
    let garbage = document.root().unwrap().deep_clone().unwrap();
    drop(garbage);
    let before = document.node_count();

    assert_cancelled(cancelled_midway(|| document.collect_garbage()));
    assert_eq!(document.node_count(), before, "nothing was reclaimed");
    assert_consistent(&document);
    assert!(document.is_valid().unwrap());

    // The collection still works once the trigger is gone, and the handle is still live.
    let stats = document.collect_garbage().unwrap();
    assert_eq!(stats.collected, before - document.node_count());
    assert!(stats.collected > 0);
    assert_eq!(kept.text().unwrap().as_str(), "kept");
    assert_consistent(&document);
}

#[test]
fn a_cancelled_copy_creates_nothing() {
    let document = Document::parse_string(&source(5_000)).unwrap();
    let root = document.root().unwrap();
    let before = document.node_count();
    assert_cancelled(cancelled_midway(|| root.deep_clone()));
    assert_eq!(document.node_count(), before);

    let target = Document::new();
    assert_cancelled(cancelled_midway(|| root.deep_clone_into(&target)));
    assert_eq!(target.node_count(), 0);
}

#[test]
fn a_copy_cancelled_at_any_point_leaves_nothing_behind() {
    // Each copy consults the trigger a few dozen times while it reads the source and while it
    // builds the copy, so cancelling at every consultation in turn covers both phases.
    let source = Document::parse_string(&source(2_000)).unwrap();
    let root = source.root().unwrap();
    let target = Document::new();
    let mut completed = false;
    for consultations in 2..200 {
        let before = source.node_count();
        let same = cancel_this::on_trigger(CancelAfter::new(consultations), || {
            root.deep_clone()
        });
        let other = cancel_this::on_trigger(CancelAfter::new(consultations), || {
            root.deep_clone_into(&target)
        });
        match (same, other) {
            (Err(first), Err(second)) => {
                assert_cancelled(first);
                assert_cancelled(second);
                assert_eq!(source.node_count(), before);
                assert_eq!(target.node_count(), 0);
            }
            (Ok(_), Ok(_)) => {
                completed = true;
                break;
            }
            other => panic!("the two copies disagree: {other:?}"),
        }
    }
    assert!(
        completed,
        "the copies never completed, so the loop did not reach the end"
    );
    assert_consistent(&source);
    assert_consistent(&target);
}

#[test]
fn a_trigger_only_applies_to_its_own_thread() {
    let document = Document::parse_string(&source(500)).unwrap();
    let trigger = cancel_this::CancelAtomic::new();
    trigger.cancel();
    let result: Result<(), XmlError> = cancel_this::on_atomic(trigger, || {
        let other = document.clone();
        let valid = std::thread::spawn(move || other.is_valid()).join().unwrap();
        assert!(valid.unwrap(), "the other thread has no trigger registered");
        Ok(())
    });
    assert!(result.is_ok());
}

#[test]
fn nested_triggers_all_apply() {
    let document = Document::parse_string(&source(500)).unwrap();
    let outer = cancel_this::CancelAtomic::new();
    let result: Result<(), XmlError> = cancel_this::on_atomic(outer.clone(), || {
        let inner = cancel_this::CancelAtomic::new();
        cancel_this::on_atomic(inner, || {
            document.validate()?;
            outer.cancel();
            document.validate()
        })
    });
    assert_cancelled(result.unwrap_err());
    // Neither trigger is registered any more.
    assert!(document.validate().is_ok());
}
