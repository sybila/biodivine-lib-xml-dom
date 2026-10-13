//! [`PyElement`]: a handle to an element node.
//!
//! Rust's `Element` derefs to `Node`, so an element has both APIs. Python has no equivalent of
//! `Deref`, so this class exposes the element-specific API plus [`PyElement::node`] for the tree
//! API. The pure-Python `Element` (in `python/biodivine_lib_xml_dom/element.py`) inherits from the
//! pure-Python `Node` and therefore does offer both in one object; see
//! [`documentation::bindings`](biodivine_lib_xml_dom::documentation::bindings).

use biodivine_lib_xml_dom::xml_spec::NCName;
use biodivine_lib_xml_dom::{Element, Node};
use pyo3::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::document::PyDocument;
use crate::error::to_py_err;
use crate::interrupt::run_interruptible;
use crate::namespace::PyNamespace;
use crate::node::{PyNode, PyNodeArg};
use crate::node_id::{PyNodeId, raw_index};
use crate::qualified_name::PyQualifiedName;

/// A handle to an element node of a document.
#[pyclass(
    name = "Element",
    module = "biodivine_lib_xml_dom._sys",
    from_py_object
)]
#[derive(Debug, Clone)]
pub struct PyElement {
    pub(crate) inner: Element,
}

impl From<Element> for PyElement {
    fn from(inner: Element) -> Self {
        Self { inner }
    }
}

/// Parses an optional prefix given as a Python string.
fn optional_prefix(prefix: Option<&str>) -> PyResult<Option<NCName>> {
    prefix
        .map(|prefix| NCName::try_from(prefix).map_err(to_py_err))
        .transpose()
}

#[pymethods]
impl PyElement {
    /// This element as a generic `Node` handle, which carries the tree API (`children`,
    /// `append_child`, `detach`, the clone family, ...).
    pub fn node(&self) -> PyNode {
        PyNode::from(self.inner.node())
    }

    /// The document this element belongs to.
    pub fn document(&self) -> PyDocument {
        PyDocument::from(self.inner.document())
    }

    /// The arena id of this element.
    pub fn id(&self) -> PyNodeId {
        PyNodeId::from(self.inner.id())
    }

    /// The expanded name of this element.
    pub fn qualified_name(&self) -> PyQualifiedName {
        PyQualifiedName::from(self.inner.qualified_name())
    }

    /// The local name of this element.
    pub fn local_name(&self) -> String {
        self.inner.local_name().as_str().to_string()
    }

    /// The namespace of this element, or `None`.
    pub fn namespace(&self) -> Option<PyNamespace> {
        self.inner.namespace().map(PyNamespace::from)
    }

    /// Changes the name of this element. No namespace declaration is added or removed.
    pub fn set_qualified_name(&self, name: &PyQualifiedName) {
        self.inner.set_qualified_name(name.inner.clone());
    }

    /// All attributes of this element, as `(QualifiedName, value)` pairs.
    pub fn attributes(&self) -> Vec<(PyQualifiedName, String)> {
        self.inner
            .attributes()
            .into_iter()
            .map(|(name, value)| {
                (PyQualifiedName::from(name), value.as_str().to_string())
            })
            .collect()
    }

    /// The value of the attribute with the given expanded name, or `None`.
    pub fn attribute(&self, name: &PyQualifiedName) -> Option<String> {
        self.inner
            .attribute(&name.inner)
            .map(|value| value.as_str().to_string())
    }

    /// The value of the attribute with the given local name that is in no namespace, or `None`.
    ///
    /// Raises:
    ///     XmlSyntaxError: if `local_name` is not a valid XML name.
    pub fn attribute_without_namespace(
        &self,
        local_name: &str,
    ) -> PyResult<Option<String>> {
        let local_name = NCName::try_from(local_name).map_err(to_py_err)?;
        Ok(self
            .inner
            .attribute_without_namespace(&local_name)
            .map(|value| value.as_str().to_string()))
    }

    /// Whether this element has an attribute with the given expanded name.
    pub fn has_attribute(&self, name: &PyQualifiedName) -> bool {
        self.inner.has_attribute(&name.inner)
    }

