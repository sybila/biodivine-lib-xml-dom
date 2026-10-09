//! The node arena of a single [`crate::Document`].
//!
//! Requirement (1) replaces the previous design — one `RwLock` per element, with `Arc` handles
//! both upwards and downwards — by an *arena*: all node payloads of a document live in one
//! `Vec<NodeSlot>`, and everything that refers to a node does so through an index ([`NodeId`]).
//!
//! Two properties follow directly, and both were the reason for the rewrite (REVIEW D3 and D4):
//!
//! * there is no `Arc` cycle, because parent/child links are plain indices, so dropping a
//!   document frees all of its nodes;
//! * every structural invariant is maintained by this module under a single `&mut self` borrow,
//!   so a half-linked state is unrepresentable outside of it and no check-then-act sequence can
//!   be interleaved by another thread.
//!
//! # Invariants maintained by [`Arena`]
//!
//! 1. `slot.parent == Some(p)` if and only if `slot` occurs exactly once in the child list of the
//!    element `p`.
//! 2. `root` is an element whose `parent` is `None`.
//! 3. No node is reachable from itself (the tree is acyclic).
//! 4. `root` never occurs in anybody's child list.
//!
//! All mutating methods validate *everything they can fail on* first and only then perform
//! infallible index writes. A failed operation therefore never changes the arena — this is the
//! operational meaning of the panic-safety argument in `docs/design/PLAN.md` §3.2.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::error::{XmlError, XmlResult};
use crate::interner::Interner;
use crate::namespace::Namespace;
use crate::qualified_name::QualifiedName;
use crate::xml_spec::{CData, Comment, NCName, PiData, PiTarget, Text};

/// The maximum number of nodes a single document can hold.
///
/// [`NodeId`] is a `u32` index, so this is `u32::MAX`. Reaching this limit requires allocating
/// more than four billion nodes in one document (hundreds of gigabytes of node payloads); it is
/// nevertheless checked explicitly rather than wrapping silently: allocating past the
/// limit panics instead of wrapping the index.
pub const MAX_NODES: usize = u32::MAX as usize;

/// An opaque index of a node inside the arena of a [`crate::Document`].
///
/// `NodeId` values are only ever produced by the document itself. They are stable for the whole
/// lifetime of the document: the arena never reuses a slot, so an id can never start referring to
/// a different node (this design deliberately trades memory — one slot per node ever created —
/// for the absence of dangling or aliasing handles).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(u32);

impl NodeId {
    /// The zero-based index of this node in its document's arena.
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The payload of a node.
#[derive(Debug)]
pub(crate) enum NodeData {
    /// An element: a name, attributes, namespace declarations and children.
    Element(ElementData),
    /// A text node. The payload is validated at construction
    /// (`rule.well-formedness.legal-characters`).
    Text(Text),
    /// A comment node (`rule.well-formedness.comment-no-double-hyphen`).
    Comment(Comment),
    /// A CDATA section (`rule.document-structure.cdata-section-must-not-contain-cdend`).
    CData(CData),
    /// A processing instruction.
    ProcessingInstruction {
        /// The PI target (`rule.well-formedness.pi-target-is-name`).
        target: PiTarget,
        /// The PI content (`rule.well-formedness.pi-no-contains-close`).
        data: PiData,
    },
}

/// The payload of an element node.
#[derive(Debug)]
pub(crate) struct ElementData {
    /// The element's expanded name (local name plus optional namespace).
    pub(crate) name: QualifiedName,
    /// Attributes, keyed by expanded name.
    ///
    /// A map keyed by [`QualifiedName`] (whose equality compares the local name and the namespace
    /// *URI*) makes two attributes with the same expanded name unrepresentable — requirement
    /// (4)(1), and `rule.elements-and-tags.unique-attribute-specification` at the storage level.
    /// Setting an attribute that is already present therefore *overwrites* the old value; that is
    /// the documented default behaviour required by the task description.
    pub(crate) attributes: BTreeMap<QualifiedName, Arc<str>>,
    /// Namespace declarations written on *this* element.
    ///
    /// The key is the prefix (`None` for the default namespace) and the value is the binding.
    /// `Some(None)` is an empty declaration (`xmlns=""`), which removes the default namespace
    /// from scope (`rule.namespace-usage.empty-default-namespace`).
    pub(crate) namespace_declarations: BTreeMap<Option<NCName>, Option<Namespace>>,
    /// The children, in document order.
    pub(crate) children: Vec<NodeId>,
}

/// One slot of the arena: the payload plus the link to the parent.
#[derive(Debug)]
pub(crate) struct NodeSlot {
    /// The parent node, or `None` if the node is detached or is the document root.
    pub(crate) parent: Option<NodeId>,
    /// The node payload.
    pub(crate) data: NodeData,
}

/// A detached, fully owned copy of a subtree.
///
/// Used by the clone operations: a copy is always created through this intermediate
/// representation, which lets [`Arena::insert_snapshot`] rebuild it with the target document's
/// interner. For same-document clones the snapshot never leaves the arena lock.
///
/// It is also what makes cross-document copies deadlock-free: the source subtree is snapshotted
/// under a read lock of the source document, that lock is released, and only then is the snapshot
/// inserted under a write lock of the target document. Two document locks are therefore never
/// held at the same time (see `docs/design/PLAN.md` §3.1).
#[derive(Debug, Clone)]
pub(crate) enum Snapshot {
    /// An element with its children (possibly empty for a shallow clone).
    Element {
        name: QualifiedName,
        attributes: BTreeMap<QualifiedName, Arc<str>>,
        namespace_declarations: BTreeMap<Option<NCName>, Option<Namespace>>,
        children: Vec<Snapshot>,
    },
    /// A text node.
    Text(Text),
    /// A comment node.
    Comment(Comment),
    /// A CDATA section.
    CData(CData),
    /// A processing instruction.
    ProcessingInstruction(PiTarget, PiData),
}

/// Checks that one more node can be allocated.
///
/// `NodeId` is a `u32` index, so the number of slots is bounded by [`MAX_NODES`]; the bound is
/// checked explicitly (rather than letting the index wrap) even though reaching it requires more
/// than four billion live nodes in a single document.
///
/// # Panics
///
/// Panics if `used` has already reached [`MAX_NODES`].
fn ensure_capacity(used: usize) {
    assert!(
        used < MAX_NODES,
        "document arena exhausted: a document can hold at most {MAX_NODES} nodes"
    );
}

/// All nodes of one document, plus its root and its name interner.
#[derive(Debug, Default)]
pub(crate) struct Arena {
    nodes: Vec<NodeSlot>,
    root: Option<NodeId>,
    interner: Interner,
}

impl Arena {
    /// Creates an empty arena.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    // -------------------------------------------------------------------------------------
    // Allocation
    // -------------------------------------------------------------------------------------

