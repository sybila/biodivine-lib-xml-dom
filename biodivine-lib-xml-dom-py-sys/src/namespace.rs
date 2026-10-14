//! [`Namespace`]: an XML namespace, i.e. a URI plus an optional prefix.

use biodivine_lib_xml_dom::Namespace;
use pyo3::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::error::to_py_err;

/// An immutable XML namespace binding: a URI and an optional prefix.
///
/// The Rust rules are enforced here too: the URI must not be empty, a prefix must be a valid
/// `NCName`, and the reserved `xml`/`xmlns` prefixes and the two reserved URIs may only be used in
/// the way the Namespaces specification allows. Violations raise
/// `XmlNamespaceError` (or `XmlSyntaxError` for a bad prefix).
///
/// Equality is structural (URI **and** prefix, as Rust's `PartialEq`); use
/// [`PyNamespace::has_same_uri`] for the namespace-level comparison that ignores the prefix.
#[pyclass(
    name = "Namespace",
    module = "biodivine_sys.xml_dom",
    frozen,
    from_py_object
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyNamespace {
    pub(crate) inner: Namespace,
}

impl From<Namespace> for PyNamespace {
    fn from(inner: Namespace) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyNamespace {
    /// Creates a namespace, with a prefix if one is given.
    ///
    /// Raises:
    ///     XmlNamespaceError: if the URI is empty or is one of the reserved URIs.
    ///     XmlSyntaxError: if the prefix is not a valid XML name.
    #[new]
    #[pyo3(signature = (uri, prefix = None))]
    pub fn new(uri: &str, prefix: Option<&str>) -> PyResult<Self> {
        let namespace = match prefix {
            Some(prefix) => Namespace::prefixed(uri, prefix),
            None => Namespace::without_prefix(uri),
        };
        namespace.map(Self::from).map_err(to_py_err)
    }

    /// Creates a namespace without a prefix (a default namespace).
    ///
    /// Raises:
    ///     XmlNamespaceError: if the URI is empty or is one of the reserved URIs.
    #[staticmethod]
    pub fn without_prefix(uri: &str) -> PyResult<Self> {
        Namespace::without_prefix(uri)
            .map(Self::from)
            .map_err(to_py_err)
    }

    /// Creates a namespace with a prefix.
    ///
    /// Raises:
    ///     XmlSyntaxError: if the prefix is not a valid XML name.
    ///     XmlNamespaceError: if the URI/prefix pair is not a legal namespace.
    #[staticmethod]
    pub fn prefixed(uri: &str, prefix: &str) -> PyResult<Self> {
        Namespace::prefixed(uri, prefix)
            .map(Self::from)
            .map_err(to_py_err)
    }

    /// The namespace URI.
    pub fn uri(&self) -> String {
        self.inner.uri().to_string()
    }

    /// The namespace prefix, or `None` for a default namespace.
    pub fn prefix(&self) -> Option<String> {
        self.inner.prefix_str().map(|prefix| prefix.to_string())
    }

    /// Whether this namespace and `other` have the same URI (the specification's notion of
    /// namespace equality; the prefix is irrelevant).
    pub fn has_same_uri(&self, other: &PyNamespace) -> bool {
        self.inner.has_same_uri(&other.inner)
    }

    /// A hash consistent with `__eq__`.
    pub fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.inner.hash(&mut hasher);
        hasher.finish()
    }

    /// Two namespaces are equal if their URI and their prefix agree.
    pub fn __eq__(&self, other: &Self) -> bool {
        self.inner == other.inner
    }

    /// A debug representation for interactive use, not a serialization of the value.
    pub fn __repr__(&self) -> String {
        match self.inner.prefix_str() {
            Some(prefix) => {
                format!("Namespace({:?}, prefix={:?})", self.inner.uri(), prefix)
            }
            None => format!("Namespace({:?})", self.inner.uri()),
        }
    }

    /// The namespace as `prefix:uri`, or just the URI for a default namespace.
    pub fn __str__(&self) -> String {
        match self.inner.prefix_str() {
            Some(prefix) => format!("{prefix}:{}", self.inner.uri()),
            None => self.inner.uri().to_string(),
        }
    }
}