    /// Sets an attribute, replacing any previous attribute with the same expanded name.
    ///
    /// Raises:
    ///     XmlSyntaxError: if the value contains characters that are not legal in XML.
    pub fn set_attribute(&self, name: &PyQualifiedName, value: &str) -> PyResult<()> {
        self.inner
            .set_attribute_checked(name.inner.clone(), value)
            .map_err(to_py_err)
    }

    /// Removes an attribute, returning its previous value (or `None`).
    pub fn remove_attribute(&self, name: &PyQualifiedName) -> Option<String> {
        self.inner
            .remove_attribute(&name.inner)
            .map(|value| value.as_str().to_string())
    }

    /// Removes all attributes of this element.
    pub fn clear_attributes(&self) {
        self.inner.clear_attributes();
    }

    /// The namespace declarations written on *this* element, without inheritance.
    ///
    /// Each entry is `(prefix_or_None, namespace_or_None)`; a namespace of `None` is an empty
    /// declaration (`xmlns=""`).
    pub fn namespace_declarations(&self) -> Vec<(Option<String>, Option<PyNamespace>)> {
        self.inner
            .namespace_declarations()
            .into_iter()
            .map(|(prefix, namespace)| {
                (
                    prefix.map(|prefix| prefix.as_str().to_string()),
                    namespace.map(PyNamespace::from),
                )
            })
            .collect()
    }

    /// All namespace bindings visible to this element, innermost first.
    pub fn namespaces_in_scope(&self) -> Vec<(Option<String>, Option<PyNamespace>)> {
        self.inner
            .namespaces_in_scope()
            .into_iter()
            .map(|(prefix, namespace)| {
                (
                    prefix.map(|prefix| prefix.as_str().to_string()),
                    namespace.map(PyNamespace::from),
                )
            })
            .collect()
    }

    /// Declares a namespace on this element, overwriting any previous binding for that prefix.
    pub fn declare_namespace(&self, namespace: &PyNamespace) {
        self.inner.declare_namespace(namespace.inner.clone());
    }

    /// Declares a namespace, refusing to *change* an existing binding for the same prefix.
    ///
    /// Raises:
    ///     XmlNamespaceError: if the prefix is already bound to a different URI.
    pub fn declare_namespace_checked(&self, namespace: &PyNamespace) -> PyResult<()> {
        self.inner
            .declare_namespace_checked(namespace.inner.clone())
            .map_err(to_py_err)
    }

    /// Declares `xmlns=""` on this element, removing the default namespace from its scope.
    pub fn undeclare_default_namespace(&self) {
        self.inner.undeclare_default_namespace();
    }

    /// Removes a namespace declaration from this element.
    ///
    /// `prefix` is `None` for the default namespace. Returns `(True, namespace)` if a declaration
    /// was removed (with `namespace` `None` for an empty declaration), and `(False, None)` if
    /// this element had no such declaration.
    ///
    /// Raises:
    ///     XmlSyntaxError: if the prefix is not a valid XML name.
    #[pyo3(signature = (prefix = None))]
    pub fn remove_namespace_declaration(
        &self,
        prefix: Option<&str>,
    ) -> PyResult<(bool, Option<PyNamespace>)> {
        let prefix = optional_prefix(prefix)?;
        Ok(
            match self.inner.remove_namespace_declaration(prefix.as_ref()) {
                Some(previous) => (true, previous.map(PyNamespace::from)),
                None => (false, None),
            },
        )
    }

    /// The namespace bound to `prefix` in the scope of this element, or `None`.
    ///
    /// `prefix` is `None` for the default namespace. The predefined `xml` prefix is *not* resolved
    /// here (it needs no declaration); use `resolve_element_name` or `resolve_attribute_name` for
    /// that.
    ///
    /// Raises:
    ///     XmlSyntaxError: if the prefix is not a valid XML name.
    #[pyo3(signature = (prefix = None))]
    pub fn lookup_namespace(
        &self,
        prefix: Option<&str>,
    ) -> PyResult<Option<PyNamespace>> {
        let prefix = optional_prefix(prefix)?;
        Ok(self
            .inner
            .lookup_namespace(prefix.as_ref())
            .map(PyNamespace::from))
    }