    /// Stores `data` and returns its (freshly assigned, never reused) id.
    ///
    /// The new node is detached: it has no parent and is not the document root.
    ///
    /// # Panics
    ///
    /// Panics if the arena already holds [`MAX_NODES`] nodes. This is unreachable in practice
    /// (it requires more than four billion live nodes in one document) and is checked explicitly
    /// so that the `u32` index never wraps silently.
    pub(crate) fn alloc(&mut self, data: NodeData) -> NodeId {
        ensure_capacity(self.nodes.len());
        let id = NodeId(self.nodes.len() as u32);
        self.nodes.push(NodeSlot { parent: None, data });
        id
    }

    /// Stores an element payload, interning its name, attributes and namespace declarations.
    pub(crate) fn alloc_element(&mut self, element: ElementData) -> NodeId {
        let element = self.intern_element(element);
        self.alloc(NodeData::Element(element))
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
    /// Attribute *values* are `Arc<str>` and are intentionally not deduplicated by value: they
    /// are frequently unique, so a per-document map of all values would be pure overhead. They are
    /// still shared whenever a node is cloned.
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

    /// The number of slots used. Slots of detached nodes are counted too.
    pub(crate) fn len(&self) -> usize {
        self.nodes.len()
    }

    /// The document root, if any.
    pub(crate) fn root(&self) -> Option<NodeId> {
        self.root
    }

    /// All node ids, in allocation order (attached and detached alike).
    pub(crate) fn ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        (0..self.nodes.len()).map(|i| NodeId(i as u32))
    }

    /// The slot of `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` does not belong to this arena. Every id used internally is produced by this
    /// arena, and [`crate::Document::node`] bounds-checks ids that come from the outside.
    pub(crate) fn slot(&self, id: NodeId) -> &NodeSlot {
        self.nodes
            .get(id.index())
            .expect("node id does not belong to this document")
    }

    /// The mutable slot of `id`. See [`Arena::slot`] for the panic conditions.
    fn slot_mut(&mut self, id: NodeId) -> &mut NodeSlot {
        self.nodes
            .get_mut(id.index())
            .expect("node id does not belong to this document")
    }

    /// The payload of `id`. See [`Arena::slot`] for the panic conditions.
    pub(crate) fn data(&self, id: NodeId) -> &NodeData {
        &self.slot(id).data
    }

    /// The element payload of `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is not an element. Call sites either know the kind statically or check it
    /// first (e.g. via [`Arena::is_element`]).
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

