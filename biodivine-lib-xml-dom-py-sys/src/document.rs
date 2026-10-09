//! [`PyDocument`]: the handle to one XML document.

use biodivine_lib_xml_dom::Document;
use pyo3::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::element::PyElement;
use crate::error::to_py_err;
use crate::node::PyNode;
use crate::node_id::PyNodeId;
use crate::qualified_name::PyQualifiedName;
use crate::validation::validation_failure;
use crate::write_options::PyXmlDeclaration;

/// An XML document.
///
/// A document is a cheap, thread-safe handle to shared state: cloning it or passing it to another
/// thread does not copy the tree. All node data lives behind a single reader-writer lock, and every
/// method acquires and releases it. Because there is exactly one lock and it is never re-acquired
/// while held, no operation can deadlock - and that property holds from Python as well.
#[pyclass(
    name = "Document",
    module = "biodivine_lib_xml_dom._sys",
    from_py_object
)]
#[derive(Debug, Clone)]
pub struct PyDocument {
    pub(crate) inner: Document,
}

impl From<Document> for PyDocument {
    fn from(inner: Document) -> Self {
        Self { inner }
    }
}

impl Default for PyDocument {
    /// An empty document, the same as [`PyDocument::new`].
    fn default() -> Self {
        Self::new()
    }
}

#[pymethods]
impl PyDocument {
    /// Creates a new, empty document (no root element).
    #[new]
    pub fn new() -> Self {
        Document::empty().into()
    }

    /// The root element, or `None` if the document has none.
    pub fn root(&self) -> Option<PyElement> {
        self.inner.root().map(PyElement::from)
    }

    /// Sets the root element and returns the previous root (if any).
    ///
    /// The previous root is not deleted: it stays in the arena as a detached subtree, so existing
    /// handles to it remain valid.
    ///
    /// Raises `XmlDocumentError` if `root` belongs to another document or is already attached to a
    /// parent. (Rust has a panicking `set_root` and a `set_root_checked`; Python has no panics, so
    /// this is the `_checked` behaviour under the plain name.)
    /// Raises:
    ///     XmlDocumentError: if `root` belongs to another document or is already attached to a parent.
    #[pyo3(signature = (root))]
    pub fn set_root(&self, root: &PyElement) -> PyResult<Option<PyElement>> {
        self.inner
            .set_root_checked(root.inner.clone())
            .map(|previous| previous.map(PyElement::from))
            .map_err(to_py_err)
    }

    /// Removes the root element and returns it.
    ///
    /// The subtree is preserved in the arena; the document simply stops pointing at it.
    pub fn clear_root(&self) -> Option<PyElement> {
        self.inner.clear_root().map(PyElement::from)
    }

    /// Creates a new, detached element with the given expanded name.
    pub fn create_element(&self, name: &PyQualifiedName) -> PyElement {
        PyElement::from(self.inner.create_element(name.inner.clone()))
    }

    /// Creates a new, detached text node.
    ///
    /// Raises `XmlSyntaxError` if the content contains characters that are not legal in XML, so an
    /// invalid text node cannot be created at all.
    /// Raises:
    ///     XmlSyntaxError: if the content contains characters that are not legal in XML.
    pub fn create_text(&self, text: &str) -> PyResult<PyNode> {
        self.inner
            .create_text(text)
            .map(PyNode::from)
            .map_err(to_py_err)
    }

    /// Creates a new, detached comment node.
    ///
    /// Raises `XmlSyntaxError` if the content contains `--` or ends with `-`.
    /// Raises:
    ///     XmlSyntaxError: if the content contains `--` or ends with `-`.
    pub fn create_comment(&self, text: &str) -> PyResult<PyNode> {
        self.inner
            .create_comment(text)
            .map(PyNode::from)
            .map_err(to_py_err)
    }

    /// Creates a new, detached CDATA section.
    ///
    /// Raises `XmlSyntaxError` if the content contains `]]>`.
    /// Raises:
    ///     XmlSyntaxError: if the content contains `]]>`.
    pub fn create_cdata(&self, text: &str) -> PyResult<PyNode> {
        self.inner
            .create_cdata(text)
            .map(PyNode::from)
            .map_err(to_py_err)
    }

