//! Counting of *external* references to an arena slot.
//!
//! Every occupied slot of an arena owns one [`ExternalRefCount`], and every handle that the outside
//! world can hold ([`Node`](crate::Node), [`Element`](crate::Element)) carries a [`NodeRef`] which
//! increments that counter while it exists. This is the only state a handle touches without the
//! arena lock, so cloning and dropping a handle is lock-free.
//!
//! Internal references are deliberately *not* counted: a parent/child link, the document root and a
//! detached subtree under construction are plain [`NodeId`](crate::NodeId) values, so a node costs
//! nothing extra for being someone's child.
//!
//! # Why relaxed ordering is enough
//!
//! The counter is a liveness *hint*, not a synchronization primitive:
//!
//! * every transition from zero to one is produced by [`crate::arena::Arena::external_ref`], which
//!   is only called while the arena read lock is held; the collector holds the write lock, so it can
//!   never observe a zero count for a slot that already had a handle when the collection started;
//! * [`NodeRef::clone`] only ever raises an already positive count, which cannot change a decision
//!   the collector has already made;
//! * the node payload itself is only ever read while the arena lock is held, so no data is published
//!   through the counter.
//!
//! The worst case of a reference being dropped while a collection is in progress is therefore that
//! the node survives that one collection and is reclaimed by the next one.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The number of live external references to one arena slot.
///
/// The arena keeps one `Arc` of this per occupied slot; the counter is expected to hold the number
/// of [`NodeRef`] values that point at it, *excluding* the arena's own `Arc`. That relationship is
/// checked by [`crate::arena::Arena::debug_check_reference_counts`].
#[derive(Debug, Default)]
pub(crate) struct ExternalRefCount {
    /// Number of live [`NodeRef`] values; see the module documentation for the ordering argument.
    count: AtomicUsize,
}

impl ExternalRefCount {
    /// Creates a counter with no external references yet (the arena does not create one).
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// The current number of live external references.
    pub(crate) fn get(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }

    /// Records one more external reference.
    fn increment(&self) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    /// Records one fewer external reference.
    fn decrement(&self) {
        self.count.fetch_sub(1, Ordering::Relaxed);
    }
}

/// One counted external reference to an arena slot.
///
/// This is what makes [`Node`](crate::Node) and [`Element`](crate::Element) keep their node alive:
/// cloning a handle clones a `NodeRef` (incrementing the count), dropping a handle drops one. Both
/// operations only touch an atomic and never acquire the arena lock.
///
/// The type is deliberately *not* `Clone` by derive and its field is private, so there is exactly
/// one place in the crate that can create a new external reference to a slot:
/// [`crate::arena::Arena::external_ref`], which is itself only reachable through the arena lock.
pub(crate) struct NodeRef {
    /// The counter of the slot, shared with the arena. Dropping the last `Arc` (that is, the slot's
    /// own copy) while the counter is non-zero is impossible: a non-zero counter means a live
    /// `NodeRef` exists, which holds an `Arc` of its own.
    counter: Arc<ExternalRefCount>,
}

impl NodeRef {
    /// Records a new external reference to the slot that owns `counter`.
    ///
    /// # Panics
    ///
    /// Panics if the counter overflows. Reaching the limit would require more than `usize::MAX`
    /// simultaneous handles to one node, so this is a guard against silent wraparound rather than a
    /// reachable condition.
    pub(crate) fn new(counter: Arc<ExternalRefCount>) -> Self {
        let current = counter.get();
        assert!(
            current < usize::MAX,
            "external reference count overflow for one arena slot"
        );
        counter.increment();
        Self { counter }
    }
}

impl Clone for NodeRef {
    fn clone(&self) -> Self {
        Self::new(Arc::clone(&self.counter))
    }
}

impl Drop for NodeRef {
    fn drop(&mut self) {
        self.counter.decrement();
    }
}

impl std::fmt::Debug for NodeRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeRef")
            .field("count", &self.counter.get())
            .finish()
    }
}
