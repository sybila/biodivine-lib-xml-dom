//! The node arena of a single [`crate::Document`].
//!
//! All node payloads of one document live in one [`Arena`]: a vector of slots plus a document root.
//! Everything that refers to a node does so through an index ([`NodeId`]), so parent/child links are
//! plain integers, there is no reference cycle, and every structural invariant is maintained by this
//! module under a single `&mut self` borrow. A half-linked state is therefore unrepresentable
//! outside of it, and no check-then-act sequence can be interleaved by another thread.
//!
//! # Slots, ids and reuse
//!
//! The arena never *compacts*: a slot index always keeps its meaning, so no id ever moves, and the
//! arena can only grow. Slots are reused, though. A slot whose node has become unreachable is
//! reclaimed by the collector and handed out again by a later allocation, which is why an id is only
//! usable while the node it was created for is alive (see [`NodeId`]).
//!
//! # External references and the collector
//!
//! The only things that keep a node alive are
//!
//! 1. **external references** — the handles that the outside world holds, counted per slot with a
//!    lock-free atomic ([`crate::ref_count`]), and
//! 2. **the document root**, which the arena itself owns as an implicit root of the mark phase.
//!
//! Internal links do not count: a detached subtree built by a copy operation or by the parser does
//! not become reachable merely because it exists. Consequently, when a collection runs, it keeps
//! every node reachable from the root or from a handle (following both parent and child links, so a
//! handle to a deep node also keeps its ancestors) and reclaims everything else.
//!
//! The two invariants that make this sound are enforced in the code:
//!
//! * *a slot can only become externally referenced while the arena lock is held* — the sole way to
//!   create a handle is [`Arena::external_ref`], which is called inside a critical section, so the
//!   collector (which holds the write lock) can never miss a fresh reference;
//! * *a bare [`NodeId`] never crosses a critical section into a context that allocates later* —
//!   every operation that hands out a node for later use creates its handle inside the same
//!   critical section, so no collection can run between "the node exists" and "the node is
//!   referenced". Bulk construction ([`Arena::insert_snapshot`]) reserves its slot count up front
//!   for the same reason.
//!
//! # Capacity policy
//!
//! An allocation reuses a free slot, then takes a fresh one, and only when the arena is full does it
//! collect and possibly grow the capacity (see [`GcConfig`] for the thresholds). Collection is
//! therefore automatic, and [`Arena::collect`] exposes it for manual use. The name interner and the
//! capacity itself never shrink: releasing memory to the allocator is the one thing this design
//! deliberately does not do.
//!
//! # Invariants maintained by [`Arena`]
//!
//! 1. `slot.parent == Some(p)` if and only if `slot` occurs exactly once in the child list of the
//!    element `p`.
//! 2. `root` is an element whose `parent` is `None`.
//! 3. No node is reachable from itself (the tree is acyclic).
//! 4. `root` never occurs in anybody's child list.
//! 5. A slot is either free or occupied, never both, and the free list contains exactly the free
//!    slots.
//!
//! All mutating methods validate *everything they can fail on* first and only then perform
//! infallible index writes. A failed operation therefore never changes the arena.

mod config;
mod gc;
mod slot;
mod snapshot;

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::error::{XmlError, XmlResult};
use crate::interner::Interner;
use crate::namespace::Namespace;
use crate::node_id::{MAX_NODES, NodeId};
use crate::qualified_name::QualifiedName;
use crate::ref_count::{ExternalRefCount, NodeRef};
use crate::xml_spec::{NCName, XmlDeclaration};

pub use crate::arena::config::{GcConfig, GcStats};
pub(crate) use crate::arena::slot::{ElementData, NodeData, NodeSlot, Slot};
pub(crate) use crate::arena::snapshot::{Snapshot, SnapshotNode};

/// All slots of one document, plus its root, its XML declaration, its name interner and its
/// collector configuration.
#[derive(Debug)]
pub(crate) struct Arena {
    /// The slots. Its length is the high-water mark of used slots: a slot at or beyond `capacity`
    /// cannot exist, because reaching `capacity` triggers a collection and a growth.
    slots: Vec<Slot>,
    /// The ids of the free slots, in LIFO order so that the most recently freed slot is reused
    /// first (which keeps the working set of a document that repeatedly builds and discards one
    /// subtree small).
    free: Vec<NodeId>,
    /// The number of occupied slots.
    live: usize,
    /// The number of slots the arena may hold before it considers itself full.
    capacity: usize,
    /// The document root, if any. It is a mark root of the collector.
    root: Option<NodeId>,
    /// The XML declaration of the document, if any. It is not a node and is never collected.
    declaration: Option<XmlDeclaration>,
    /// The interner of names and namespaces used by this document.
    interner: Interner,
    /// The capacity policy and collector thresholds of this document.
    config: GcConfig,
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

impl Arena {
    /// Creates an empty arena with the default capacity policy.
    pub(crate) fn new() -> Self {
        Self::with_config(GcConfig::DEFAULT)
    }

    /// Creates an empty arena with the given capacity policy.
    pub(crate) fn with_config(config: GcConfig) -> Self {
        Self {
            slots: Vec::new(),
            free: Vec::new(),
            live: 0,
            capacity: config.initial_capacity().min(MAX_NODES),
            root: None,
            declaration: None,
            interner: Interner::default(),
            config,
        }
    }

