//! [`NodeId`]: the arena index of one node.
//!
//! An id is opaque in the sense that it is useless on its own: the only things Python can do with
//! one are compare it, hash it, use it as an integer (`__index__`, for display) and look the node up
//! again with `Document.node(id)`. An id can also be built from an integer, which mirrors Rust's
//! `NodeId::from(u32)`; it is then resolved - or rejected - by the document it is used with, and it
//! stops resolving once the node it referred to has been reclaimed by the collector.
//!
//! That mirrors Rust exactly.

use biodivine_lib_xml_dom::NodeId;
use pyo3::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// The raw integer behind an id, for display and for `__index__`.
///
/// The value is an arena slot index, which is stable while the slot exists, but which says nothing
/// about whether the slot still holds the same node. It is a debugging and interop convenience;
/// resolving an id to a node always goes through `Document.node`.
pub(crate) fn raw_index(id: NodeId) -> usize {
    usize::try_from(u32::from(id)).expect("a node id fits into a platform index")
}

/// The index of a node in its document's arena.
///
/// An id is only meaningful together with the document that produced it; use
/// [`PyDocument::node`](crate::PyDocument::node) to get the node back, and
/// [`PyNode::ptr_eq`](crate::PyNode::ptr_eq) to compare nodes from possibly different documents.
#[pyclass(
    name = "NodeId",
    module = "biodivine_lib_xml_dom._sys",
    frozen,
    from_py_object
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PyNodeId {
    pub(crate) inner: NodeId,
}

impl From<NodeId> for PyNodeId {
    fn from(inner: NodeId) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyNodeId {
    /// Creates an id from a raw integer, exactly like Rust's `NodeId::from(u32)`.
    ///
    /// The value is not checked against any document: it resolves to a node only if the document it
    /// is used with holds a node in that slot, so `Document.node` may well return `None` for it.
    #[new]
    pub fn new(index: u32) -> Self {
        Self {
            inner: NodeId::from(index),
        }
    }

    /// The zero-based index of the node in its document's arena.
    pub fn index(&self) -> usize {
        raw_index(self.inner)
    }

    /// The value as an integer index, so a `NodeId` can be used in `range()`, list
    /// indexing and slicing.
    pub fn __index__(&self) -> usize {
        raw_index(self.inner)
    }

    /// The value as an `int` (the same as `__index__`).
    pub fn __int__(&self) -> usize {
        raw_index(self.inner)
    }

    /// A hash consistent with `__eq__` (the same value Rust's `Hash` produces, so
    /// equal handles hash equally and can be used as dictionary keys).
    pub fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.inner.hash(&mut hasher);
        hasher.finish()
    }

    /// Equality with another handle of the same type: the same node/document/value
    /// (Rust's `PartialEq`). Python object identity is *not* part of it.
    pub fn __eq__(&self, other: &Self) -> bool {
        self.inner == other.inner
    }

    /// A debug representation for interactive use, not a serialization of the value.
    pub fn __repr__(&self) -> String {
        format!("NodeId({})", raw_index(self.inner))
    }

    /// The value as a string: a node as XML, a name as `prefix:local`, a namespace as
    /// `prefix:uri` (or just the URI), a number as its digits.
    pub fn __str__(&self) -> String {
        self.inner.to_string()
    }
}