    /// Whether `id` refers to an element.
    pub(crate) fn is_element(&self, id: NodeId) -> bool {
        matches!(self.data(id), NodeData::Element(_))
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
    /// acyclic, so the bound is never reached; it exists so that even a hypothetical bug cannot
    /// turn this into an infinite loop.
    pub(crate) fn is_ancestor(&self, ancestor: NodeId, node: NodeId) -> bool {
        let mut current = self.slot(node).parent;
        for _ in 0..self.nodes.len() {
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
        for _ in 0..self.nodes.len() {
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
    /// Iterative on purpose: the previous implementation recursed through `Arc` handles, which
    /// limited the usable document depth by the stack size.
    pub(crate) fn resolve_prefix(
        &self,
        id: NodeId,
        prefix: Option<&NCName>,
    ) -> Option<Option<Namespace>> {
        let mut current = Some(id);
        for _ in 0..self.nodes.len() {
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
    /// Note that the slot itself is *not* reclaimed: `NodeId` values stay valid forever, so a
    /// detached node remains fully usable (it can be inspected, edited, cloned and re-attached).
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
    /// `index` is interpreted in the child list *after* `child` has been detached from any
    /// previous parent, so it is always possible to say "move this node to position `i`".
    ///
    /// # Errors
    ///
    /// - [`XmlError::CycleDetected`] if `parent` is `child` or if `child` is an ancestor of
    ///   `parent` (invariant 3).
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

    /// Replaces the node `old` by `new` in `old`'s parent, leaving both in the arena (`old` ends
    /// up detached). Returns the parent that `old` was removed from.
    ///
    /// # Errors
    ///
    /// - [`XmlError::NodeHasNoParent`] if `old` is detached.
    /// - [`XmlError::CycleDetected`] if `new` would create a cycle — either because it is an
    ///   ancestor of `old`'s parent, or because it is an ancestor of `old` itself (the latter
    ///   would leave `old` inside `new`'s subtree while claiming it is detached).
    /// - [`XmlError::CannotAttachRoot`], [`XmlError::NotAnElement`] as [`Arena::attach`].
    ///
    /// Replacing a node by itself is a no-op. Nothing is modified when an error is returned.
    pub(crate) fn replace(&mut self, old: NodeId, new: NodeId) -> XmlResult<NodeId> {
        if old == new {
            return Err(XmlError::NodeHasNoParent(old));
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
        Ok(parent)
    }

    /// Sets the document root, returning the previous root.
    ///
    /// The previous root is left in the arena in a detached state.
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
    /// When `with_children` is `false` the copy contains no children (a shallow clone). The copy
    /// is detached and is not the document root. Shared `Arc` payloads (`QualifiedName`,
    /// `Namespace`, `Arc<str>`) are reused as-is, which is what makes cloning cheap.
    pub(crate) fn snapshot(&self, id: NodeId, with_children: bool) -> Snapshot {
        match self.data(id) {
            NodeData::Element(element) => {
                let children = if with_children {
                    element
                        .children
                        .iter()
                        .map(|&child| self.snapshot(child, true))
                        .collect()
                } else {
                    Vec::new()
                };
                Snapshot::Element {
                    name: element.name.clone(),
                    attributes: element.attributes.clone(),
                    namespace_declarations: element.namespace_declarations.clone(),
                    children,
                }
            }
            NodeData::Text(text) => Snapshot::Text(text.clone()),
            NodeData::Comment(comment) => Snapshot::Comment(comment.clone()),
            NodeData::CData(cdata) => Snapshot::CData(cdata.clone()),
            NodeData::ProcessingInstruction { target, data } => {
                Snapshot::ProcessingInstruction(target.clone(), data.clone())
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
    pub(crate) fn insert_snapshot(&mut self, snapshot: Snapshot) -> NodeId {
        match snapshot {
            Snapshot::Element {
                name,
                attributes,
                namespace_declarations,
                children,
            } => {
                let children = children
                    .into_iter()
                    .map(|child| self.insert_snapshot(child))
                    .collect::<Vec<_>>();
                let id = self.alloc_element(ElementData {
                    name,
                    attributes,
                    namespace_declarations,
                    children: children.clone(),
                });
                for child in children {
                    self.slot_mut(child).parent = Some(id);
                }
                id
            }
            Snapshot::Text(text) => self.alloc(NodeData::Text(text)),
            Snapshot::Comment(comment) => self.alloc(NodeData::Comment(comment)),
            Snapshot::CData(cdata) => self.alloc(NodeData::CData(cdata)),
            Snapshot::ProcessingInstruction(target, data) => {
                self.alloc(NodeData::ProcessingInstruction { target, data })
            }
        }
    }

    /// Creates a copy of the subtree rooted at `id` inside this same arena.
    pub(crate) fn copy_within(&mut self, id: NodeId, with_children: bool) -> NodeId {
        let snapshot = self.snapshot(id, with_children);
        self.insert_snapshot(snapshot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml_spec::nc_name;

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
    fn exceeding_the_arena_capacity_panics_instead_of_wrapping() {
        // The guard is checked directly, because allocating `u32::MAX` nodes is not feasible.
        ensure_capacity(MAX_NODES);
    }

    #[test]
    fn the_capacity_guard_allows_everything_below_the_limit() {
        ensure_capacity(0);
        ensure_capacity(MAX_NODES - 1);
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

        // Moving an existing child interprets the index in the list *after* detaching it, so
        // the resulting count is 0 and only index 0 is in range.
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

        assert_eq!(arena.replace(old, new).unwrap(), parent);
        assert_eq!(arena.children(parent), &[new]);
        assert_eq!(arena.parent(new), Some(parent));
        assert_eq!(arena.parent(old), None);
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