    // -------------------------------------------------------------------------------------
    // Capacity and configuration
    // -------------------------------------------------------------------------------------

    /// The capacity policy of this document.
    pub(crate) fn config(&self) -> GcConfig {
        self.config
    }

    /// Replaces the capacity policy, raising the capacity if the new initial capacity is larger.
    ///
    /// The capacity never shrinks, so a configuration that asks for a smaller initial capacity only
    /// changes the thresholds of the next growth.
    pub(crate) fn set_config(&mut self, config: GcConfig) {
        self.config = config;
        self.capacity = self.capacity.max(config.initial_capacity().min(MAX_NODES));
    }

    /// The number of slots this arena may hold before it considers itself full.
    pub(crate) fn capacity(&self) -> usize {
        self.capacity
    }

    /// The number of occupied slots.
    pub(crate) fn live(&self) -> usize {
        self.live
    }

    /// The number of slots that can be allocated without growing the arena.
    pub(crate) fn free_slots(&self) -> usize {
        self.capacity - self.live
    }

    // -------------------------------------------------------------------------------------
    // Allocation
    // -------------------------------------------------------------------------------------

    /// Stores `data` in a slot and returns its id.
    ///
    /// The new node is detached: it has no parent, is not the document root and has no external
    /// reference, so it is only kept alive once the caller turns the id into a handle *inside the
    /// same critical section*.
    ///
    /// # Panics
    ///
    /// Panics if the arena has reached [`MAX_NODES`] live nodes and a collection cannot free one.
    /// This is unreachable in practice (it requires more than four billion live nodes in one
    /// document) and is checked explicitly so that the `u32` index never wraps silently.
    pub(crate) fn alloc(&mut self, data: NodeData) -> NodeId {
        let id = self.reserve_slot();
        self.slots[id.index()] = Slot::Occupied(NodeSlot {
            parent: None,
            data,
            external: ExternalRefCount::new(),
        });
        self.live += 1;
        id
    }

    /// Stores an element payload, interning its name, attributes and namespace declarations.
    pub(crate) fn alloc_element(&mut self, element: ElementData) -> NodeId {
        let element = self.intern_element(element);
        self.alloc(NodeData::Element(element))
    }

    /// Returns a slot for a new node, collecting and growing the arena if it is full.
    ///
    /// # Panics
    ///
    /// Panics if the arena cannot grow any further (see [`Arena::alloc`]).
    fn reserve_slot(&mut self) -> NodeId {
        loop {
            if let Some(id) = self.free.pop() {
                return id;
            }
            if self.slots.len() < self.capacity {
                let id = NodeId::from_index(self.slots.len());
                self.slots.push(Slot::Free);
                return id;
            }
            // The arena is full: reclaim what is unreachable, and grow if that was not enough.
            self.collect_internal();
            let grow = self.free.is_empty()
                || self.capacity_free_fraction() < self.config.minimum_free_fraction();
            if grow && !self.grow() {
                // The capacity is at `MAX_NODES` and the collection freed nothing, so there is
                // nothing left to grow into. Panicking here is what makes the documented behaviour
                // of [`Arena::alloc`] true instead of spinning forever.
                panic!("document arena exhausted: a document can hold at most {MAX_NODES} nodes");
            }
        }
    }

    /// Makes room for `count` new slots without running a collection in the middle.
    ///
    /// This is what makes bulk construction safe: a subtree is built by allocating its nodes one by
    /// one, and none of those nodes is externally referenced yet, so a collection between two
    /// allocations would reclaim the nodes that were already built. Reserving the slots up front
    /// means the collection (and the growth) happen before the first allocation, and the allocation
    /// loop cannot be interrupted.
    ///
    /// # Panics
    ///
    /// Panics if `count` slots cannot be made available because the arena is limited to [`MAX_NODES`]
    /// slots (in other words, if `count` exceeds what is left of the limit).
    fn reserve_slots(&mut self, count: usize) {
        while self.free_slots() < count {
            self.collect_internal();
            if self.free_slots() >= count {
                return;
            }
            if !self.grow() {
                panic!(
                    "document arena exhausted: a document can hold at most {MAX_NODES} nodes, \
                     and {count} more slots were requested"
                );
            }
        }
    }

    /// Grows the capacity by the configured factor, capped at [`MAX_NODES`].
    ///
    /// Returns `false` if the capacity is already at the limit.
    fn grow(&mut self) -> bool {
        if self.capacity >= MAX_NODES {
            return false;
        }
        let grown = self
            .capacity
            .saturating_mul(self.config.growth_factor())
            .max(self.capacity + 1)
            .min(MAX_NODES);
        self.capacity = grown;
        true
    }

    /// Returns the document's canonical value for `name`.
    pub(crate) fn intern_name(&mut self, name: QualifiedName) -> QualifiedName {
        self.interner.name(name)
    }

    /// Returns the document's canonical value for `namespace`.
    pub(crate) fn intern_namespace(&mut self, namespace: Namespace) -> Namespace {
        self.interner.namespace(namespace)
    }