    /// Resolves an element name written as a string against the declarations in scope.
    ///
    /// Raises:
    ///     XmlSyntaxError: if the name is malformed.
    ///     XmlNamespaceError: if the prefix is undeclared or reserved.
    pub fn resolve_element_name(&self, name: &str) -> PyResult<PyQualifiedName> {
        self.inner
            .resolve_element_name(name)
            .map(PyQualifiedName::from)
            .map_err(to_py_err)
    }

    /// Resolves an attribute name written as a string against the declarations in scope.
    ///
    /// Raises:
    ///     XmlSyntaxError: if the name is malformed.
    ///     XmlNamespaceError: if the prefix is undeclared or reserved.
    pub fn resolve_attribute_name(&self, name: &str) -> PyResult<PyQualifiedName> {
        self.inner
            .resolve_attribute_name(name)
            .map(PyQualifiedName::from)
            .map_err(to_py_err)
    }

    /// Whether the element is reachable from the document root.
    pub fn is_attached(&self) -> bool {
        self.inner.is_attached()
    }

    /// The parent of this element, or `None`.
    pub fn parent(&self) -> Option<PyNode> {
        self.inner.parent().map(PyNode::from)
    }

    /// The children of this element in document order.
    pub fn children(&self) -> Vec<PyNode> {
        self.inner
            .children()
            .into_iter()
            .map(PyNode::from)
            .collect()
    }

    /// The element children of this element in document order.
    pub fn child_elements(&self) -> Vec<PyElement> {
        self.inner
            .child_elements()
            .into_iter()
            .map(PyElement::from)
            .collect()
    }

    /// Appends `child` as the last child of this element.
    ///
    /// Raises:
    ///     XmlDocumentError: for a foreign document, a cycle, or an attempt to attach the root.
    pub fn append_child(&self, child: PyNodeArg) -> PyResult<()> {
        self.inner
            .append_child_checked(Node::from(child))
            .map_err(to_py_err)
    }

    /// Detaches this element from its parent, returning the previous parent.
    pub fn detach(&self) -> Option<PyNode> {
        self.inner.detach().map(PyNode::from)
    }

    /// Creates a detached, shallow copy of this element in the same document.
    pub fn shallow_clone(&self) -> PyElement {
        PyElement::from(self.inner.shallow_clone())
    }

    /// Creates a detached, deep copy of this subtree in the same document. The GIL is released.
    ///
    /// Raises:
    ///     BaseException: whatever a Python signal handler raises to interrupt the operation (for
    ///         example `KeyboardInterrupt`).
    pub fn deep_clone(&self, py: Python<'_>) -> PyResult<PyElement> {
        let element = self.inner.clone();
        let size = element.document().node_count();
        run_interruptible(py, size, move || element.deep_clone()).map(PyElement::from)
    }

    /// Creates a detached, shallow copy of this element in `target`.
    pub fn shallow_clone_into(&self, target: &PyDocument) -> PyElement {
        PyElement::from(self.inner.shallow_clone_into(&target.inner))
    }

    /// Creates a detached, deep copy of this subtree in `target`. The GIL is released.
    ///
    /// Raises:
    ///     BaseException: whatever a Python signal handler raises to interrupt the operation (for
    ///         example `KeyboardInterrupt`).
    pub fn deep_clone_into(
        &self,
        py: Python<'_>,
        target: &PyDocument,
    ) -> PyResult<PyElement> {
        let element = self.inner.clone();
        let target = target.inner.clone();
        let size = element.document().node_count();
        run_interruptible(py, size, move || element.deep_clone_into(&target))
            .map(PyElement::from)
    }

    /// Whether this element and `other` refer to the same node.
    pub fn ptr_eq(&self, other: &PyElement) -> bool {
        self.inner.ptr_eq(&other.inner)
    }

    /// The element and its subtree as XML.
    pub fn __str__(&self) -> String {
        self.inner.to_string()
    }

    /// A hash consistent with `__eq__`: handles of one node hash equally.
    pub fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.inner.hash(&mut hasher);
        hasher.finish()
    }

    /// Two handles are equal if they refer to the same node of the same document.
    pub fn __eq__(&self, other: &Self) -> bool {
        self.inner == other.inner
    }

    /// A debug representation for interactive use, not a serialization of the value.
    pub fn __repr__(&self) -> String {
        format!(
            "Element({}, id={}, attached={})",
            self.inner.qualified_name(),
            raw_index(self.inner.id()),
            self.inner.is_attached()
        )
    }
}
