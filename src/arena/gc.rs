//! Mark-and-sweep collection for [`Arena`].
//!
//! The collector answers one question: *which nodes can the outside world still reach?* The roots of
//! the mark phase are
//!
//! * every occupied slot with a positive external reference count, and
//! * the document root, which the arena owns itself.
//!
//! From those, reachability follows **both** the parent and the child links. The parent direction
//! matters: a handle to a text node deep inside a tree must also keep the ancestors alive, because
//! the handle's methods (`parent`, `is_attached`, namespace resolution) walk upwards. The child
//! direction matters because a handle to an element keeps its whole subtree usable.
//!
//! Everything that is not marked is unreachable by construction, so its payload is dropped and its
//! slot goes onto the free list. Collection never shrinks the capacity and never compacts: an id
//! keeps referring to the same slot, which may then hold a different node.

#[cfg(debug_assertions)]
use std::sync::Arc;

use crate::arena::config::free_fraction;
use crate::arena::{Arena, GcStats, NodeData, Slot};
use crate::error::XmlResult;
use crate::node_id::NodeId;

impl Arena {
    /// Runs a collection and reports what it did.
    ///
    /// The capacity is never changed by a collection; growth is the business of the allocator
    /// (see [`Arena::alloc`]). References that are dropped by another thread while this runs are
    /// simply not observed, so the node they referred to may survive this collection and be
    /// reclaimed by the next one.
    pub(crate) fn collect(&mut self) -> GcStats {
        // The allocator calls this path, and allocation is infallible, so cancellation is
        // deliberately not consulted here: a collection that is skipped or interrupted only leaves
        // unreachable nodes for the next collection, it never affects correctness.
        // `collect_internal(false)` therefore cannot return an error.
        let collected = self
            .collect_internal(false)
            .expect("a collection with cancellation disabled cannot be cancelled");
        self.stats(collected)
    }

    /// Runs a collection that honours cancellation triggers registered on this thread.
    ///
    /// Returns [`crate::XmlError::Cancelled`] if such a trigger fires while the arena is being
    /// marked or swept. A cancelled collection leaves the arena consistent; the nodes it did not get
    /// to are reclaimed by the next collection.
    pub(crate) fn collect_cancellable(&mut self) -> XmlResult<GcStats> {
        let collected = self.collect_internal(true)?;
        Ok(self.stats(collected))
    }

    fn stats(&self, collected: usize) -> GcStats {
        GcStats {
            capacity: self.capacity(),
            live: self.live(),
            free: self.free_slots(),
            collected,
        }
    }

    /// The collector proper: mark, sweep, and check the invariants in debug builds.
    fn collect_internal(&mut self, cancellable: bool) -> XmlResult<usize> {
        let marked = self.mark(cancellable)?;
        let collected = self.sweep(&marked, cancellable)?;
        #[cfg(debug_assertions)]
        self.debug_check_reference_counts();
        Ok(collected)
    }

    /// Marks every node reachable from an external reference or from the document root.
    fn mark(&self, cancellable: bool) -> XmlResult<Vec<bool>> {
        let mut marked = vec![false; self.slots.len()];
        let mut stack: Vec<NodeId> = Vec::new();

        for (index, slot) in self.slots.iter().enumerate() {
            if cancellable {
                cancel_this::is_cancelled!()?;
            }
            if let Some(node) = slot.occupied()
                && node.external.get() > 0
            {
                marked[index] = true;
                stack.push(NodeId::from_index(index));
            }
        }
        if let Some(root) = self.root {
            self.push_marked(root, &mut marked, &mut stack);
        }

        while let Some(id) = stack.pop() {
            if cancellable {
                cancel_this::is_cancelled!()?;
            }
            let Some(slot) = self.occupied(id) else {
                continue;
            };
            if let Some(parent) = slot.parent {
                self.push_marked(parent, &mut marked, &mut stack);
            }
            if let NodeData::Element(element) = &slot.data {
                for &child in &element.children {
                    self.push_marked(child, &mut marked, &mut stack);
                }
            }
        }
        Ok(marked)
    }

    /// Marks `id` and schedules it for traversal unless it already is marked.
    fn push_marked(&self, id: NodeId, marked: &mut [bool], stack: &mut Vec<NodeId>) {
        if self.occupied(id).is_none() {
            // Unreachable given the arena invariants; ignoring a dangling link is still better than
            // panicking from inside the collector.
            return;
        }
        if let Some(entry) = marked.get_mut(id.index())
            && !*entry
        {
            *entry = true;
            stack.push(id);
        }
    }

