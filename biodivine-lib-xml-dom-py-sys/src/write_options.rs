//! Serialization options and the XML declaration.

use biodivine_lib_xml_dom::{
    DeclarationStyle, EmptyElementStyle, WriteOptions, xml_spec::XmlDeclaration,
};
use pyo3::prelude::*;
use pyo3::types::PyType;

/// When [`PyWriteOptions`] should write the `<?xml ...?>` declaration.
#[pyclass(
    name = "DeclarationStyle",
    module = "biodivine_lib_xml_dom._sys",
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

impl From<DeclarationStyle> for PyDeclarationStyle {
    fn from(value: DeclarationStyle) -> Self {
        match value {
            DeclarationStyle::Never => PyDeclarationStyle::Never,
            DeclarationStyle::IfPresent => PyDeclarationStyle::IfPresent,
            DeclarationStyle::Always => PyDeclarationStyle::Always,
        }
    }
}

/// How [`PyWriteOptions`] writes an element that has no children.
#[pyclass(
    name = "EmptyElementStyle",
    module = "biodivine_lib_xml_dom._sys",
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

impl From<EmptyElementStyle> for PyEmptyElementStyle {
    fn from(value: EmptyElementStyle) -> Self {
        match value {
            EmptyElementStyle::SelfClosing => PyEmptyElementStyle::SelfClosing,
            EmptyElementStyle::ExplicitEndTag => PyEmptyElementStyle::ExplicitEndTag,
        }
    }
}

/// Options for the `write_*_with` functions.
///
/// The defaults reproduce a parsed document as closely as the data model allows: the declaration is
/// written when the document has one, and empty elements are written as `<a/>`.
#[pyclass(
    name = "WriteOptions",
    module = "biodivine_lib_xml_dom._sys",
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
        WriteOptions {
            declaration: value.declaration.into(),
            empty_elements: value.empty_elements.into(),
        }
    }
}

impl From<WriteOptions> for PyWriteOptions {
    fn from(value: WriteOptions) -> Self {
        PyWriteOptions {
            declaration: value.declaration.into(),
            empty_elements: value.empty_elements.into(),
        }
    }
}

#[pymethods]
impl PyWriteOptions {
    /// Creates options; both fields default to the values described on the class.
    #[new]
    #[pyo3(signature = (declaration = None, empty_elements = None))]
    pub fn new(
        declaration: Option<PyDeclarationStyle>,
        empty_elements: Option<PyEmptyElementStyle>,
    ) -> Self {
        let defaults = WriteOptions::default();
        PyWriteOptions {
            declaration: declaration.unwrap_or(defaults.declaration.into()),
            empty_elements: empty_elements.unwrap_or(defaults.empty_elements.into()),
        }
    }

    /// The default options.
    #[classmethod]
    pub fn default(_cls: &Bound<'_, PyType>) -> Self {
        WriteOptions::default().into()
    }

    /// A debug representation for interactive use, not a serialization of the value.
    pub fn __repr__(&self) -> String {
        format!(
            "WriteOptions(declaration={:?}, empty_elements={:?})",
            self.declaration, self.empty_elements
        )
    }

    /// Equality with another handle of the same type: the same node/document/value
    /// (Rust's `PartialEq`). Python object identity is *not* part of it.
    pub fn __eq__(&self, other: &Self) -> bool {
        self.declaration == other.declaration && self.empty_elements == other.empty_elements
    }
}

/// The `<?xml version="1.0" encoding="UTF-8"?>` declaration of a document.
///
/// It is metadata about the document, not part of the tree, and the serializer writes it according
/// to [`PyWriteOptions::declaration`].
#[pyclass(
    name = "XmlDeclaration",
    module = "biodivine_lib_xml_dom._sys",
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
    /// Creates a declaration from its parts, without validating them (validation is the parser's
    /// job, exactly as in Rust).
    #[new]
    #[pyo3(signature = (version = "1.0".to_string(), encoding = None, standalone = None))]
    pub fn new(version: String, encoding: Option<String>, standalone: Option<bool>) -> Self {
        XmlDeclaration::new(version, encoding, standalone).into()
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

    /// Equality with another handle of the same type: the same node/document/value
    /// (Rust's `PartialEq`). Python object identity is *not* part of it.
    pub fn __eq__(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
