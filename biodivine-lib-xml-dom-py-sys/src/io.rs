//! The parsing and serialization implementations behind the `Document` methods.
//!
//! The Python-visible entry points are the methods of `Document` (see [`PyDocument`]), mirroring
//! the Rust API, where parsing and serialization are `Document` methods as well. The GIL is released
//! around the work, because each entry point touches the whole document; the closures capture only
//! owned Rust values (the Rust types are `Send + Sync`), never a Python reference.

use biodivine_lib_xml_dom as xml;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::path::PathBuf;

use crate::document::PyDocument;
use crate::error::to_py_err;
use crate::write_options::PyWriteOptions;

/// Parses an XML document from a string; see [`PyDocument::parse_string`].
pub(crate) fn parse_string(py: Python<'_>, source: &str) -> PyResult<PyDocument> {
    let owned = source.to_owned();
    py.detach(move || {
        cancel_this::on_python(move || xml::Document::parse_string(&owned))
    })
    .map(PyDocument::from)
    .map_err(to_py_err)
}

/// Parses an XML document from bytes; see [`PyDocument::parse_bytes`].
pub(crate) fn parse_bytes(
    py: Python<'_>,
    data: &Bound<'_, PyBytes>,
) -> PyResult<PyDocument> {
    let owned = data.as_bytes().to_vec();
    py.detach(move || {
        cancel_this::on_python(move || xml::Document::parse_bytes(&owned))
    })
    .map(PyDocument::from)
    .map_err(to_py_err)
}

/// Parses an XML document from a file; see [`PyDocument::parse_file`].
pub(crate) fn parse_file(py: Python<'_>, path: PathBuf) -> PyResult<PyDocument> {
    py.detach(move || cancel_this::on_python(move || xml::Document::parse_file(&path)))
        .map(PyDocument::from)
        .map_err(to_py_err)
}

/// Serializes a document into a string; see [`PyDocument::write_string`].
pub(crate) fn write_string(
    py: Python<'_>,
    document: &PyDocument,
    options: Option<PyWriteOptions>,
) -> PyResult<String> {
    let document = document.inner.clone();
    let options = options.map(xml::WriteOptions::from).unwrap_or_default();
    py.detach(move || {
        cancel_this::on_python(move || document.write_string_with(&options))
    })
    .map_err(to_py_err)
}

/// Writes a document to a file; see [`PyDocument::write_file`].
pub(crate) fn write_file(
    py: Python<'_>,
    document: &PyDocument,
    path: PathBuf,
    options: Option<PyWriteOptions>,
) -> PyResult<()> {
    let document = document.inner.clone();
    let options = options.map(xml::WriteOptions::from).unwrap_or_default();
    py.detach(move || {
        cancel_this::on_python(move || document.write_file_with(&path, &options))
    })
    .map_err(to_py_err)
}