    /// Interns everything inside `element` that is stored in the arena.
    ///
    /// Attribute *values* are `Arc<str>` and are intentionally not deduplicated by value: they are
    /// frequently unique, so a per-document map of all values would be pure overhead. They are still
    /// shared whenever a node is cloned.
    fn intern_element(&mut self, element: ElementData) -> ElementData {
        let mut attributes = BTreeMap::new();
        for (name, value) in element.attributes {
            let name = self.interner.name(name);
            attributes.insert(name, value);
        }
        let mut namespace_declarations = BTreeMap::new();
        for (prefix, namespace) in element.namespace_declarations {
            let namespace = namespace.map(|ns| self.interner.namespace(ns));
            namespace_declarations.insert(prefix, namespace);
        }
        ElementData {
            name: self.interner.name(element.name),
            attributes,
            namespace_declarations,
            children: element.children,
        }
    }

    // -------------------------------------------------------------------------------------
    // Read access
    // -------------------------------------------------------------------------------------

    /// The document root, if any.
    pub(crate) fn root(&self) -> Option<NodeId> {
        self.root
    }

    /// The occupied slot of `id`, or `None` if the slot is free or out of range.
    pub(crate) fn occupied(&self, id: NodeId) -> Option<&NodeSlot> {
        self.slots.get(id.index()).and_then(Slot::occupied)
    }

    /// Creates a counted external reference to `id`, or returns `None` if the slot holds no node.
    ///
    /// This is the only way an external reference comes into existence, and it is deliberately an
    /// arena method: callers hold the arena lock while they call it, so a slot cannot be reclaimed
    /// between the lookup and the creation of the handle. The returned [`NodeRef`] owns one count,
    /// which nothing but cloning and dropping the handle changes (see [`crate::ref_count`]).
    pub(crate) fn external_ref(&self, id: NodeId) -> Option<NodeRef> {
        let slot = self.occupied(id)?;
        Some(NodeRef::new(Arc::clone(&slot.external)))
    }

