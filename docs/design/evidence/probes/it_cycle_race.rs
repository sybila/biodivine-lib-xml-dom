//! PROBE `it_cycle_race` — two concurrent edits can create a parent/child cycle.
//!
//! Reproduce:
//!   docs/design/evidence/probes/run_probe.sh docs/design/evidence/probes/it_cycle_race.rs
//!
//! Expected (defect present): the test FAILS with a large number of created cycles.
//! Observed on 2026-10 (see RESULTS.md): 41984 cycles out of 128000 attempted pairs.
//!
//! Cause: `Element::add_child_element` is a check-then-act sequence guarded only by
//! per-node locks that are released between steps:
//!   1. `child.0.read().parent.is_some()`      (src/element.rs)
//!   2. `child.is_ancestor(self)`              (walks the parent chain, one lock at a time)
//!   3. `child.0.write().parent = Some(self)`
//!   4. `self.0.write().children.push(...)`
//! Two threads operating on two *different* children (thread A: `a.add_child(b)`,
//! thread B: `b.add_child(a)`) each pass their own "parent is none" check, because the two
//! checks read different fields. Both then perform step 3 and produce `a -> b -> a`.
//! After that, `is_ancestor` itself loops forever if it is ever asked whether `a` is an
//! ancestor of something outside the cycle, and `is_attached` does the same.
//!
//! This is exactly the class of bug the requested single document-level lock removes: the
//! whole check-then-act sequence becomes atomic.

use biodivine_lib_xml_dom::{Document, QualifiedName};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};

#[test]
fn concurrent_edits_must_not_create_cycles() {
    let cycles = Arc::new(AtomicUsize::new(0));
    let pairs_per_repetition = 256usize;
    let repetitions = 500usize;

    for _ in 0..repetitions {
        let doc = Document::empty();
        let mut pairs = Vec::with_capacity(pairs_per_repetition);
        for _ in 0..pairs_per_repetition {
            let a = doc.create_element(QualifiedName::without_namespace("a").unwrap());
            let b = doc.create_element(QualifiedName::without_namespace("b").unwrap());
            pairs.push((a, b));
        }
        let pairs = Arc::new(pairs);

        let barrier = Arc::new(Barrier::new(2));
        let (p1, p2) = (Arc::clone(&pairs), Arc::clone(&pairs));
        let (b1, b2) = (Arc::clone(&barrier), Arc::clone(&barrier));
        let (c1, c2) = (Arc::clone(&cycles), Arc::clone(&cycles));

        let t1 = std::thread::spawn(move || {
            b1.wait();
            for (a, b) in p1.iter() {
                let _ = a.add_child_element(b.clone());
            }
            for (a, _) in p1.iter() {
                if a.is_ancestor(a) {
                    c1.fetch_add(1, Ordering::Relaxed);
                }
            }
        });
        let t2 = std::thread::spawn(move || {
            b2.wait();
            for (a, b) in p2.iter() {
                let _ = b.add_child_element(a.clone());
            }
            for (a, _) in p2.iter() {
                if a.is_ancestor(a) {
                    c2.fetch_add(1, Ordering::Relaxed);
                }
            }
        });
        t1.join().unwrap();
        t2.join().unwrap();
    }

    let total = cycles.load(Ordering::Relaxed);
    println!(
        "loops created: {total} out of {} attempted parent/child pairs",
        pairs_per_repetition * repetitions
    );
    assert_eq!(total, 0, "concurrent edits created parent/child cycles");
}