    /// Frees every unmarked slot and returns how many there were.
    fn sweep(&mut self, marked: &[bool], cancellable: bool) -> XmlResult<usize> {
        let mut collected = 0;
        for index in 0..self.slots.len() {
            if cancellable {
                cancel_this::is_cancelled!()?;
            }
            if marked.get(index).copied().unwrap_or(false) {
                continue;
            }
            if self.slots[index].occupied().is_some() {
                self.slots[index] = Slot::Free;
                self.free.push(NodeId::from_index(index));
                self.live -= 1;
                collected += 1;
            }
        }
        if let Some(root) = self.root
            && self.occupied(root).is_none()
        {
            // The root is a mark root, so this can only happen if the root was replaced by another
            // one between the mark and the sweep phase; keeping a `root` that points at a free slot
            // would break invariant 2.
            self.root = None;
        }
        #[cfg(debug_assertions)]
        self.debug_check_invariants();
        Ok(collected)
    }

    /// Checks every structural invariant of the arena.
    ///
    /// This is a debugging aid: it is compiled into debug builds only and called after every
    /// collection, so a bookkeeping bug in the collector fails loudly in tests instead of producing a
    /// document that is subtly wrong. It is safe to call while other threads hold handles: it does
    /// not look at reference counts (see [`Arena::debug_check_reference_counts`] for that).
    #[cfg(debug_assertions)]
    pub(crate) fn debug_check_invariants(&self) {
        let occupied = self
            .slots
            .iter()
            .filter(|slot| slot.occupied().is_some())
            .count();
        assert_eq!(
            occupied, self.live,
            "the live count disagrees with the occupied slots"
        );

        // The free list contains exactly the free slots, without duplicates.
        let mut listed = vec![false; self.slots.len()];
        for &id in &self.free {
            assert!(
                id.index() < self.slots.len(),
                "the free list contains an out-of-range id"
            );
            assert!(!listed[id.index()], "the free list contains a duplicate");
            assert!(
                self.occupied(id).is_none(),
                "the free list contains an occupied slot"
            );
            listed[id.index()] = true;
        }
        for (index, slot) in self.slots.iter().enumerate() {
            assert_eq!(
                slot.occupied().is_none(),
                listed[index],
                "a free slot is missing from the free list"
            );
        }

        // The root is an element without a parent and never a child.
        if let Some(root) = self.root {
            assert!(self.occupied(root).is_some(), "the root slot is free");
            assert!(self.is_element(root), "the root is not an element");
            assert!(self.parent(root).is_none(), "the root has a parent");
        }

        // Parent and child links agree, and neither points at a free slot.
        for (index, slot) in self.slots.iter().enumerate() {
            let Some(node) = slot.occupied() else {
                continue;
            };
            let id = NodeId::from_index(index);
            if let Some(parent) = node.parent {
                assert!(
                    self.occupied(parent).is_some(),
                    "a parent link points at a free slot"
                );
                assert!(
                    self.children(parent).contains(&id),
                    "a child does not appear in its parent's child list"
                );
            }
            if let NodeData::Element(element) = &node.data {
                for &child in &element.children {
                    assert!(
                        self.occupied(child).is_some(),
                        "a child link points at a free slot"
                    );
                    assert_eq!(
                        self.parent(child),
                        Some(id),
                        "a child does not point back at its parent"
                    );
                    assert!(child != id, "a node is its own child");
                }
            }
        }
    }

    /// Checks that every slot's reference count matches the number of handles that hold it.
    ///
    /// The check compares the atomic counter with the strong count of the slot's `Arc`; the arena
    /// itself holds exactly one strong reference, so a quiescent slot must satisfy
    /// `counter + 1 == strong_count`. The exact equality only holds while no other thread is
    /// cloning or dropping handles, so the collector runs the weaker inequality (see
    /// [`Arena::collect_internal`]) and this function is meant for tests that own all handles.
    #[cfg(debug_assertions)]
    pub(crate) fn debug_check_reference_counts(&self) {
        for slot in self.slots.iter().filter_map(Slot::occupied) {
            let strong = Arc::strong_count(&slot.external);
            let counted = slot.external.get();
            assert!(
                counted <= strong.saturating_sub(1),
                "more external references were counted than there are handles for them"
            );
        }
    }

    /// The exact form of [`Arena::debug_check_reference_counts`], for quiescent tests.
    ///
    /// Only the crate's own unit tests can call this, and only in a debug build: the equality it
    /// asserts does not hold while another thread may be dropping handles.
    #[cfg(all(test, debug_assertions))]
    pub(crate) fn debug_check_reference_counts_exactly(&self) {
        self.debug_check_reference_counts();
        for slot in self.slots.iter().filter_map(Slot::occupied) {
            let strong = Arc::strong_count(&slot.external);
            let counted = slot.external.get();
            assert_eq!(
                counted,
                strong - 1,
                "the external reference count of a slot is out of step with its handles"
            );
        }
    }

    /// The free fraction of the capacity; used by the allocator's growth decision.
    pub(crate) fn capacity_free_fraction(&self) -> f64 {
        free_fraction(self.free_slots(), self.capacity())
    }
}

#[cfg(all(test, debug_assertions))]
mod tests {
    use super::*;
    use crate::arena::{ElementData, GcConfig, NodeData};
    use crate::qualified_name::QualifiedName;
    use crate::xml_spec::Text;
    use std::collections::BTreeMap;

