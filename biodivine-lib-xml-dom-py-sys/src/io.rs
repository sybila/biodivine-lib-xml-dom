//! The parsing and serialization implementations behind the `Document` methods.
//!
//! The Python-visible entry points are the methods of `Document` (see [`PyDocument`]), mirroring
//! the Rust API, where parsing and serialization are `Document` methods as well. Each one touches
//! the whole document, so it runs through [`run_interruptible`]: without the GIL, and interruptible
//! by Python signal handlers. The closures capture only owned Rust values (the Rust types are
//! `Send + Sync`), never a Python reference.

use biodivine_lib_xml_dom as xml;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::path::PathBuf;

use crate::document::PyDocument;
use crate::interrupt::run_interruptible;
use crate::write_options::PyWriteOptions;

/// Parses an XML document from a string; see [`PyDocument::parse_string`].
pub(crate) fn parse_string(py: Python<'_>, source: &str) -> PyResult<PyDocument> {
    let owned = source.to_owned();
    run_interruptible(py, owned.len(), move || xml::Document::parse_string(&owned))
        .map(PyDocument::from)
}

/// Parses an XML document from bytes; see [`PyDocument::parse_bytes`].
pub(crate) fn parse_bytes(
    py: Python<'_>,
    data: &Bound<'_, PyBytes>,
) -> PyResult<PyDocument> {
    let owned = data.as_bytes().to_vec();
    run_interruptible(py, owned.len(), move || xml::Document::parse_bytes(&owned))
        .map(PyDocument::from)
}

/// Parses an XML document from a file; see [`PyDocument::parse_file`].
pub(crate) fn parse_file(py: Python<'_>, path: PathBuf) -> PyResult<PyDocument> {
    // The size only decides whether a signal can interrupt the parse; a file whose size cannot be
    // read fails inside the parse with a proper error.
    let size = std::fs::metadata(&path)
        .map(|metadata| usize::try_from(metadata.len()).unwrap_or(usize::MAX))
        .unwrap_or(0);
    run_interruptible(py, size, move || xml::Document::parse_file(&path))
        .map(PyDocument::from)
}

/// Serializes a document into a string; see [`PyDocument::write_string`].
pub(crate) fn write_string(
    py: Python<'_>,
    document: &PyDocument,
    options: Option<PyWriteOptions>,
) -> PyResult<String> {
    let document = document.inner.clone();
    let options = options.map(xml::WriteOptions::from).unwrap_or_default();
    let size = document.node_count();
    run_interruptible(py, size, move || document.write_string_with(&options))
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
    let size = document.node_count();
    run_interruptible(py, size, move || document.write_file_with(&path, &options))
}
