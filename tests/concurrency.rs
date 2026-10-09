//! Concurrency: the guarantees that follow from a single document-level lock.
//!
//! These tests cover the two failure modes of the previous design (REVIEW D3 and D4):
//! interleaved check-then-act sequences creating cycles, and per-node locking making the edits
//! non-atomic. They also act as a deadlock detector: every worker reports back over a channel and
//! the test fails on a timeout instead of hanging forever.

mod common;

use biodivine_lib_xml_dom::{Document, Element, Node, QualifiedName};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, channel};
use std::sync::{Arc, Barrier};
use std::time::Duration;

use common::{assert_consistent, element};

/// How long a worker may take before the test declares a deadlock.
const WATCHDOG: Duration = Duration::from_secs(60);

/// Waits for `count` workers to report completion, failing the test on a timeout.
fn wait_for(receiver: &Receiver<()>, count: usize) {
    for _ in 0..count {
        receiver
            .recv_timeout(WATCHDOG)
            .expect("a worker did not finish in time: the document lock is deadlocked or stuck");
    }
}

#[test]
fn two_threads_adding_opposite_children_never_create_a_cycle() {
    // This is the ported version of the `it_cycle_race` reproduction probe from G1, which created
    // ~29% of cycles with the old per-node locking. With one lock the entire check-then-act
    // sequence is a single critical section, so the number of created cycles must be exactly 0.
    let pairs_per_round = 256usize;
    let rounds = 500usize;
    let cycles = Arc::new(AtomicUsize::new(0));
    let resolved = Arc::new(AtomicUsize::new(0));
    let (sender, receiver) = channel();

    for _ in 0..rounds {
        let document = Document::empty();
        let pairs: Vec<(Element, Element)> = (0..pairs_per_round)
            .map(|_| (element(&document, "a"), element(&document, "b")))
            .collect();
        let pairs = Arc::new(pairs);
        let barrier = Arc::new(Barrier::new(2));

        for direction in 0..2 {
            let pairs = Arc::clone(&pairs);
            let barrier = Arc::clone(&barrier);
            let cycles = Arc::clone(&cycles);
            let resolved = Arc::clone(&resolved);
            let sender = sender.clone();
            std::thread::spawn(move || {
                barrier.wait();
                for (first, second) in pairs.iter() {
                    let (parent, child) = if direction == 0 {
                        (first, second)
                    } else {
                        (second, first)
                    };
                    let _ = parent.append_child_checked(child.clone());
                }
                // Cycle check: walking up from a node must never return to it.
                for (first, _) in pairs.iter() {
                    if first.is_ancestor(&first.node()) {
                        cycles.fetch_add(1, Ordering::Relaxed);
                    }
                }
                // Exactly one of the two opposite attachments can win; the loser must be
                // rejected as a cycle. Counting this in one thread only keeps it deterministic.
                if direction == 0 {
                    for (first, second) in pairs.iter() {
                        if first.parent().is_some() != second.parent().is_some() {
                            resolved.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
                sender.send(()).unwrap();
            });
        }
        wait_for(&receiver, 2);
    }
    drop(sender);

    assert_eq!(
        cycles.load(Ordering::Relaxed),
        0,
        "concurrent edits created cycles"
    );
    assert_eq!(
        resolved.load(Ordering::Relaxed),
        pairs_per_round * rounds,
        "not every pair ended with exactly one attachment, so some edits were rejected for the \
         wrong reason (or silently skipped)"
    );
}

#[test]
fn readers_and_writers_do_not_deadlock() {
    let document = Document::empty();
    let root = element(&document, "root");
    document.set_root(root.clone());
    for index in 0..16 {
        let child = element(&document, &format!("initial{index}"));
        child.append_child(document.create_text("payload").unwrap());
        root.append_child(child);
    }

    let workers = 6usize;
    let iterations = 400usize;
    let (sender, receiver) = channel();

    for worker in 0..workers {
        let document = document.clone();
        let root = root.clone();
        let sender = sender.clone();
        std::thread::spawn(move || {
            for iteration in 0..iterations {
                if worker % 2 == 0 {
                    // Writer: create, attach, clone, detach.
                    let child = element(&document, &format!("w{worker}i{iteration}"));
                    child.append_child(document.create_text("payload").unwrap());
                    root.append_child(child.clone());
                    let copy = child.deep_clone();
                    copy.detach();
                    child.detach();
                } else {
                    // Reader: traverse and serialize.
                    let children = root.children();
                    let _ = children.len();
                    let _ = root.descendants().len();
                    let _ = biodivine_lib_xml_dom::write_string(&document).unwrap();
                    let _ = document.node_count();
                }
            }
            sender.send(()).unwrap();
        });
    }

    wait_for(&receiver, workers);
    assert_consistent(&document);
}

#[test]
fn structural_edits_stay_consistent_under_contention() {
    // Several threads repeatedly move a small set of nodes around. With a single lock every move
    // is atomic, so the tree must still satisfy every invariant afterwards.
    let document = Document::empty();
    let root = element(&document, "root");
    document.set_root(root.clone());

    let slots: Vec<Element> = (0..8)
        .map(|i| element(&document, &format!("slot{i}")))
        .collect();
    for slot in &slots {
        root.append_child(slot.clone());
    }

    let workers = 4usize;
    let iterations = 300usize;
    let (sender, receiver) = channel();

    for worker in 0..workers {
        let document = document.clone();
        let root = root.clone();
        let slots = slots.clone();
        let sender = sender.clone();
        std::thread::spawn(move || {
            for iteration in 0..iterations {
                let source = &slots[(worker + iteration) % slots.len()];
                let target = &slots[(worker + iteration + 1) % slots.len()];
                let node = element(&document, "mover");
                let _ = source.append_child_checked(node.clone());
                let _ = target.append_child_checked(node.clone());
                let _ = node.detach();
                let _ = root.insert_child_checked(0, node.clone());
                let _ = node.detach();
            }
            sender.send(()).unwrap();
        });
    }

    wait_for(&receiver, workers);
    assert_consistent(&document);
}

#[test]
fn handles_can_be_moved_between_threads() {
    fn assert_send<T: Send>() {}
    assert_send::<Document>();
    assert_send::<Element>();
    assert_send::<Node>();

    fn assert_sync<T: Sync>() {}
    assert_sync::<Document>();
    assert_sync::<Element>();
    assert_sync::<Node>();

    let document = Document::empty();
    let root = element(&document, "root");
    document.set_root(root.clone());

    // Pass handles (not just the document) to other threads and edit from there.
    let workers = 4usize;
    let (sender, receiver) = channel();
    let mut handles = Vec::new();
    for worker in 0..workers {
        let name = format!("child{worker}");
        handles.push((worker, name));
    }

    let shared: Arc<Vec<(usize, String)>> = Arc::new(handles);
    for worker in 0..workers {
        let document = document.clone();
        let root = root.clone();
        let shared = Arc::clone(&shared);
        let sender = sender.clone();
        std::thread::spawn(move || {
            let (index, name) = &shared[worker];
            let child = document.create_element(QualifiedName::without_namespace(name).unwrap());
            root.append_child_checked(child.clone()).unwrap();
            let _ = index;
            sender.send(()).unwrap();
        });
    }

    wait_for(&receiver, workers);
    assert_eq!(root.child_elements().len(), workers);
    assert_consistent(&document);
}

#[test]
fn a_node_handle_remains_valid_after_being_moved_by_another_thread() {
    let document = Document::empty();
    let root = element(&document, "root");
    document.set_root(root.clone());
    let child = element(&document, "child");
    root.append_child(child.clone());

    let (sender, receiver) = channel();
    let child_for_thread = child.clone();
    std::thread::spawn(move || {
        child_for_thread.detach();
        sender.send(()).unwrap();
    });
    wait_for(&receiver, 1);

    // The handle still points at the same node; it is simply detached now.
    assert!(!child.is_attached());
    assert_eq!(child.local_name(), "child");
    assert_eq!(child.parent(), None);
    root.append_child(child.clone());
    assert!(child.is_attached());
    assert_consistent(&document);
}