    /// Creates a new, detached processing instruction.
    ///
    /// Raises `XmlSyntaxError` if the target is not a valid XML `Name` or matches `xml`
    /// case-insensitively, or if the content contains `?>`.
    /// Raises:
    ///     XmlSyntaxError: if the target is not a valid XML name or matches `xml` case-insensitively,
    ///     or if the content contains `?>`.
    pub fn create_processing_instruction(&self, target: &str, data: &str) -> PyResult<PyNode> {
        self.inner
            .create_processing_instruction(target, data)
            .map(PyNode::from)
            .map_err(to_py_err)
    }

    /// The XML declaration of the document, if it has one.
    pub fn xml_declaration(&self) -> Option<PyXmlDeclaration> {
        self.inner.xml_declaration().map(PyXmlDeclaration::from)
    }

    /// Replaces the XML declaration of the document.
    #[pyo3(signature = (declaration))]
    pub fn set_xml_declaration(&self, declaration: Option<PyXmlDeclaration>) {
        self.inner
            .set_xml_declaration(declaration.map(|declaration| declaration.inner));
    }

    /// Checks the whole document and reports **every** problem it can find.
    ///
    /// Raises `XmlValidationError` whose `args[0]` is a `ValidationErrors` sequence; the message is
    /// the same multi-line summary Rust produces. Use [`PyDocument::validation_errors`] to get the
    /// list without catching an exception.
    ///
    /// The GIL is released for the duration of the walk.
    /// Raises:
    ///     XmlValidationError: if the document has any problem. `args[0]` is the multi-line summary
    ///     and `args[1]` the `ValidationErrors` sequence.
    pub fn validate(&self, py: Python<'_>) -> PyResult<()> {
        let document = self.inner.clone();
        py.detach(|| document.validate())
            .map_err(validation_failure)
    }

    /// Every problem found by whole-document validation, as a list (empty when the document is
    /// valid).
    ///
    /// This is the non-raising counterpart of [`PyDocument::validate`].
    pub fn validation_errors(
        &self,
        py: Python<'_>,
    ) -> Vec<crate::validation::PyXmlValidationError> {
        let document = self.inner.clone();
        match py.detach(|| document.validate()) {
            Ok(()) => Vec::new(),
            Err(errors) => errors
                .into_errors()
                .into_iter()
                .map(crate::validation::PyXmlValidationError::from)
                .collect(),
        }
    }

    /// Whether the document passes [`PyDocument::validate`].
    pub fn is_valid(&self, py: Python<'_>) -> bool {
        let document = self.inner.clone();
        py.detach(|| document.is_valid())
    }

    /// The number of arena slots this document uses (attached and detached nodes alike).
    pub fn node_count(&self) -> usize {
        self.inner.node_count()
    }

    /// All nodes of this document, attached and detached alike, in creation order.
    pub fn nodes(&self) -> Vec<PyNode> {
        self.inner.nodes().into_iter().map(PyNode::from).collect()
    }

    /// Looks up a node by id, or `None` if the id does not belong to this document.
    pub fn node(&self, id: &PyNodeId) -> Option<PyNode> {
        self.inner.node(id.inner).map(PyNode::from)
    }

    /// Whether two handles refer to the same document.
    pub fn ptr_eq(&self, other: &PyDocument) -> bool {
        self.inner.ptr_eq(&other.inner)
    }

    /// A hash consistent with `__eq__` (the same value Rust's `Hash` produces, so
    /// equal handles hash equally and can be used as dictionary keys).
    pub fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        hash_document(&self.inner, &mut hasher);
        hasher.finish()
    }

    /// Two documents are equal if and only if they refer to the same underlying document.
    pub fn __eq__(&self, other: &Self) -> bool {
        self.inner == other.inner
    }

    /// A debug representation for interactive use, not a serialization of the value.
    pub fn __repr__(&self) -> String {
        format!("Document(nodes={})", self.inner.node_count())
    }
}

/// Hashes the document by identity, exactly as Rust's `Hash for Document` does.
fn hash_document<H: Hasher>(document: &Document, state: &mut H) {
    document.hash(state);
}