    /// An arena that holds at most two slots, so that nearly every allocation collects.
    fn tiny_arena() -> Arena {
        Arena::with_config(
            GcConfig::new()
                .with_initial_capacity(2)
                .with_minimum_free_fraction(0.5),
        )
    }

    fn element(arena: &mut Arena, name: &str) -> NodeId {
        arena.alloc_element(ElementData {
            name: QualifiedName::without_namespace(name).unwrap(),
            attributes: BTreeMap::new(),
            namespace_declarations: BTreeMap::new(),
            children: Vec::new(),
        })
    }

    /// The reference-count bookkeeping has to stay exactly in step with the handles that exist:
    /// this is the check that catches a forgotten increment or a double decrement.
    #[test]
    fn reference_counts_match_the_handles_exactly_when_quiescent() {
        let mut arena = tiny_arena();
        let parent = element(&mut arena, "parent");
        let parent_ref = arena.external_ref(parent).expect("occupied");
        let child = element(&mut arena, "child");
        arena.append(parent, child).unwrap();
        arena.set_root(parent).unwrap();

        let child_ref = arena.external_ref(child).expect("occupied");
        arena.debug_check_reference_counts_exactly();

        // Cloning a handle raises the count of exactly one slot.
        let clone = parent_ref.clone();
        arena.debug_check_reference_counts_exactly();

        drop(clone);
        drop(child_ref);
        arena.debug_check_reference_counts_exactly();

        // Even without any handle the tree stays alive, because the document root is a mark root.
        drop(parent_ref);
        assert_eq!(arena.collect().collected, 0);

        // Clearing the root removes the last path to the nodes, so the next collection reclaims
        // both of them and the slots are reused.
        arena.clear_root();
        let stats = arena.collect();
        assert_eq!(
            stats.collected, 2,
            "the root and its child are both unreachable"
        );
        arena.debug_check_reference_counts_exactly();
        assert!(arena.occupied_ids().next().is_none());

        let reused = arena.alloc(NodeData::Text(Text::try_from("text").unwrap()));
        assert_eq!(reused, child, "the free list is LIFO");
        arena.debug_check_reference_counts_exactly();
    }

    /// A collection keeps a referenced node and reclaims everything else, and it is idempotent.
    #[test]
    fn collection_is_conservative_about_references_and_idempotent() {
        let mut arena = tiny_arena();
        let referenced = element(&mut arena, "referenced");
        let handle = arena.external_ref(referenced).expect("occupied");
        let garbage = element(&mut arena, "garbage");

        let first = arena.collect();
        assert_eq!(first.collected, 1);
        assert!(arena.occupied(referenced).is_some());
        assert!(arena.occupied(garbage).is_none());

        let second = arena.collect();
        assert_eq!(
            second.collected, 0,
            "a second collection finds nothing to do"
        );
        assert_eq!(second.live, 1);
        assert_eq!(second.free, 1);
        // The handle is still usable: the slot it refers to was not reclaimed.
        assert!(arena.external_ref(referenced).is_some());
        drop(handle);
    }

    /// The arena grows when a collection cannot free enough room and stops at the limit.
    #[test]
    fn growth_happens_only_when_a_collection_is_not_enough() {
        let mut arena = tiny_arena();
        let first = element(&mut arena, "first");
        let first_ref = arena.external_ref(first).expect("occupied");
        let second = element(&mut arena, "second");
        let second_ref = arena.external_ref(second).expect("occupied");

        let capacity = arena.capacity();
        // Both slots are referenced, so the arena has to grow to satisfy the next allocation.
        let third = element(&mut arena, "third");
        assert!(arena.capacity() > capacity);
        assert!(arena.occupied(third).is_some());
        drop(first_ref);
        drop(second_ref);
        arena.debug_check_reference_counts_exactly();
    }

    /// Bulk construction reserves its slots up front, so a collection can never run while a
    /// detached subtree is half built.
    #[test]
    fn bulk_construction_reserves_its_slots_before_allocating() {
        let mut arena = tiny_arena();
        let root = element(&mut arena, "root");
        // The root has to be externally referenced while the tree is built: otherwise the very
        // first collection (which the tiny capacity triggers immediately) reclaims it.
        let source_ref = arena.external_ref(root).expect("occupied");
        for index in 0..6 {
            let child = element(&mut arena, &format!("child{index}"));
            arena.append(root, child).unwrap();
        }
        arena.set_root(root).unwrap();

        // The snapshot is taken from the rooted tree, inserted into an arena that is far too small,
        // and the copy must be complete afterwards.
        let snapshot = arena.snapshot(root, true).unwrap();
        let expected = snapshot.nodes.len();
        let copy = arena.insert_snapshot(snapshot);
        assert_eq!(arena.children(copy).len(), 6);
        assert_eq!(arena.parent(arena.children(copy)[0]), Some(copy));
        assert!(arena.capacity() >= expected);
        drop(source_ref);
        arena.debug_check_reference_counts_exactly();
    }
}
