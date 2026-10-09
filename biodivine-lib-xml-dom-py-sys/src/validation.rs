//! Validation results: the node kind enum, the validation error list and the validation exception.

use biodivine_lib_xml_dom::{
    NodeKind, ValidationErrorKind, XmlValidationError, XmlValidationErrors,
};
use pyo3::prelude::*;

use crate::error::XmlValidationError as PyValidationException;
use crate::node_id::PyNodeId;

/// The kind of a node, without its payload.
///
/// This is the cheap discriminant of [`PyNode`](crate::PyNode); the payload is available through the
/// typed accessors (`as_element`, `text`, `comment`, `cdata`, `processing_instruction`).
#[pyclass(
    name = "NodeKind",
    module = "biodivine_lib_xml_dom._sys",
    frozen,
    eq,
    eq_int,
    from_py_object
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PyNodeKind {
    /// An element node.
    Element = 0,
    /// A text node.
    Text = 1,
    /// A comment node.
    Comment = 2,
    /// A CDATA section.
    CData = 3,
    /// A processing instruction.
    ProcessingInstruction = 4,
}

impl From<NodeKind> for PyNodeKind {
    fn from(kind: NodeKind) -> Self {
        match kind {
            NodeKind::Element => Self::Element,
            NodeKind::Text => Self::Text,
            NodeKind::Comment => Self::Comment,
            NodeKind::CData => Self::CData,
            NodeKind::ProcessingInstruction => Self::ProcessingInstruction,
        }
    }
}

/// One problem found by whole-document validation.
#[pyclass(
    name = "ValidationError",
    module = "biodivine_lib_xml_dom._sys",
    frozen,
    from_py_object
)]
#[derive(Debug, Clone)]
pub struct PyXmlValidationError {
    pub(crate) inner: XmlValidationError,
}

#[pymethods]
impl PyXmlValidationError {
    /// A stable name for the kind of problem, e.g. `"undeclared_prefix"`.
    pub fn kind(&self) -> String {
        kind_name(self.inner.kind()).to_string()
    }

    /// The rule file in `specification/rules/` this problem comes from.
    pub fn rule(&self) -> String {
        self.inner.kind().rule().to_string()
    }

    /// The node the problem is attached to, or `None` for a problem about the document as a whole
    /// (a missing root element).
    pub fn node(&self) -> Option<PyNodeId> {
        self.inner.node().map(PyNodeId::from)
    }

    /// A human-readable description.
    pub fn message(&self) -> String {
        self.inner.message().to_string()
    }

    /// The value as a string: a node as XML, a name as `prefix:local`, a namespace as
    /// `prefix:uri` (or just the URI), a number as its digits.
    pub fn __str__(&self) -> String {
        self.inner.to_string()
    }

    /// A debug representation for interactive use, not a serialization of the value.
    pub fn __repr__(&self) -> String {
        format!(
            "ValidationError(kind={:?}, node={:?}, message={:?})",
            self.kind(),
            self.inner.node().map(|node| node.index()),
            self.message()
        )
    }

    /// Equality with another handle of the same type: the same node/document/value
    /// (Rust's `PartialEq`). Python object identity is *not* part of it.
    pub fn __eq__(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl From<XmlValidationError> for PyXmlValidationError {
    fn from(inner: XmlValidationError) -> Self {
        Self { inner }
    }
}

/// Every problem found by one validation run.
///
/// This is what the `XmlValidationError` exception carries in `args[0]`: it is a sequence of
/// [`PyXmlValidationError`], so Python can iterate it directly.
#[pyclass(
    name = "ValidationErrors",
    module = "biodivine_lib_xml_dom._sys",
    frozen,
    from_py_object
)]
#[derive(Debug, Clone)]
pub struct PyXmlValidationErrors {
    pub(crate) inner: XmlValidationErrors,
}

#[pymethods]
impl PyXmlValidationErrors {
    /// How many problems were found.
    pub fn __len__(&self) -> usize {
        self.inner.len()
    }

    /// The problem at `index`.
    /// Raises:
    ///     IndexError: if there is no problem at `index`.
    pub fn __getitem__(&self, index: usize) -> PyResult<PyXmlValidationError> {
        self.inner
            .as_slice()
            .get(index)
            .cloned()
            .map(PyXmlValidationError::from)
            .ok_or_else(|| pyo3::exceptions::PyIndexError::new_err("no such validation error"))
    }

    /// The problems, as a list.
    pub fn errors(&self) -> Vec<PyXmlValidationError> {
        self.inner
            .iter()
            .cloned()
            .map(PyXmlValidationError::from)
            .collect()
    }

    /// The value as a string: a node as XML, a name as `prefix:local`, a namespace as
    /// `prefix:uri` (or just the URI), a number as its digits.
    pub fn __str__(&self) -> String {
        self.inner.to_string()
    }

    /// A debug representation for interactive use, not a serialization of the value.
    pub fn __repr__(&self) -> String {
        format!("ValidationErrors({} problem(s))", self.inner.len())
    }
}

impl From<XmlValidationErrors> for PyXmlValidationErrors {
    fn from(inner: XmlValidationErrors) -> Self {
        Self { inner }
    }
}

/// A stable snake_case name for a validation error kind, for use in Python code.
fn kind_name(kind: &ValidationErrorKind) -> &'static str {
    match kind {
        ValidationErrorKind::MissingRoot => "missing_root",
        ValidationErrorKind::RootIsNotAnElement => "root_is_not_an_element",
        ValidationErrorKind::RootHasParent => "root_has_parent",
        ValidationErrorKind::ParentChildMismatch => "parent_child_mismatch",
        ValidationErrorKind::CyclicStructure => "cyclic_structure",
        ValidationErrorKind::UndeclaredPrefix { .. } => "undeclared_prefix",
        ValidationErrorKind::PrefixBoundToDifferentUri { .. } => "prefix_bound_to_different_uri",
        ValidationErrorKind::MissingDefaultNamespace { .. } => "missing_default_namespace",
        ValidationErrorKind::DefaultNamespaceMismatch { .. } => "default_namespace_mismatch",
        ValidationErrorKind::UnprefixedNameTakesDefaultNamespace { .. } => {
            "unprefixed_name_takes_default_namespace"
        }
        ValidationErrorKind::AttributeNamespaceWithoutPrefix { .. } => {
            "attribute_namespace_without_prefix"
        }
        ValidationErrorKind::ReservedPrefix { .. } => "reserved_prefix",
        ValidationErrorKind::IllegalNamespaceDeclaration { .. } => "illegal_namespace_declaration",
        ValidationErrorKind::XmlIdIsNotAName { .. } => "xml_id_is_not_a_name",
        ValidationErrorKind::DuplicateXmlId { .. } => "duplicate_xml_id",
        ValidationErrorKind::InvalidXmlLang { .. } => "invalid_xml_lang",
        ValidationErrorKind::InvalidXmlSpace { .. } => "invalid_xml_space",
    }
}

/// Raises [`PyValidationException`] carrying the issues.
///
/// Used by `Document.validate`, which is the only place where a validation failure surfaces.
pub(crate) fn validation_failure(errors: XmlValidationErrors) -> PyErr {
    let message = errors.to_string();
    let payload = PyXmlValidationErrors::from(errors);
    PyValidationException::new_err((message, payload))
}

/// Adds the classes to the module.
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyXmlValidationError>()?;
    module.add_class::<PyXmlValidationErrors>()?;
    Ok(())
}
