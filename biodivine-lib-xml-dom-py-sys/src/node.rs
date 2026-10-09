//! [`PyNode`]: a handle to any node of a document.

use biodivine_lib_xml_dom::Node;
use pyo3::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::document::PyDocument;
use crate::element::PyElement;
use crate::error::to_py_err;
use crate::node_id::PyNodeId;
use crate::validation::PyNodeKind;

/// Either a [`PyNode`] or a [`PyElement`].
///
/// Rust methods that take `impl Into<Node>` (appending, inserting, replacing, ...) accept either,
/// so Python callers do not have to spell `.node()`.
#[derive(Debug, FromPyObject)]
pub enum PyNodeArg {
    /// A generic node handle.
    Node(PyNode),
    /// An element handle.
    Element(PyElement),
}

impl From<PyNodeArg> for Node {
    fn from(value: PyNodeArg) -> Self {
        match value {
            PyNodeArg::Node(node) => node.inner,
            PyNodeArg::Element(element) => element.inner.node(),
        }
    }
}

/// A handle to one node of a [`PyDocument`]: an element, text, comment, CDATA or processing
/// instruction.
///
/// The handle is cheap to copy, stays valid even if the node is detached, and can be passed between
/// threads. Cloning the handle refers to the same node; use `deep_clone`/`shallow_clone` to
/// duplicate the node itself.
#[pyclass(name = "Node", module = "biodivine_lib_xml_dom._sys", from_py_object)]
#[derive(Debug, Clone)]
pub struct PyNode {
    pub(crate) inner: Node,
}

