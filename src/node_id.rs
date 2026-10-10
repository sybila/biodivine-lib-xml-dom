//! Identifiers of arena slots.

/// The maximum number of nodes a single document can hold at one time.
///
/// A [`NodeId`] is a `u32` index into the arena and the arena never compacts, so this is also the
/// largest capacity the arena can grow to. Reaching the limit requires allocating more than four
/// billion nodes in one document (hundreds of gigabytes of node payloads); it is nevertheless
/// checked explicitly rather than wrapping silently: allocating past the limit panics instead of
/// producing an aliased index.
pub const MAX_NODES: usize = u32::MAX as usize;

/// An opaque index of a node inside the arena of a [`Document`](crate::Document).
///
/// An id on its own is *not* a usable reference: it can only be turned into a working handle by
/// looking it up with [`Document::node`](crate::Document::node), which returns `None` once the slot
/// has been reclaimed by a collection.
///
/// # Validity
///
/// The arena never compacts, so an id always refers to the same arena slot. A slot, however, can be
/// reclaimed once the node is unreachable from every externally referenced node and the document
/// root (see [`Document::collect_garbage`](crate::Document::collect_garbage)) and then handed out to
/// a later allocation, exactly like a file descriptor or a process id. An id is therefore only
/// meaningful while the node it was created for is alive, and a *stale* id silently refers to
/// whatever node comes next — which is why the library never accepts an id where it needs a node.
///
/// A [`Node`](crate::Node) or [`Element`](crate::Element) handle, in contrast, keeps its node alive
/// for as long as the handle exists.
///
/// # Examples
///
/// ```rust
/// use biodivine_lib_xml_dom::{Document, NodeId, QualifiedName};
///
/// let document = Document::empty();
/// let node = document.create_text("value").unwrap();
/// let id = node.id();
///
/// // The id round-trips through the arena ...
/// assert!(document.node(id).unwrap().ptr_eq(&node));
/// // ... and is just a number otherwise.
/// assert_eq!(u32::from(id), 0);
/// assert_eq!(NodeId::from(0_u32), id);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(u32);

impl NodeId {
    /// The zero-based index of this node's arena slot.
    ///
    /// The index is stable for as long as the slot exists. It says nothing about whether the slot
    /// currently holds a node; use [`Document::node`](crate::Document::node) for that.
    pub(crate) fn index(self) -> usize {
        self.0 as usize
    }

    /// Converts the index of an arena slot into an id.
    ///
    /// # Panics
    ///
    /// Panics if `index` does not fit into the id space, which cannot happen because the arena
    /// capacity is bounded by [`MAX_NODES`].
    pub(crate) fn from_index(index: usize) -> Self {
        Self(u32::try_from(index).expect("arena slot indices are bounded by MAX_NODES"))
    }
}

impl From<u32> for NodeId {
    /// Reinterprets a raw integer as an id.
    ///
    /// The value is not checked against any arena here: an id that was never produced by a document
    /// simply does not resolve, and [`Document::node`](crate::Document::node) rejects it.
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<NodeId> for u32 {
    /// The raw integer behind an id; the inverse of [`From<u32>`](NodeId#impl-From<u32>-for-NodeId).
    fn from(id: NodeId) -> Self {
        id.0
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
