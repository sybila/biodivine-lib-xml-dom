//! [`NodeId`]: the arena index of one node.
//!
//! Ids are opaque in Rust, and they stay opaque here: the only things Python can do with one are
//! compare it, hash it, use it as an integer index (`__index__`) for display purposes, and look the
//! node up again with `Document.node(id)`. That mirrors Rust exactly.

use biodivine_lib_xml_dom::NodeId;
use pyo3::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

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
    /// The zero-based index of the node in its document's arena.
    pub fn index(&self) -> usize {
        self.inner.index()
    }

    pub fn __index__(&self) -> usize {
        self.inner.index()
    }

    pub fn __int__(&self) -> usize {
        self.inner.index()
    }

    pub fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.inner.hash(&mut hasher);
        hasher.finish()
    }

    pub fn __eq__(&self, other: &Self) -> bool {
        self.inner == other.inner
    }

    pub fn __repr__(&self) -> String {
        format!("NodeId({})", self.inner.index())
    }

    pub fn __str__(&self) -> String {
        self.inner.to_string()
    }
}