impl From<Node> for PyNode {
    fn from(inner: Node) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyNode {
    /// The document this node belongs to.
    pub fn document(&self) -> PyDocument {
        PyDocument::from(self.inner.document())
    }

    /// The arena id of this node. Ids are only meaningful together with their document.
    pub fn id(&self) -> PyNodeId {
        PyNodeId::from(self.inner.id())
    }

    /// The kind of this node.
    pub fn kind(&self) -> PyNodeKind {
        self.inner.kind().into()
    }

    /// This node as an element, or `None` if it has a different kind.
    pub fn as_element(&self) -> Option<PyElement> {
        self.inner.as_element().map(PyElement::from)
    }

    /// Whether the node is reachable from the document root.
    pub fn is_attached(&self) -> bool {
        self.inner.is_attached()
    }

    /// The parent of this node, or `None` if it is detached or is the document root.
    pub fn parent(&self) -> Option<PyNode> {
        self.inner.parent().map(PyNode::from)
    }

    /// Whether this node is a strict ancestor of `other`.
    pub fn is_ancestor(&self, other: &PyNode) -> bool {
        self.inner.is_ancestor(&other.inner)
    }

    /// The children of this node in document order (empty for non-elements).
    pub fn children(&self) -> Vec<PyNode> {
        self.inner
            .children()
            .into_iter()
            .map(PyNode::from)
            .collect()
    }

    /// The element children of this node in document order.
    pub fn child_elements(&self) -> Vec<PyElement> {
        self.inner
            .child_elements()
            .into_iter()
            .map(PyElement::from)
            .collect()
    }

    /// The first child of this node, if any.
    pub fn first_child(&self) -> Option<PyNode> {
        self.inner.first_child().map(PyNode::from)
    }

    /// The last child of this node, if any.
    pub fn last_child(&self) -> Option<PyNode> {
        self.inner.last_child().map(PyNode::from)
    }

    /// The next sibling of this node, if any.
    pub fn next_sibling(&self) -> Option<PyNode> {
        self.inner.next_sibling().map(PyNode::from)
    }

    /// The previous sibling of this node, if any.
    pub fn previous_sibling(&self) -> Option<PyNode> {
        self.inner.previous_sibling().map(PyNode::from)
    }

    /// The index of this node in its parent's child list, if it has a parent.
    pub fn index_in_parent(&self) -> Option<usize> {
        self.inner.index_in_parent()
    }

    /// All descendants of this node in document order (depth first, iterative).
    pub fn descendants(&self) -> Vec<PyNode> {
        self.inner
            .descendants()
            .into_iter()
            .map(PyNode::from)
            .collect()
    }

    /// The content of this node as text, or `None` if it is not a text node.
    pub fn text(&self) -> Option<String> {
        self.inner.text().map(|text| text.as_str().to_string())
    }

    /// The content of this node as a comment, or `None`.
    pub fn comment(&self) -> Option<String> {
        self.inner
            .comment()
            .map(|comment| comment.as_str().to_string())
    }

    /// The content of this node as CDATA, or `None`.
    pub fn cdata(&self) -> Option<String> {
        self.inner.cdata().map(|cdata| cdata.as_str().to_string())
    }

    /// The `(target, data)` of this node as a processing instruction, or `None`.
    pub fn processing_instruction(&self) -> Option<(String, String)> {
        self.inner
            .processing_instruction()
            .map(|(target, data)| (target.as_str().to_string(), data.as_str().to_string()))
    }

    /// Detaches this node from its parent and returns the previous parent.
    ///
    /// The node stays in the arena in a fully usable state; detaching a detached node does nothing.
    pub fn detach(&self) -> Option<PyNode> {
        self.inner.detach().map(PyNode::from)
    }

    /// Detaches this node and returns the node itself, so it can be re-attached in one expression.
    pub fn remove(&self) -> PyNode {
        PyNode::from(self.inner.remove())
    }

    /// Appends `child` as the last child of this node, moving it if it is already attached.
    ///
    /// Raises `XmlDocumentError` if `child` belongs to another document (`foreign_document`), if it
    /// is this node or an ancestor of it (`cycle_detected`), or if it is the document root
    /// (`cannot_attach_root`). No namespace declaration is added, removed or rewritten: run
    /// `Document.validate` to find the resulting inconsistencies.
    pub fn append_child(&self, child: PyNodeArg) -> PyResult<()> {
        self.inner
            .append_child_checked(Node::from(child))
            .map_err(to_py_err)
    }

    /// Inserts `child` as the `index`-th child of this node.
    ///
    /// `index` is interpreted after `child` has been detached from any previous parent. Raises
    /// `XmlDocumentError` including `index_out_of_bounds`-style failures.
    pub fn insert_child(&self, index: usize, child: PyNodeArg) -> PyResult<()> {
        self.inner
            .insert_child_checked(index, Node::from(child))
            .map_err(to_py_err)
    }

    /// Inserts `child` directly before the child `sibling` of this node.
    pub fn insert_before(&self, sibling: PyNodeArg, child: PyNodeArg) -> PyResult<()> {
        self.inner
            .insert_before_checked(Node::from(sibling), Node::from(child))
            .map_err(to_py_err)
    }

    /// Inserts `child` directly after the child `sibling` of this node.
    pub fn insert_after(&self, sibling: PyNodeArg, child: PyNodeArg) -> PyResult<()> {
        self.inner
            .insert_after_checked(Node::from(sibling), Node::from(child))
            .map_err(to_py_err)
    }

    /// Replaces this node with `replacement` in this node's parent, returning this node (now
    /// detached).
    pub fn replace_with(&self, replacement: PyNodeArg) -> PyResult<PyNode> {
        self.inner
            .replace_with_checked(Node::from(replacement))
            .map(|()| PyNode::from(self.inner.clone()))
            .map_err(to_py_err)
    }

    /// Creates a detached, shallow copy of this node in the same document (no children).
    pub fn shallow_clone(&self) -> PyNode {
        PyNode::from(self.inner.shallow_clone())
    }

    /// Creates a detached, deep copy of this subtree in the same document.
    pub fn deep_clone(&self) -> PyNode {
        PyNode::from(self.inner.deep_clone())
    }

    /// Creates a detached, shallow copy of this node in `target`.
    pub fn shallow_clone_into(&self, target: &PyDocument) -> PyNode {
        PyNode::from(self.inner.shallow_clone_into(&target.inner))
    }

    /// Creates a detached, deep copy of this subtree in `target`.
    ///
    /// This is the sanctioned way to move a tree between documents. The copy is a point-in-time
    /// snapshot: the source lock is taken for the snapshot only and released before the target is
    /// modified, so the two documents are never locked at the same time. The GIL is released here.
    pub fn deep_clone_into(&self, py: Python<'_>, target: &PyDocument) -> PyNode {
        let node = self.inner.clone();
        let target = target.inner.clone();
        PyNode::from(py.detach(move || node.deep_clone_into(&target)))
    }

    /// Whether two handles refer to the same node of the same document.
    pub fn ptr_eq(&self, other: &PyNode) -> bool {
        self.inner.ptr_eq(&other.inner)
    }

    /// Whether this node belongs to `document`.
    pub fn belongs_to(&self, document: &PyDocument) -> bool {
        self.inner.belongs_to(&document.inner)
    }

    /// The node as it would appear in a document (elements are serialized in full).
    pub fn __str__(&self) -> String {
        self.inner.to_string()
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
        let kind = match self.inner.kind() {
            biodivine_lib_xml_dom::NodeKind::Element => "Element".to_string(),
            other => format!("{other:?}"),
        };
        format!(
            "Node({}, id={}, attached={})",
            kind,
            self.inner.id().index(),
            self.inner.is_attached()
        )
    }
}
