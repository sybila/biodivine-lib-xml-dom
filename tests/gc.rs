//! Garbage collection: slot reuse, capacity growth, reachability and concurrent releases.
//!
//! These tests use only the public API. They are deliberately built around *forced* collections
//! (a tiny configured capacity) so that they exercise the collector at the moments where a mistake
//! would be visible: while a subtree is being copied into a full document, while nodes are detached
//! and re-attached, and while another thread releases references.

use std::thread;

use biodivine_lib_xml_dom::{Document, Element, GcConfig, NodeId, QualifiedName, write_string};

/// A configuration whose arena fills up quickly, so collections happen all the time.
fn small_arena() -> GcConfig {
    GcConfig::new()
        .with_initial_capacity(4)
        .with_minimum_free_fraction(0.25)
}

/// Creates a detached element with `name`.
fn element(document: &Document, name: &str) -> Element {
    document.create_element(QualifiedName::without_namespace(name).unwrap())
}

/// Builds `<root><child>text</child>...</root>` with `count` children and returns the document.
fn tree(count: usize) -> Document {
    let document = Document::empty();
    let root = element(&document, "root");
    document.set_root(root.clone());
    for index in 0..count {
        let child = element(&document, "child");
        child.set_attribute(
            QualifiedName::without_namespace("n").unwrap(),
            index.to_string(),
        );
        child.append_child(document.create_text(format!("text {index}")).unwrap());
        root.append_child(child);
    }
    document
}

#[test]
fn an_id_is_a_raw_integer_that_only_the_arena_can_resolve() {
    let document = Document::empty();
    let node = document.create_text("kept").unwrap();

    let id = node.id();
    assert_eq!(NodeId::from(u32::from(id)), id);

    // An id that no document produced simply does not resolve.
    assert!(document.node(NodeId::from(4_000_000)).is_none());
    assert!(document.node(NodeId::from(u32::MAX)).is_none());

    // A resolved id yields a handle to the very same node.
    let resolved = document.node(id).expect("the node is alive");
    assert!(resolved.ptr_eq(&node));
}

#[test]
fn a_collected_slot_is_reused_by_the_next_allocation() {
    let document = Document::empty();
    let first = document.create_text("first").unwrap();
    let first_id = first.id();

    // While the handle exists, nothing is collected.
    assert_eq!(document.collect_garbage().collected, 0);
    assert_eq!(document.node_count(), 1);

    // Dropping the handle makes the node unreachable: the collection reclaims it and the id stops
    // resolving until something else reuses the slot.
    drop(first);
    let stats = document.collect_garbage();
    assert_eq!(stats.collected, 1);
    assert_eq!(stats.live, 0);
    assert!(document.node(first_id).is_none());

    let second = document.create_text("second").unwrap();
    assert_eq!(second.id(), first_id, "the freed slot is reused");
    assert!(document.node(first_id).unwrap().ptr_eq(&second));
}

#[test]
fn the_document_root_keeps_its_tree_alive_without_any_handle() {
    let document = tree(8);
    let expected = write_string(&document).unwrap();

    // No handle to any node of the tree exists here: the document itself is the only owner.
    let stats = document.collect_garbage();
    assert_eq!(stats.collected, 0);
    assert_eq!(stats.live, 17);
    assert_eq!(write_string(&document).unwrap(), expected);
}

#[test]
fn a_handle_keeps_a_detached_subtree_alive() {
    let document = Document::empty();
    let kept = element(&document, "kept");
    let discarded = element(&document, "discarded");
    let discarded_id = discarded.id();
    assert_eq!(document.node_count(), 2);

    // Only `kept` stays referenced by a handle.
    drop(discarded);
    let stats = document.collect_garbage();
    assert_eq!(
        stats.collected, 1,
        "only the unreferenced node is reclaimed"
    );
    assert_eq!(document.node_count(), 1);
    assert!(document.node(kept.id()).is_some());
    assert!(document.node(discarded_id).is_none());
}

