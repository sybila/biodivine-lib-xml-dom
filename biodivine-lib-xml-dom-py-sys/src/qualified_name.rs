//! [`QualifiedName`]: an expanded name, i.e. a local name plus an optional namespace.

use biodivine_lib_xml_dom::QualifiedName;
use pyo3::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::error::to_py_err;
use crate::namespace::PyNamespace;

/// An immutable XML expanded name: a local name (a valid `NCName`) and an optional namespace.
///
/// The prefix lives in the namespace, exactly as in Rust. Equality and hashing compare the local
/// name and the namespace *URI* (so two names that differ only in the prefix are equal), which is
/// what makes the attribute maps in the core work.
#[pyclass(
    name = "QualifiedName",
    module = "biodivine_lib_xml_dom._sys",
    frozen,
    from_py_object
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyQualifiedName {
    pub(crate) inner: QualifiedName,
}

impl From<QualifiedName> for PyQualifiedName {
    fn from(inner: QualifiedName) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyQualifiedName {
    /// Creates an expanded name from a local name and an optional namespace.
    ///
    /// Raises `XmlSyntaxError` if the local name is not a valid `NCName`.
    /// Raises:
    ///     XmlSyntaxError: if the local name is not a valid NCName.
    #[new]
    #[pyo3(signature = (local_name, namespace = None))]
    pub fn new(local_name: &str, namespace: Option<PyNamespace>) -> PyResult<Self> {
        match namespace {
            Some(namespace) => QualifiedName::with_namespace(local_name, &namespace.inner),
            None => QualifiedName::without_namespace(local_name),
        }
        .map(Self::from)
        .map_err(to_py_err)
    }

    /// Creates an expanded name with no namespace.
    /// Raises:
    ///     XmlSyntaxError: if the local name is not a valid NCName.
    #[staticmethod]
    pub fn without_namespace(local_name: &str) -> PyResult<Self> {
        QualifiedName::without_namespace(local_name)
            .map(Self::from)
            .map_err(to_py_err)
    }

    /// Creates an expanded name in `namespace`.
    /// Raises:
    ///     XmlSyntaxError: if the local name is not a valid NCName.
    #[staticmethod]
    pub fn with_namespace(local_name: &str, namespace: &PyNamespace) -> PyResult<Self> {
        QualifiedName::with_namespace(local_name, &namespace.inner)
            .map(Self::from)
            .map_err(to_py_err)
    }

    /// The local name.
    pub fn local_name(&self) -> String {
        self.inner.local_name_str().to_string()
    }

    /// The namespace, or `None` if the name is in no namespace.
    pub fn namespace(&self) -> Option<PyNamespace> {
        self.inner.namespace().cloned().map(PyNamespace::from)
    }

    /// Resolves an element name written as a string (`"prefix:local"` or `"local"`) against the
    /// declarations that are in scope for `element` (the default namespace applies to elements).
    ///
    /// Raises `XmlNamespaceError` for an undeclared prefix or a reserved one, and
    /// `XmlSyntaxError` for a malformed name.
    /// Raises:
    ///     XmlSyntaxError: if the name is malformed.
    ///     XmlNamespaceError: if the prefix is undeclared or reserved.
    #[staticmethod]
    pub fn resolve_element(element: &crate::element::PyElement, name: &str) -> PyResult<Self> {
        QualifiedName::resolve_element(&element.inner, name)
            .map(Self::from)
            .map_err(to_py_err)
    }

    /// Resolves an attribute name written as a string against the declarations in scope for
    /// `element`. The default namespace never applies to attributes.
    /// Raises:
    ///     XmlSyntaxError: if the name is malformed.
    ///     XmlNamespaceError: if the prefix is undeclared or reserved.
    #[staticmethod]
    pub fn resolve_attribute(element: &crate::element::PyElement, name: &str) -> PyResult<Self> {
        QualifiedName::resolve_attribute(&element.inner, name)
            .map(Self::from)
            .map_err(to_py_err)
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
        format!("QualifiedName({:?})", self.inner.to_string())
    }

    /// The name as it would be written in a document (`prefix:local`, or just `local`).
    pub fn __str__(&self) -> String {
        self.inner.to_string()
    }
}
