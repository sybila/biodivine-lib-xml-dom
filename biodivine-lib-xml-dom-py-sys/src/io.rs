//! The parsing and serialization entry points.
//!
//! These mirror the crate-root functions of `biodivine-lib-xml-dom` one for one. The GIL is
//! released around them, because each of them touches the whole document; the closures capture only
//! owned Rust values (the Rust types are `Send + Sync`), never a Python reference.

use biodivine_lib_xml_dom as xml;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::path::PathBuf;

use crate::document::PyDocument;
use crate::error::to_py_err;
use crate::write_options::PyWriteOptions;

/// Parses an XML document from a string.
///
/// Raises `XmlSyntaxError` (well-formedness), `XmlNamespaceError` (namespaces) or `XmlIoError` as
/// appropriate; the message is the Rust error text.
#[pyfunction]
#[pyo3(signature = (source))]
pub fn parse_string(py: Python<'_>, source: &str) -> PyResult<PyDocument> {
    let owned = source.to_owned();
    py.detach(move || xml::parse_string(&owned))
        .map(PyDocument::from)
        .map_err(to_py_err)
}

/// Parses an XML document from bytes (which must be valid UTF-8).
#[pyfunction]
#[pyo3(signature = (data))]
pub fn parse_bytes(py: Python<'_>, data: &Bound<'_, PyBytes>) -> PyResult<PyDocument> {
    let owned = data.as_bytes().to_vec();
    py.detach(move || xml::parse_bytes(&owned))
        .map(PyDocument::from)
        .map_err(to_py_err)
}

/// Parses an XML document from a file.
///
/// Raises `XmlIoError` if the file cannot be read.
#[pyfunction]
#[pyo3(signature = (path))]
pub fn parse_file(py: Python<'_>, path: PathBuf) -> PyResult<PyDocument> {
    py.detach(move || xml::parse_file(&path))
        .map(PyDocument::from)
        .map_err(to_py_err)
}

/// Serializes a document into a string, using the default options.
///
/// A document without a root element and without a declaration serializes to the empty string.
#[pyfunction]
#[pyo3(signature = (document))]
pub fn write_string(py: Python<'_>, document: &PyDocument) -> PyResult<String> {
    write_string_with(py, document, None)
}

/// Serializes a document into a string with explicit options.
#[pyfunction]
#[pyo3(signature = (document, options = None))]
pub fn write_string_with(
    py: Python<'_>,
    document: &PyDocument,
    options: Option<PyWriteOptions>,
) -> PyResult<String> {
    let document = document.inner.clone();
    let options = options.map(xml::WriteOptions::from).unwrap_or_default();
    py.detach(move || xml::write_string_with(&document, &options))
        .map_err(to_py_err)
}

/// Writes a document to a file, using the default options.
#[pyfunction]
#[pyo3(signature = (document, path))]
pub fn write_file(py: Python<'_>, document: &PyDocument, path: PathBuf) -> PyResult<()> {
    write_file_with(py, document, path, None)
}

/// Writes a document to a file with explicit options.
#[pyfunction]
#[pyo3(signature = (document, path, options = None))]
pub fn write_file_with(
    py: Python<'_>,
    document: &PyDocument,
    path: PathBuf,
    options: Option<PyWriteOptions>,
) -> PyResult<()> {
    let document = document.inner.clone();
    let options = options.map(xml::WriteOptions::from).unwrap_or_default();
    py.detach(move || xml::write_file_with(&document, &path, &options))
        .map_err(to_py_err)
}
