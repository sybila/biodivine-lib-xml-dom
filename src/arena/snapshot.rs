//! Detached, fully owned copies of subtrees, used to move data between documents.

use std::collections::BTreeMap;

use crate::namespace::Namespace;
use crate::qualified_name::QualifiedName;
use crate::xml_spec::{CData, Comment, NCName, PiData, PiTarget, Text};

/// A detached, fully owned copy of a subtree.
///
/// The representation is deliberately *flat*: nodes are stored in a `Vec` (in an order where a
/// child always has a larger index than its parent) and the parent/child relation is a list of
/// indices rather than nested values. Two things follow, and both matter for the same reason:
///
/// * taking and rebuilding a snapshot is iterative, so it cannot overflow the stack on a deeply
///   nested document — a stack overflow aborts the process and could therefore not be reported as
///   a typed error;
/// * dropping a snapshot is a flat `Vec` drop rather than a recursive one.
///
/// Snapshots are also what makes cross-document copies deadlock-free: the source subtree is
/// snapshotted under a read lock of the source document, that lock is released, and only then is
/// the snapshot inserted under a write lock of the target document.
#[derive(Debug, Clone, Default)]
pub(crate) struct Snapshot {
    /// The node payloads. See [`SnapshotNode`].
    pub(crate) nodes: Vec<SnapshotNode>,
    /// `children[i]` lists the indices in `nodes` that are children of `nodes[i]`, in document
    /// order. Only elements can have children, so the entries of non-elements are empty.
    pub(crate) children: Vec<Vec<usize>>,
    /// The index of the copied subtree's root.
    pub(crate) root: usize,
}

/// The payload of one node of a [`Snapshot`].
#[derive(Debug, Clone)]
pub(crate) enum SnapshotNode {
    /// An element, without its children (they live in [`Snapshot::children`]).
    Element {
        name: QualifiedName,
        attributes: BTreeMap<QualifiedName, Text>,
        namespace_declarations: BTreeMap<Option<NCName>, Option<Namespace>>,
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
