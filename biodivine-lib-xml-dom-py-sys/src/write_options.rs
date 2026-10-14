//! Serialization options and the XML declaration.

use biodivine_lib_xml_dom::{
    DeclarationStyle, EmptyElementStyle, WriteOptions, xml_spec::XmlDeclaration,
};
use pyo3::prelude::*;

use crate::error::to_py_err;

/// When [`PyWriteOptions`] should write the `<?xml ...?>` declaration.
#[pyclass(
    name = "DeclarationStyle",
    module = "biodivine_sys.xml_dom",
    frozen,
    eq,
    eq_int,
    from_py_object
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PyDeclarationStyle {
    /// Never write a declaration.
    Never = 0,
    /// Write a declaration if the document has one (the default).
    IfPresent = 1,
    /// Always write a declaration.
    Always = 2,
}

impl From<PyDeclarationStyle> for DeclarationStyle {
    fn from(value: PyDeclarationStyle) -> Self {
        match value {
            PyDeclarationStyle::Never => DeclarationStyle::Never,
            PyDeclarationStyle::IfPresent => DeclarationStyle::IfPresent,
            PyDeclarationStyle::Always => DeclarationStyle::Always,
        }
    }
}

/// How [`PyWriteOptions`] writes an element that has no children.
#[pyclass(
    name = "EmptyElementStyle",
    module = "biodivine_sys.xml_dom",
    frozen,
    eq,
    eq_int,
    from_py_object
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PyEmptyElementStyle {
    /// `<a/>` (the default).
    SelfClosing = 0,
    /// `<a></a>`.
    ExplicitEndTag = 1,
}

impl From<PyEmptyElementStyle> for EmptyElementStyle {
    fn from(value: PyEmptyElementStyle) -> Self {
        match value {
            PyEmptyElementStyle::SelfClosing => EmptyElementStyle::SelfClosing,
            PyEmptyElementStyle::ExplicitEndTag => EmptyElementStyle::ExplicitEndTag,
        }
    }
}

/// Options for `Document.write_string` and `Document.write_file`.
///
/// The defaults reproduce a parsed document as closely as the data model allows: the declaration is
/// written when the document has one, and empty elements are written as `<a/>`.
#[pyclass(
    name = "WriteOptions",
    module = "biodivine_sys.xml_dom",
    from_py_object
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PyWriteOptions {
    /// When to write the XML declaration.
    #[pyo3(get, set)]
    pub declaration: PyDeclarationStyle,
    /// How to write elements without children.
    #[pyo3(get, set)]
    pub empty_elements: PyEmptyElementStyle,
}

impl From<PyWriteOptions> for WriteOptions {
    fn from(value: PyWriteOptions) -> Self {
        WriteOptions::default()
            .with_declaration(value.declaration.into())
            .with_empty_elements(value.empty_elements.into())
    }
}

#[pymethods]
impl PyWriteOptions {
    /// Creates options; both fields default to the values described on the class.
    #[new]
    #[pyo3(signature = (
        declaration = PyDeclarationStyle::IfPresent,
        empty_elements = PyEmptyElementStyle::SelfClosing,
    ))]
    pub fn new(
        declaration: PyDeclarationStyle,
        empty_elements: PyEmptyElementStyle,
    ) -> Self {
        PyWriteOptions {
            declaration,
            empty_elements,
        }
    }

    /// A debug representation for interactive use, not a serialization of the value.
    pub fn __repr__(&self) -> String {
        format!(
            "WriteOptions(declaration={:?}, empty_elements={:?})",
            self.declaration, self.empty_elements
        )
    }

    /// Two option sets are equal if both fields agree.
    pub fn __eq__(&self, other: &Self) -> bool {
        self.declaration == other.declaration
            && self.empty_elements == other.empty_elements
    }
}

/// The `<?xml version="1.0" encoding="UTF-8"?>` declaration of a document.
///
/// It is metadata about the document, not part of the tree, and the serializer writes it according
/// to [`PyWriteOptions::declaration`].
#[pyclass(
    name = "XmlDeclaration",
    module = "biodivine_sys.xml_dom",
    frozen,
    from_py_object
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PyXmlDeclaration {
    pub(crate) inner: XmlDeclaration,
}

impl From<XmlDeclaration> for PyXmlDeclaration {
    fn from(inner: XmlDeclaration) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyXmlDeclaration {
    /// Creates a declaration from its parts.
    ///
    /// Raises:
    ///     XmlSyntaxError: if the version is not 1.0 or the encoding does not name UTF-8.
    #[new]
    #[pyo3(signature = (version = "1.0".to_string(), encoding = None, standalone = None))]
    pub fn new(
        version: String,
        encoding: Option<String>,
        standalone: Option<bool>,
    ) -> PyResult<Self> {
        XmlDeclaration::new(version, encoding, standalone)
            .map(Self::from)
            .map_err(to_py_err)
    }

    /// The declaration this library writes by default: version 1.0, UTF-8.
    #[staticmethod]
    pub fn utf8() -> Self {
        XmlDeclaration::utf8().into()
    }

    /// The declared XML version.
    pub fn version(&self) -> String {
        self.inner.version().to_string()
    }

    /// The declared encoding, if the declaration has one.
    pub fn encoding(&self) -> Option<String> {
        self.inner.encoding().map(|encoding| encoding.to_string())
    }

    /// The declared `standalone` value, if the declaration has one.
    pub fn standalone(&self) -> Option<bool> {
        self.inner.standalone()
    }

    /// The declaration as it appears in a document.
    pub fn __str__(&self) -> String {
        self.inner.to_string()
    }

    /// A debug representation for interactive use, not a serialization of the value.
    pub fn __repr__(&self) -> String {
        format!("XmlDeclaration({:?})", self.inner.to_string())
    }

    /// Two declarations are equal if version, encoding and `standalone` agree.
    pub fn __eq__(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