#[test]
fn a_handle_keeps_its_ancestors_reachable() {
    let document = Document::empty();
    let root = element(&document, "root");
    document.set_root(root.clone());
    let mut deepest = root.clone().into_node();
    for index in 0..5 {
        let child = element(&document, &format!("level{index}"));
        deepest.append_child(child.clone());
        deepest = child.into_node();
    }

    // Detach the whole chain from the document and drop every handle except the one for the
    // deepest node: that single handle now has to keep all five ancestors alive.
    let root_id = root.id();
    document.clear_root();
    drop(root);
    let stats = document.collect_garbage();
    assert_eq!(
        stats.collected, 0,
        "the detached chain is reachable through the handle"
    );

    // Walking upwards still works: the ancestors were kept alive by the handle.
    let mut current = deepest.clone();
    let mut length = 1;
    while let Some(parent) = current.parent() {
        current = parent;
        length += 1;
    }
    assert_eq!(length, 6);
    assert_eq!(current.id(), root_id);
    assert!(!current.is_attached());
}

#[test]
fn an_unreachable_detached_forest_is_reclaimed() {
    let document = tree(3);
    let rooted = document.node_count();

    // Build a whole forest and keep no handle to it.
    for index in 0..20 {
        let node = element(&document, &format!("garbage{index}"));
        node.append_child(document.create_text("payload").unwrap());
    }
    assert_eq!(document.node_count(), rooted + 40);

    let stats = document.collect_garbage();
    assert_eq!(stats.collected, 40);
    assert_eq!(document.node_count(), rooted);
    assert!(document.validate().is_ok());
}

#[test]
fn the_arena_grows_when_a_collection_cannot_free_enough_room() {
    let document = Document::with_gc_config(small_arena());
    assert_eq!(document.capacity(), 4);

    // Every node is referenced, so the collection cannot free anything and the arena has to grow.
    let handles: Vec<_> = (0..4)
        .map(|index| document.create_text(format!("{index}")).unwrap())
        .collect();
    assert_eq!(document.capacity(), 4);

    let extra = document.create_text("triggers growth").unwrap();
    assert!(
        document.capacity() > 4,
        "a collection that frees nothing must grow the arena"
    );
    assert_eq!(document.node_count(), 5);
    assert!(
        handles
            .iter()
            .all(|handle| document.node(handle.id()).is_some())
    );
    assert!(!extra.is_attached());
}

#[test]
fn a_manual_collection_does_not_grow_the_capacity() {
    let document = Document::with_gc_config(small_arena());
    let capacity = document.capacity();
    // Hold the handles, so the nodes survive, and stay within the capacity.
    let handles: Vec<_> = (0..4)
        .map(|index| document.create_text(format!("{index}")).unwrap())
        .collect();
    let stats = document.collect_garbage();
    assert_eq!(stats.collected, 0);
    assert_eq!(stats.capacity, capacity);
    assert_eq!(stats.free, 0);
    assert_eq!(handles.len(), 4);
}

#[test]
fn stats_describe_the_capacity_policy() {
    let document = tree(5);
    let stats = document.collect_garbage();
    assert_eq!(stats.capacity, document.capacity());
    assert_eq!(stats.live, document.node_count());
    assert_eq!(stats.capacity, stats.live + stats.free);
    let expected = (stats.free as f64) / (stats.capacity as f64);
    assert!((stats.free_fraction() - expected).abs() < f64::EPSILON);
}

#[test]
fn a_collection_during_a_cross_document_copy_keeps_the_copy_intact() {
    let source = tree(30);
    let expected = write_string(&source).unwrap();

    // The target arena is far too small for the copy and is full of garbage, so inserting the
    // subtree has to collect and grow before it can allocate the copied nodes.
    let target = Document::with_gc_config(small_arena());
    for index in 0..4 {
        // Dropping the handle makes each node garbage immediately.
        let garbage = element(&target, &format!("garbage{index}"));
        garbage.append_child(target.create_text("payload").unwrap());
    }

    let copy = source.root().unwrap().deep_clone_into(&target);
    assert!(
        !copy.is_attached(),
        "a copy is detached until it is attached"
    );

    // The copy is complete: same node count and the same serialization as the source.
    assert_eq!(target.node_count(), source.node_count());
    assert_eq!(copy.as_element().unwrap().to_string(), expected);

    let copy_as_root = copy.as_element().unwrap();
    target.set_root(copy_as_root);
    assert_eq!(write_string(&target).unwrap(), expected);
    assert!(target.validate().is_ok());
}