    /// The ids of all occupied slots, in allocation order.
    pub(crate) fn occupied_ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| slot.occupied().is_some())
            .map(|(index, _)| NodeId::from_index(index))
    }

    /// The slot of `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` does not refer to an occupied slot of this arena. Every id used internally is
    /// produced by this arena for a node that the caller holds a handle to, and
    /// [`crate::Document::node`] rejects ids that come from the outside.
    pub(crate) fn slot(&self, id: NodeId) -> &NodeSlot {
        self.occupied(id)
            .expect("node id does not refer to a live node of this document")
    }

    /// The mutable slot of `id`. See [`Arena::slot`] for the panic conditions.
    fn slot_mut(&mut self, id: NodeId) -> &mut NodeSlot {
        self.slots
            .get_mut(id.index())
            .and_then(Slot::occupied_mut)
            .expect("node id does not refer to a live node of this document")
    }

    /// The payload of `id`. See [`Arena::slot`] for the panic conditions.
    pub(crate) fn data(&self, id: NodeId) -> &NodeData {
        &self.slot(id).data
    }

    /// The element payload of `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is not an element. Call sites either know the kind statically or check it first
    /// (e.g. via [`Arena::is_element`]).
    pub(crate) fn element(&self, id: NodeId) -> &ElementData {
        match self.data(id) {
            NodeData::Element(element) => element,
            other => panic!("node {id} is not an element but {other:?}"),
        }
    }

    /// The mutable element payload of `id`. See [`Arena::element`] for the panic conditions.
    pub(crate) fn element_mut(&mut self, id: NodeId) -> &mut ElementData {
        match &mut self.slot_mut(id).data {
            NodeData::Element(element) => element,
            other => panic!("node {id} is not an element but {other:?}"),
        }
    }

    /// Whether `id` is an element. Returns `false` for a free or out-of-range slot.
    pub(crate) fn is_element(&self, id: NodeId) -> bool {
        matches!(
            self.occupied(id),
            Some(NodeSlot {
                data: NodeData::Element(_),
                ..
            })
        )
    }

    /// The parent of `id`.
    pub(crate) fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.slot(id).parent
    }

    /// The children of `id`, empty for non-elements.
    pub(crate) fn children(&self, id: NodeId) -> &[NodeId] {
        match self.data(id) {
            NodeData::Element(element) => &element.children,
            _ => &[],
        }
    }

    /// Whether `ancestor` is a strict ancestor of `node`.
    ///
    /// The walk is bounded by the number of slots in the arena. Invariant 3 says the tree is
    /// acyclic, so the bound is never reached; it exists so that even a hypothetical bug cannot turn
    /// this into an infinite loop.
    pub(crate) fn is_ancestor(&self, ancestor: NodeId, node: NodeId) -> bool {
        let mut current = self.slot(node).parent;
        for _ in 0..self.slots.len() {
            match current {
                Some(parent) if parent == ancestor => return true,
                Some(parent) => current = self.slot(parent).parent,
                None => return false,
            }
        }
        false
    }

    /// Whether `node` is reachable from the document root.
    pub(crate) fn is_attached(&self, node: NodeId) -> bool {
        match self.root {
            Some(root) => node == root || self.is_ancestor(root, node),
            None => false,
        }
    }

    /// All bindings visible to `id`, innermost first.
    ///
    /// Namespace declarations are inherited along the ancestor chain
    /// (`rule.namespace-usage.prefix-declaration-scope`), and a declaration on an inner element
    /// shadows one on an outer element. The walk is bounded like [`Arena::is_ancestor`].
    pub(crate) fn namespaces_in_scope(
        &self,
        id: NodeId,
    ) -> Vec<(Option<NCName>, Option<Namespace>)> {
        let mut result: Vec<(Option<NCName>, Option<Namespace>)> = Vec::new();
        let mut seen: Vec<Option<NCName>> = Vec::new();
        let mut current = Some(id);
        for _ in 0..self.slots.len() {
            let Some(node) = current else { break };
            if let NodeData::Element(element) = self.data(node) {
                for (prefix, namespace) in &element.namespace_declarations {
                    if !seen.contains(prefix) {
                        seen.push(prefix.clone());
                        result.push((prefix.clone(), namespace.clone()));
                    }
                }
            }
            current = self.slot(node).parent;
        }
        result
    }

    /// Resolves a prefix in the scope of `id` (`None` = the default namespace).
    ///
    /// Iterative on purpose: an implementation that recursed through handles would limit the usable
    /// document depth by the stack size.
    pub(crate) fn resolve_prefix(
        &self,
        id: NodeId,
        prefix: Option<&NCName>,
    ) -> Option<Option<Namespace>> {
        let mut current = Some(id);
        for _ in 0..self.slots.len() {
            let Some(node) = current else { break };
            if let NodeData::Element(element) = self.data(node)
                && let Some(binding) = element.namespace_declarations.get(&prefix.cloned())
            {
                return Some(binding.clone());
            }
            current = self.slot(node).parent;
        }
        None
    }

    // -------------------------------------------------------------------------------------
    // Structural mutation
    // -------------------------------------------------------------------------------------

    /// Removes `id` from its parent's child list and clears its parent link.
    ///
    /// Returns the previous parent, or `None` if the node was already detached. This operation is
    /// infallible: it is the primitive that all other structural edits build on.
    ///
    /// Detaching does *not* reclaim the slot: a detached node stays fully usable (it can be
    /// inspected, edited, cloned and re-attached) as long as something keeps it reachable, typically
    /// the handle the caller got back. Once nothing does, the slot is reclaimed by the next
    /// collection.
    pub(crate) fn detach(&mut self, id: NodeId) -> Option<NodeId> {
        let parent = self.slot_mut(id).parent.take()?;
        let children = &mut self.element_mut(parent).children;
        if let Some(position) = children.iter().position(|&child| child == id) {
            children.remove(position);
        }
        Some(parent)
    }

    /// Attaches `child` as the `index`-th child of `parent`.
    ///
    /// `index` is interpreted in the child list *after* `child` has been detached from any previous
    /// parent, so it is always possible to say "move this node to position `i`".
    ///
    /// # Errors
    ///
    /// - [`XmlError::CycleDetected`] if `parent` is `child` or if `child` is an ancestor of `parent`
    ///   (invariant 3).
    /// - [`XmlError::CannotAttachRoot`] if `child` is the document root (invariant 4).
    /// - [`XmlError::NotAnElement`] if `parent` is not an element.
    /// - [`XmlError::IndexOutOfBounds`] if `index` is greater than the resulting child count.
    ///
    /// Nothing is modified when an error is returned.
    pub(crate) fn attach(&mut self, parent: NodeId, index: usize, child: NodeId) -> XmlResult<()> {
        self.check_attachable(parent, child)?;

        let same_parent = self.slot(child).parent == Some(parent);
        let current_len = self.element(parent).children.len();
        let resulting_len = if same_parent {
            current_len - 1
        } else {
            current_len
        };
        if index > resulting_len {
            return Err(XmlError::IndexOutOfBounds {
                index,
                len: resulting_len,
            });
        }

        self.detach(child);
        self.element_mut(parent).children.insert(index, child);
        self.slot_mut(child).parent = Some(parent);
        Ok(())
    }

    /// Attaches `child` as the last child of `parent` (equivalent to `attach` with the resulting
    /// child count as the index, but without the intermediate length query).
    ///
    /// # Errors
    ///
    /// As [`Arena::attach`], except that the index can never be out of bounds.
    pub(crate) fn append(&mut self, parent: NodeId, child: NodeId) -> XmlResult<()> {
        self.check_attachable(parent, child)?;
        self.detach(child);
        let element = self.element_mut(parent);
        element.children.push(child);
        self.slot_mut(child).parent = Some(parent);
        Ok(())
    }

    /// Attaches `child` directly before or after the child `sibling` of `parent`.
    ///
    /// # Errors
    ///
    /// As [`Arena::attach`], plus [`XmlError::NotAChild`] if `sibling` is not a child of `parent`.
    ///
    /// Inserting a node before or after itself is a no-op.
    pub(crate) fn attach_relative(
        &mut self,
        parent: NodeId,
        sibling: NodeId,
        child: NodeId,
        after: bool,
    ) -> XmlResult<()> {
        if sibling == child {
            return Ok(());
        }
        self.check_attachable(parent, child)?;
        if !self.element(parent).children.contains(&sibling) {
            return Err(XmlError::NotAChild(parent, sibling));
        }

        self.detach(child);
        // `sibling != child`, so detaching `child` cannot have removed `sibling`.
        let position = self
            .element(parent)
            .children
            .iter()
            .position(|&candidate| candidate == sibling)
            .expect("validated above");
        let index = if after { position + 1 } else { position };
        self.element_mut(parent).children.insert(index, child);
        self.slot_mut(child).parent = Some(parent);
        Ok(())
    }

    /// Replaces the node `old` by `new` in `old`'s parent, leaving both in the arena (`old` ends up
    /// detached).
    ///
    /// Replacing a node by itself is a **no-op that reports success**, mirroring
    /// [`Arena::attach_relative`], where inserting a node before or after itself is also a no-op. A
    /// no-op is preferable to an error here because "replace X by X" is a well-defined request with
    /// an obvious meaning (the document already satisfies it), and because the node's attachment
    /// state must not change the outcome: an attached node can no more be "replaced by itself" than
    /// a detached one.
    ///
    /// # Errors
    ///
    /// - [`XmlError::NodeHasNoParent`] if `old` is detached (`old != new`), because then there is
    ///   nothing to replace it in.
    /// - [`XmlError::CycleDetected`] if `new` would create a cycle — either because it is an
    ///   ancestor of `old`'s parent, or because it is an ancestor of `old` itself (the latter would
    ///   leave `old` inside `new`'s subtree while claiming it is detached).
    /// - [`XmlError::CannotAttachRoot`] if `new` is the document root.
    ///
    /// [`XmlError::NotAnElement`] cannot occur here: the parent link of a node is only ever set by
    /// [`Arena::attach`], which requires the parent to be an element.
    ///
    /// Nothing is modified when an error is returned.
    pub(crate) fn replace(&mut self, old: NodeId, new: NodeId) -> XmlResult<()> {
        if old == new {
            return Ok(());
        }
        let parent = self
            .slot(old)
            .parent
            .ok_or(XmlError::NodeHasNoParent(old))?;
        self.check_attachable(parent, new)?;
        if self.is_ancestor(new, old) {
            return Err(XmlError::CycleDetected);
        }

        self.detach(new);
        let position = self
            .element(parent)
            .children
            .iter()
            .position(|&candidate| candidate == old)
            .expect("`new != old`, so the position is unchanged");
        self.element_mut(parent).children[position] = new;
        self.slot_mut(new).parent = Some(parent);
        self.slot_mut(old).parent = None;
        Ok(())
    }

    /// Sets the document root, returning the previous root.
    ///
    /// The previous root is left in the arena in a detached state; the caller is responsible for
    /// creating a handle for it before the arena lock is released if it should survive a collection.
    ///
    /// # Errors
    ///
    /// - [`XmlError::NotAnElement`] if `id` is not an element.
    /// - [`XmlError::RootHasParent`] if `id` is currently attached to a parent.
    ///
    /// Nothing is modified when an error is returned.
    pub(crate) fn set_root(&mut self, id: NodeId) -> XmlResult<Option<NodeId>> {
        if !self.is_element(id) {
            return Err(XmlError::NotAnElement(id));
        }
        if self.slot(id).parent.is_some() {
            return Err(XmlError::RootHasParent);
        }
        Ok(self.root.replace(id))
    }

    /// Clears the document root, returning the previous one.
    pub(crate) fn clear_root(&mut self) -> Option<NodeId> {
        self.root.take()
    }

    /// The shared precondition of every "make `child` a child of `parent`" operation.
    ///
    /// See [`Arena::attach`] for the error conditions.
    fn check_attachable(&self, parent: NodeId, child: NodeId) -> XmlResult<()> {
        if parent == child {
            return Err(XmlError::CycleDetected);
        }
        if !self.is_element(parent) {
            return Err(XmlError::NotAnElement(parent));
        }
        if self.root == Some(child) {
            return Err(XmlError::CannotAttachRoot);
        }
        if self.is_ancestor(child, parent) {
            return Err(XmlError::CycleDetected);
        }
        Ok(())
    }

    // -------------------------------------------------------------------------------------
    // Cloning
    // -------------------------------------------------------------------------------------

    /// Creates an owned copy of the subtree rooted at `id`.
    ///
    /// When `with_children` is `false` the copy contains no children (a shallow clone). The copy is
    /// detached and is not the document root. Shared `Arc` payloads (`QualifiedName`, `Namespace`,
    /// `Arc<str>`) are reused as-is, which is what makes cloning cheap.
    ///
    /// The traversal is iterative (see [`Snapshot`]), so a deeply nested document cannot overflow
    /// the stack.
    pub(crate) fn snapshot(&self, id: NodeId, with_children: bool) -> Snapshot {
        let mut snapshot = Snapshot::default();
        snapshot.nodes.push(self.snapshot_payload(id));
        snapshot.children.push(Vec::new());
        let mut work: Vec<(NodeId, usize)> = vec![(id, 0)];
        if with_children {
            while let Some((source, index)) = work.pop() {
                for &child in self.children(source) {
                    let child_index = snapshot.nodes.len();
                    snapshot.nodes.push(self.snapshot_payload(child));
                    snapshot.children.push(Vec::new());
                    snapshot.children[index].push(child_index);
                    work.push((child, child_index));
                }
            }
        }
        snapshot
    }

    /// Copies the payload of one node (without children).
    fn snapshot_payload(&self, id: NodeId) -> SnapshotNode {
        match self.data(id) {
            NodeData::Element(element) => SnapshotNode::Element {
                name: element.name.clone(),
                attributes: element.attributes.clone(),
                namespace_declarations: element.namespace_declarations.clone(),
            },
            NodeData::Text(text) => SnapshotNode::Text(text.clone()),
            NodeData::Comment(comment) => SnapshotNode::Comment(comment.clone()),
            NodeData::CData(cdata) => SnapshotNode::CData(cdata.clone()),
            NodeData::ProcessingInstruction { target, data } => {
                SnapshotNode::ProcessingInstruction(target.clone(), data.clone())
            }
        }
    }

    /// Rebuilds a snapshot as a detached subtree of *this* arena, interning its names and
    /// namespaces into this document.
    ///
    /// This is the second half of a cross-document copy: the snapshot is produced by the source
    /// document (using the source's shared `Arc` payloads) and rebuilt here, so the copy shares no
    /// arena-internal structure with the source while still reusing the immutable `Arc` payloads
    /// until the interner replaces them with this document's canonical values.
    ///
    /// The rebuild is iterative too: nodes are allocated from the last index to the first (which
    /// works because every child has a larger index than its parent), and the parent links are
    /// written in a second pass. The slots are reserved before the first allocation, so the
    /// half-built subtree cannot be reclaimed by a collection in the middle of the loop; the caller
    /// is responsible for turning the returned id into a handle inside the same critical section.
    ///
    /// # Panics
    ///
    /// Panics if the arena cannot hold `snapshot.nodes.len()` more nodes (see
    /// [`Arena::reserve_slots`]).
    pub(crate) fn insert_snapshot(&mut self, snapshot: Snapshot) -> NodeId {
        let count = snapshot.nodes.len();
        self.reserve_slots(count);
        let live_before = self.live;
        let mut ids: Vec<NodeId> = vec![NodeId::from_index(0); count];
        for index in (0..count).rev() {
            ids[index] = match &snapshot.nodes[index] {
                SnapshotNode::Element {
                    name,
                    attributes,
                    namespace_declarations,
                } => self.alloc_element(ElementData {
                    name: name.clone(),
                    attributes: attributes.clone(),
                    namespace_declarations: namespace_declarations.clone(),
                    children: Vec::new(),
                }),
                SnapshotNode::Text(text) => self.alloc(NodeData::Text(text.clone())),
                SnapshotNode::Comment(comment) => self.alloc(NodeData::Comment(comment.clone())),
                SnapshotNode::CData(cdata) => self.alloc(NodeData::CData(cdata.clone())),
                SnapshotNode::ProcessingInstruction(target, data) => {
                    self.alloc(NodeData::ProcessingInstruction {
                        target: target.clone(),
                        data: data.clone(),
                    })
                }
            };
        }
        // The reservation above guaranteed that no allocation in the loop could trigger a
        // collection, so the subtree is complete and neither the capacity nor the live count moved
        // for any other reason than these allocations.
        debug_assert_eq!(
            self.live,
            live_before + count,
            "bulk construction must allocate exactly the reserved number of slots"
        );
        for index in 0..count {
            let children: Vec<NodeId> = snapshot.children[index]
                .iter()
                .map(|&child| ids[child])
                .collect();
            if !children.is_empty() {
                self.element_mut(ids[index]).children = children.clone();
            }
            for child in children {
                self.slot_mut(child).parent = Some(ids[index]);
            }
        }
        ids[snapshot.root]
    }

    /// Creates a copy of the subtree rooted at `id` inside this same arena.
    pub(crate) fn copy_within(&mut self, id: NodeId, with_children: bool) -> NodeId {
        let snapshot = self.snapshot(id, with_children);
        self.insert_snapshot(snapshot)
    }

    /// The XML declaration stored for this document, if any.
    pub(crate) fn declaration(&self) -> Option<XmlDeclaration> {
        self.declaration.clone()
    }

    /// Replaces the stored XML declaration.
    pub(crate) fn set_declaration(&mut self, declaration: Option<XmlDeclaration>) {
        self.declaration = declaration;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml_spec::nc_name;

    /// Creates a detached element with no children.
    fn element(arena: &mut Arena, name: &str) -> NodeId {
        arena.alloc_element(ElementData {
            name: QualifiedName::without_namespace(name).unwrap(),
            attributes: BTreeMap::new(),
            namespace_declarations: BTreeMap::new(),
            children: Vec::new(),
        })
    }

    #[test]
    #[should_panic(expected = "document arena exhausted")]
    fn reserving_more_slots_than_the_arena_can_hold_panics() {
        // The arena can never hold more than `MAX_NODES` slots, so a larger reservation fails. The
        // growth loop performs the doubling before it gives up, but it never allocates memory: the
        // slots themselves are created lazily.
        let mut arena = Arena::with_config(GcConfig::new().with_initial_capacity(1));
        arena.reserve_slots(MAX_NODES + 1);
    }

    #[test]
    fn the_capacity_grows_by_the_configured_factor_and_stops_at_the_limit() {
        let mut arena = Arena::with_config(GcConfig::new().with_initial_capacity(2));
        assert_eq!(arena.capacity(), 2);
        assert!(arena.grow());
        assert_eq!(arena.capacity(), 4);
        assert!(arena.grow());
        assert_eq!(arena.capacity(), 8);

        // A larger factor grows faster.
        let mut arena = Arena::with_config(
            GcConfig::new()
                .with_initial_capacity(3)
                .with_growth_factor(3),
        );
        assert!(arena.grow());
        assert_eq!(arena.capacity(), 9);

        // The capacity never exceeds the id space, so growth eventually reports failure.
        let mut arena = Arena::with_config(GcConfig::new().with_initial_capacity(MAX_NODES));
        assert_eq!(arena.capacity(), MAX_NODES);
        assert!(!arena.grow());
    }

    #[test]
    fn a_reclaimed_slot_is_reused_by_the_next_allocation() {
        let mut arena = Arena::new();
        let node = element(&mut arena, "a");
        let handle = arena.external_ref(node).expect("the slot is occupied");

        // A handle keeps the node through a collection.
        assert_eq!(arena.collect().collected, 0);
        assert_eq!(arena.live(), 1);
        assert!(arena.occupied(node).is_some());

        // Without it, the node is reclaimed and its slot is handed out again.
        drop(handle);
        let stats = arena.collect();
        assert_eq!(stats.collected, 1);
        assert_eq!(stats.live, 0);
        assert!(arena.occupied(node).is_none());
        assert!(arena.external_ref(node).is_none());

        let reused = element(&mut arena, "b");
        assert_eq!(reused, node);
        assert_eq!(arena.live(), 1);
    }

    #[test]
    fn attach_and_detach_maintain_both_directions() {
        let mut arena = Arena::new();
        let parent = element(&mut arena, "p");
        let child = element(&mut arena, "c");
        arena.append(parent, child).unwrap();
        assert_eq!(arena.children(parent), &[child]);
        assert_eq!(arena.parent(child), Some(parent));
        assert!(arena.is_ancestor(parent, child));

        assert_eq!(arena.detach(child), Some(parent));
        assert!(arena.children(parent).is_empty());
        assert_eq!(arena.parent(child), None);
        assert!(!arena.is_ancestor(parent, child));
    }

    #[test]
    fn detaching_a_detached_node_is_a_no_op() {
        let mut arena = Arena::new();
        let node = element(&mut arena, "n");
        assert_eq!(arena.detach(node), None);
        assert_eq!(arena.detach(node), None);
    }

    #[test]
    fn cyclic_attachments_are_rejected() {
        let mut arena = Arena::new();
        let a = element(&mut arena, "a");
        let b = element(&mut arena, "b");
        let c = element(&mut arena, "c");
        arena.append(a, b).unwrap();
        arena.append(b, c).unwrap();

        assert!(matches!(arena.append(c, a), Err(XmlError::CycleDetected)));
        assert!(matches!(arena.append(a, a), Err(XmlError::CycleDetected)));
        // The rejected operations left the arena untouched.
        assert_eq!(arena.children(a), &[b]);
        assert_eq!(arena.children(b), &[c]);
        assert!(arena.children(c).is_empty());
    }

    #[test]
    fn the_index_is_interpreted_after_detaching() {
        let mut arena = Arena::new();
        let parent = element(&mut arena, "p");
        let a = element(&mut arena, "a");
        let b = element(&mut arena, "b");
        let c = element(&mut arena, "c");
        arena.append(parent, a).unwrap();
        arena.append(parent, b).unwrap();
        arena.append(parent, c).unwrap();

        // Move the last child to the front.
        arena.attach(parent, 0, c).unwrap();
        assert_eq!(arena.children(parent), &[c, a, b]);
        // Move the front child to the very end.
        arena.attach(parent, 2, c).unwrap();
        assert_eq!(arena.children(parent), &[a, b, c]);
    }

    #[test]
    fn out_of_range_index_is_an_error_and_changes_nothing() {
        let mut arena = Arena::new();
        let parent = element(&mut arena, "p");
        let child = element(&mut arena, "c");
        let detached = element(&mut arena, "d");
        arena.append(parent, child).unwrap();

        // `detached` is not attached yet, so the resulting child count is 1.
        assert!(matches!(
            arena.attach(parent, 2, detached),
            Err(XmlError::IndexOutOfBounds { index: 2, len: 1 })
        ));
        assert_eq!(arena.children(parent), &[child]);
        assert_eq!(arena.parent(detached), None);
        assert_eq!(arena.parent(child), Some(parent));

        // Moving an existing child interprets the index in the list *after* detaching it, so the
        // resulting count is 0 and only index 0 is in range.
        assert!(matches!(
            arena.attach(parent, 1, child),
            Err(XmlError::IndexOutOfBounds { index: 1, len: 0 })
        ));
        assert_eq!(arena.children(parent), &[child]);
        assert_eq!(arena.parent(child), Some(parent));
    }

    #[test]
    fn relative_insertion() {
        let mut arena = Arena::new();
        let parent = element(&mut arena, "p");
        let a = element(&mut arena, "a");
        let b = element(&mut arena, "b");
        let c = element(&mut arena, "c");
        arena.append(parent, a).unwrap();
        arena.append(parent, c).unwrap();

        arena.attach_relative(parent, c, b, false).unwrap();
        assert_eq!(arena.children(parent), &[a, b, c]);
        arena.attach_relative(parent, a, b, true).unwrap();
        assert_eq!(arena.children(parent), &[a, b, c]);
        arena.attach_relative(parent, b, b, true).unwrap();
        assert_eq!(arena.children(parent), &[a, b, c]);
        assert!(matches!(arena.attach_relative(parent, b, b, true), Ok(())));
    }

    #[test]
    fn replace_keeps_links_consistent() {
        let mut arena = Arena::new();
        let parent = element(&mut arena, "p");
        let old = element(&mut arena, "old");
        let new = element(&mut arena, "new");
        arena.append(parent, old).unwrap();

        arena.replace(old, new).unwrap();
        assert_eq!(arena.children(parent), &[new]);
        assert_eq!(arena.parent(new), Some(parent));
        assert_eq!(arena.parent(old), None);
    }

    #[test]
    fn replace_by_self_is_a_no_op() {
        let mut arena = Arena::new();
        let parent = element(&mut arena, "p");
        let child = element(&mut arena, "c");
        let detached = element(&mut arena, "d");
        arena.append(parent, child).unwrap();

        // Attached node: no-op, and crucially not an error.
        assert!(matches!(arena.replace(child, child), Ok(())));
        assert_eq!(arena.children(parent), &[child]);
        assert_eq!(arena.parent(child), Some(parent));

        // Detached node: also a no-op, and also not an error.
        assert!(matches!(arena.replace(detached, detached), Ok(())));
        assert_eq!(arena.parent(detached), None);
    }

    #[test]
    fn replace_rejects_an_ancestor_of_the_replaced_node() {
        let mut arena = Arena::new();
        let parent = element(&mut arena, "p");
        let old = element(&mut arena, "old");
        let grandchild = element(&mut arena, "grandchild");
        arena.append(parent, old).unwrap();
        arena.append(old, grandchild).unwrap();

        // Replacing `grandchild` by its ancestor `old` would leave `grandchild` inside `old`.
        assert!(matches!(
            arena.replace(grandchild, old),
            Err(XmlError::CycleDetected)
        ));
        assert_eq!(arena.children(old), &[grandchild]);
        assert_eq!(arena.children(parent), &[old]);
    }

    #[test]
    fn root_cannot_be_attached() {
        let mut arena = Arena::new();
        let root = element(&mut arena, "root");
        let other = element(&mut arena, "other");
        arena.set_root(root).unwrap();
        assert!(matches!(
            arena.append(other, root),
            Err(XmlError::CannotAttachRoot)
        ));
        // Getting a new root back is fine as long as it is detached.
        assert!(arena.set_root(other).is_ok());
    }

    #[test]
    fn attached_root_cannot_be_reset_as_root() {
        let mut arena = Arena::new();
        let root = element(&mut arena, "root");
        let child = element(&mut arena, "child");
        arena.append(root, child).unwrap();
        arena.set_root(root).unwrap();
        assert!(matches!(
            arena.set_root(child),
            Err(XmlError::RootHasParent)
        ));
    }

    #[test]
    fn namespaces_in_scope_shadow_outer_declarations() {
        let mut arena = Arena::new();
        let outer = element(&mut arena, "outer");
        let inner = element(&mut arena, "inner");
        arena.append(outer, inner).unwrap();

        let outer_ns = Namespace::prefixed("http://outer", "ex").unwrap();
        arena
            .element_mut(outer)
            .namespace_declarations
            .insert(Some(nc_name("ex")), Some(outer_ns.clone()));
        let inner_ns = Namespace::prefixed("http://inner", "ex").unwrap();
        arena
            .element_mut(inner)
            .namespace_declarations
            .insert(Some(nc_name("ex")), Some(inner_ns.clone()));

        let scope = arena.namespaces_in_scope(inner);
        assert_eq!(scope.len(), 1);
        assert_eq!(scope[0].1, Some(inner_ns.clone()));

        assert_eq!(
            arena.resolve_prefix(inner, Some(&nc_name("ex"))),
            Some(Some(inner_ns))
        );
        assert_eq!(
            arena.resolve_prefix(outer, Some(&nc_name("ex"))),
            Some(Some(outer_ns))
        );
        assert_eq!(arena.resolve_prefix(inner, Some(&nc_name("nope"))), None);
    }

    #[test]
    fn snapshot_round_trip_preserves_the_subtree() {
        let mut arena = Arena::new();
        let parent = element(&mut arena, "p");
        let child = element(&mut arena, "c");
        arena.append(parent, child).unwrap();
        arena.element_mut(child).attributes.insert(
            QualifiedName::without_namespace("a").unwrap(),
            Arc::from("v"),
        );

        let shallow = arena.copy_within(parent, false);
        assert_eq!(arena.children(shallow).len(), 0);
        assert_eq!(arena.parent(shallow), None);

        let deep = arena.copy_within(parent, true);
        assert_eq!(arena.children(deep).len(), 1);
        let copied_child = arena.children(deep)[0];
        assert_ne!(copied_child, child);
        assert_eq!(arena.element(copied_child).attributes.len(), 1);
        assert_eq!(arena.parent(copied_child), Some(deep));
    }
}
