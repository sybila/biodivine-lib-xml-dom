//! [`PyDocument`]: the handle to one XML document.
//!
//! The doc comments on `#[pymethods]` become Python docstrings, so they are written for Python
//! readers: plain text plus a `Raises:` section, without rustdoc links.

use biodivine_lib_xml_dom::Document;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use crate::element::PyElement;
use crate::error::to_py_err;
use crate::gc::PyGcStats;
use crate::interrupt::run_interruptible;
use crate::node::PyNode;
use crate::node_id::PyNodeId;
use crate::qualified_name::PyQualifiedName;
use crate::validation::PyValidationError;
use crate::write_options::{PyWriteOptions, PyXmlDeclaration};

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
        Document::new().into()
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
    /// Raises:
    ///     XmlDocumentError: if `root` belongs to another document or is already attached to a
    ///         parent.
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
    /// Raises:
    ///     XmlSyntaxError: if the target is not a valid XML name or matches `xml`
    ///         case-insensitively, or if the content contains `?>`.
    pub fn create_processing_instruction(
        &self,
        target: &str,
        data: &str,
    ) -> PyResult<PyNode> {
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

    /// Checks the whole document and reports every problem it can find.
    ///
    /// The GIL is released for the duration of the walk.
    ///
    /// Raises:
    ///     XmlValidationError: if the document has any problem. `args[0]` is the multi-line
    ///         summary and `args[1]` the list of `ValidationError` objects.
    ///     BaseException: whatever a Python signal handler raises to interrupt the operation.
    pub fn validate(&self, py: Python<'_>) -> PyResult<()> {
        let document = self.inner.clone();
        run_interruptible(py, document.node_count(), move || document.validate())
    }

    /// Every problem found by whole-document validation, as a list (empty when the document is
    /// valid).
    ///
    /// Raises:
    ///     BaseException: whatever a Python signal handler raises to interrupt the operation (for
    ///         example `KeyboardInterrupt`).
    pub fn validation_errors(
        &self,
        py: Python<'_>,
    ) -> PyResult<Vec<PyValidationError>> {
        let document = self.inner.clone();
        let errors = run_interruptible(py, document.node_count(), move || {
            document.validation_errors()
        })?;
        Ok(errors.into_iter().map(PyValidationError::from).collect())
    }

    /// Whether the document passes `validate`.
    ///
    /// Raises:
    ///     BaseException: whatever a Python signal handler raises to interrupt the operation (for
    ///         example `KeyboardInterrupt`).
    pub fn is_valid(&self, py: Python<'_>) -> PyResult<bool> {
        let document = self.inner.clone();
        run_interruptible(py, document.node_count(), move || document.is_valid())
    }

    /// The number of nodes this document currently holds.
    ///
    /// Attached and detached nodes are counted alike; nodes that the garbage collector has
    /// reclaimed are not.
    pub fn node_count(&self) -> usize {
        self.inner.node_count()
    }

    /// All nodes this document currently holds, in slot order.
    ///
    /// The returned handles keep their nodes alive for as long as the list exists.
    pub fn nodes(&self) -> Vec<PyNode> {
        self.inner.nodes().into_iter().map(PyNode::from).collect()
    }

    /// Looks up a node by id, or `None` if the id does not refer to a live node of this document.
    pub fn node(&self, id: &PyNodeId) -> Option<PyNode> {
        self.inner.node(id.inner).map(PyNode::from)
    }

    /// Reclaims every node that is unreachable from a handle or from the document root, and
    /// returns what the collection did.
    ///
    /// The collector runs automatically when the arena is full, so calling this is only needed to
    /// bound memory earlier, or to observe what is still alive.
    ///
    /// Raises:
    ///     BaseException: whatever a Python signal handler raises to interrupt the operation (for
    ///         example `KeyboardInterrupt`).
    pub fn collect_garbage(&self, py: Python<'_>) -> PyResult<PyGcStats> {
        let document = self.inner.clone();
        run_interruptible(py, document.node_count(), move || {
            document.collect_garbage()
        })
        .map(PyGcStats::from)
    }

    /// The number of nodes this document may hold before it considers its arena full.
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Whether two handles refer to the same document.
    pub fn ptr_eq(&self, other: &PyDocument) -> bool {
        self.inner.ptr_eq(&other.inner)
    }

    /// A hash consistent with `__eq__`: handles of one document hash equally.
    pub fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.inner.hash(&mut hasher);
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

    /// Parses an XML document from a string.
    ///
    /// Raises:
    ///     XmlSyntaxError: if the input is not well-formed.
    ///     XmlNamespaceError: if a namespace is used illegally.
    #[staticmethod]
    #[pyo3(signature = (source))]
    pub fn parse_string(py: Python<'_>, source: &str) -> PyResult<PyDocument> {
        crate::io::parse_string(py, source)
    }

    /// Parses an XML document from bytes (which must be valid UTF-8).
    ///
    /// Raises:
    ///     XmlSyntaxError: if the input is not well-formed or is not valid UTF-8.
    ///     XmlNamespaceError: if a namespace is used illegally.
    #[staticmethod]
    #[pyo3(signature = (data))]
    pub fn parse_bytes(
        py: Python<'_>,
        data: &Bound<'_, PyBytes>,
    ) -> PyResult<PyDocument> {
        crate::io::parse_bytes(py, data)
    }

    /// Parses an XML document from a file.
    ///
    /// Raises:
    ///     XmlIoError: if the file cannot be read.
    ///     XmlSyntaxError: if the input is not well-formed or is not valid UTF-8.
    ///     XmlNamespaceError: if a namespace is used illegally.
    #[staticmethod]
    #[pyo3(signature = (path))]
    pub fn parse_file(py: Python<'_>, path: PathBuf) -> PyResult<PyDocument> {
        crate::io::parse_file(py, path)
    }

    /// Serializes this document into a string, with the given options or the default ones.
    ///
    /// A document without a root element and without a declaration serializes to the empty string.
    ///
    /// Raises:
    ///     BaseException: whatever a Python signal handler raises to interrupt the operation (for
    ///         example `KeyboardInterrupt`).
    #[pyo3(signature = (options = None))]
    pub fn write_string(
        &self,
        py: Python<'_>,
        options: Option<PyWriteOptions>,
    ) -> PyResult<String> {
        crate::io::write_string(py, self, options)
    }

    /// Writes this document to a file, with the given options or the default ones.
    ///
    /// Raises:
    ///     XmlIoError: if the file cannot be created or written.
    #[pyo3(signature = (path, options = None))]
    pub fn write_file(
        &self,
        py: Python<'_>,
        path: PathBuf,
        options: Option<PyWriteOptions>,
    ) -> PyResult<()> {
        crate::io::write_file(py, self, path, options)
    }
}