#[test]
fn a_collection_during_a_same_document_copy_keeps_the_copy_intact() {
    let document = tree(20);
    let root = document.root().unwrap();
    let before = document.node_count();

    let copy = root.deep_clone();
    // A detached copy is referenced by its handle, so a collection leaves it alone.
    assert_eq!(document.collect_garbage().collected, 0);
    assert_eq!(document.node_count(), 2 * before);
    assert_eq!(copy.children().len(), 20);
    assert_eq!(copy.as_element().unwrap().to_string(), root.to_string());
    assert!(!copy.is_attached());
}

#[test]
fn clearing_and_replacing_the_root_leaves_the_returned_subtree_alive() {
    let document = tree(4);
    let previous = document.clear_root().expect("the tree has a root");
    assert!(document.root().is_none());

    let stats = document.collect_garbage();
    assert_eq!(stats.collected, 0, "the returned handle keeps the old tree");
    assert_eq!(previous.children().len(), 4);

    // The detached tree can be installed as the root again.
    document.set_root(previous.clone());
    assert!(document.root().unwrap().ptr_eq(&previous));
}

#[test]
fn nodes_enumerates_only_the_live_set() {
    let document = tree(3);
    let detached = element(&document, "detached");
    let total = document.node_count();
    let with_detached = document.nodes();
    assert_eq!(with_detached.len(), total);
    drop(with_detached);

    drop(detached);
    assert_eq!(document.collect_garbage().collected, 1);
    let live = document.nodes();
    assert_eq!(live.len(), total - 1);
    for node in &live {
        assert!(document.node(node.id()).is_some());
    }
}

#[test]
fn handles_released_by_another_thread_are_safe_while_collecting() {
    let document = tree(10);
    let workers: Vec<_> = (0..4)
        .map(|index| {
            let document = document.clone();
            thread::spawn(move || {
                for round in 0..200 {
                    let node = document
                        .create_text(format!("thread {index} round {round}"))
                        .unwrap();
                    let child = document
                        .create_element(QualifiedName::without_namespace("temporary").unwrap());
                    child.append_child(node);
                    // Dropping both handles at the end of the iteration makes them collectable.
                }
            })
        })
        .collect();

    let collector = {
        let document = document.clone();
        thread::spawn(move || {
            let mut total = 0;
            for _ in 0..200 {
                total += document.collect_garbage().collected;
            }
            total
        })
    };

    for worker in workers {
        worker.join().expect("a worker thread panicked");
    }
    let collected_during_the_run = collector.join().expect("the collector thread panicked");

    // The rooted tree is untouched by all that collecting and allocating.
    assert_eq!(document.root().unwrap().children().len(), 10);
    assert!(document.validate().is_ok());

    // Once every worker is gone the temporaries are unreachable, so a single collection now
    // reclaims whatever survived a cycle it raced with, and a second one has nothing left to do.
    // (A node released while a collection was reading the reference count survives that collection
    // and is reclaimed by a later one, which is why the running collector may not have seen it.)
    let leftovers = document.collect_garbage().collected;
    assert_eq!(document.node_count(), 21, "only the rooted tree is left");
    assert_eq!(document.collect_garbage().collected, 0);
    assert_eq!(document.root().unwrap().children().len(), 10);
    assert!(
        collected_during_the_run + leftovers > 0,
        "neither the collector thread nor the final collection reclaimed anything"
    );
}

#[test]
fn a_collection_does_not_disturb_document_identity_or_handles() {
    let document = tree(2);
    let root = document.root().unwrap();
    let child = root.children()[0].clone();

    let clone = document.clone();
    assert_eq!(document.collect_garbage().collected, 0);
    assert!(clone.ptr_eq(&document));
    assert!(root.belongs_to(&clone));
    assert!(child.parent().unwrap().ptr_eq(&root));
    assert_eq!(child.parent().unwrap().id(), root.id());
}
