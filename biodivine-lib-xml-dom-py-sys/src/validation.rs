//! Validation results: the node kind enum, one validation error, and the validation exception.

use biodivine_lib_xml_dom::{
    NodeKind, ValidationError, ValidationErrorKind, ValidationErrors,
};
use pyo3::prelude::*;
use pyo3::types::PyList;

use crate::error::XmlValidationError as PyValidationException;
use crate::node_id::{PyNodeId, raw_index};

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
pub struct PyValidationError {
    pub(crate) inner: ValidationError,
}

#[pymethods]
impl PyValidationError {
    /// A stable name for the kind of problem, e.g. `"undeclared_prefix"`.
    pub fn kind(&self) -> String {
        kind_name(self.inner.kind()).to_string()
    }

    /// The identifier of the specification rule this problem comes from.
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

    /// The problem as one line of text: node, message and rule.
    pub fn __str__(&self) -> String {
        self.inner.to_string()
    }

    /// A debug representation for interactive use, not a serialization of the value.
    pub fn __repr__(&self) -> String {
        format!(
            "ValidationError(kind={:?}, node={:?}, message={:?})",
            self.kind(),
            self.inner.node().map(raw_index),
            self.message()
        )
    }

    /// Two problems are equal if their kind, node and message agree.
    pub fn __eq__(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl From<ValidationError> for PyValidationError {
    fn from(inner: ValidationError) -> Self {
        Self { inner }
    }
}

/// A stable snake_case name for a validation error kind, for use in Python code.
fn kind_name(kind: &ValidationErrorKind) -> &'static str {
    match kind {
        ValidationErrorKind::MissingRoot => "missing_root",
        ValidationErrorKind::UndeclaredPrefix { .. } => "undeclared_prefix",
        ValidationErrorKind::PrefixBoundToDifferentUri { .. } => {
            "prefix_bound_to_different_uri"
        }
        ValidationErrorKind::MissingDefaultNamespace { .. } => {
            "missing_default_namespace"
        }
        ValidationErrorKind::DefaultNamespaceMismatch { .. } => {
            "default_namespace_mismatch"
        }
        ValidationErrorKind::UnprefixedNameTakesDefaultNamespace { .. } => {
            "unprefixed_name_takes_default_namespace"
        }
        ValidationErrorKind::AttributeNamespaceWithoutPrefix { .. } => {
            "attribute_namespace_without_prefix"
        }
        ValidationErrorKind::ReservedPrefix { .. } => "reserved_prefix",
        ValidationErrorKind::XmlIdIsNotAName { .. } => "xml_id_is_not_a_name",
        ValidationErrorKind::DuplicateXmlId { .. } => "duplicate_xml_id",
        ValidationErrorKind::InvalidXmlLang { .. } => "invalid_xml_lang",
        ValidationErrorKind::InvalidXmlSpace { .. } => "invalid_xml_space",
        // `ValidationErrorKind` is non-exhaustive; the bindings are updated with each new kind.
        _ => "other",
    }
}

/// Raises [`PyValidationException`] carrying the problems.
///
/// The exception's `args` are the multi-line summary and a `list` of [`PyValidationError`].
/// Used by `Document.validate`, which is the only place where a validation failure surfaces.
pub(crate) fn validation_failure(errors: ValidationErrors) -> PyErr {
    let message = errors.to_string();
    Python::attach(|py| {
        let problems = errors.into_iter().map(PyValidationError::from);
        match PyList::new(py, problems) {
            Ok(list) => PyValidationException::new_err((message, list.unbind())),
            Err(error) => error,
        }
    })
}

/// Adds the classes to the module.
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyValidationError>()?;
    Ok(())
}
