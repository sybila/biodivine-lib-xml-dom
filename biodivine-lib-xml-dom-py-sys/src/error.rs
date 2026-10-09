//! The Python exception hierarchy.
//!
//! Rust reports failures as a typed `XmlError`; Python reports them as exceptions. The mapping is
//! total and lives in exactly one place ([`to_py_err`]), so a new Rust variant cannot silently
//! become a generic `Exception`:
//!
//! | Rust | Python |
//! | --- | --- |
//! | `XmlError::Io` | `XmlIoError` |
//! | name/content/markup/encoding problems (see below) | `XmlSyntaxError` |
//! | namespace problems | `XmlNamespaceError` |
//! | API misuse (foreign document, cycle, bad index, …) | `XmlDocumentError` |
//! | `Document::validate` failures | `XmlValidationError` |
//! | anything else | `XmlError` |
//!
//! `XmlSyntaxError` covers [`XmlError::InvalidName`], [`XmlError::InvalidText`],
//! [`XmlError::InvalidComment`], [`XmlError::InvalidCData`],
//! [`XmlError::InvalidProcessingInstruction`], [`XmlError::DuplicateAttribute`],
//! [`XmlError::MalformedXml`], [`XmlError::InvalidUtf8`], [`XmlError::UnsupportedEncoding`],
//! [`XmlError::UnsupportedXmlVersion`], [`XmlError::UndeclaredEntityReference`],
//! [`XmlError::InvalidCharacterReference`], [`XmlError::MultipleRootElements`],
//! [`XmlError::ContentOutsideRoot`] and [`XmlError::MissingRoot`] — i.e. everything that is a
//! property of the input document. `XmlNamespaceError` covers [`XmlError::InvalidNamespace`],
//! [`XmlError::ReservedPrefix`] and [`XmlError::UndeclaredPrefix`].
//!
//! Every exception message is the Rust `Display` of the error, so a Python user sees the same text
//! a Rust user would.

use biodivine_lib_xml_dom::XmlError as RustXmlError;
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

create_exception!(
    _sys,
    XmlError,
    PyException,
    "Base class of every error this library raises."
);
create_exception!(
    _sys,
    XmlSyntaxError,
    XmlError,
    "The input document is not well-formed XML."
);
create_exception!(
    _sys,
    XmlNamespaceError,
    XmlError,
    "A namespace is used in a way the specification forbids."
);
create_exception!(
    _sys,
    XmlDocumentError,
    XmlError,
    "A document operation is not allowed (foreign document, cycle, bad index, ...)."
);
create_exception!(
    _sys,
    XmlIoError,
    XmlError,
    "An underlying I/O operation failed."
);
create_exception!(
    _sys,
    XmlValidationError,
    XmlError,
    "Whole-document validation found problems; the list is in `args[0]`."
);

/// Maps a Rust [`RustXmlError`] onto the Python exception hierarchy.
pub(crate) fn to_py_err(error: RustXmlError) -> PyErr {
    let message = error.to_string();
    match error {
        RustXmlError::Io(_) => XmlIoError::new_err(message),

        RustXmlError::InvalidNamespace(_)
        | RustXmlError::ReservedPrefix(_)
        | RustXmlError::UndeclaredPrefix(_) => XmlNamespaceError::new_err(message),

        RustXmlError::ForeignDocument
        | RustXmlError::CycleDetected
        | RustXmlError::NodeNotFound(_)
        | RustXmlError::NotAnElement(_)
        | RustXmlError::NodeHasNoParent(_)
        | RustXmlError::NotAChild(_, _)
        | RustXmlError::IndexOutOfBounds { .. }
        | RustXmlError::RootHasParent
        | RustXmlError::CannotAttachRoot => XmlDocumentError::new_err(message),

        RustXmlError::InvalidName(_)
        | RustXmlError::InvalidText(_)
        | RustXmlError::InvalidComment(_)
        | RustXmlError::InvalidCData(_)
        | RustXmlError::InvalidProcessingInstruction(_)
        | RustXmlError::DuplicateAttribute(_)
        | RustXmlError::MalformedXml(_)
        | RustXmlError::InvalidUtf8(_)
        | RustXmlError::UnsupportedEncoding(_)
        | RustXmlError::UnsupportedXmlVersion(_)
        | RustXmlError::UndeclaredEntityReference(_)
        | RustXmlError::InvalidCharacterReference(_)
        | RustXmlError::MultipleRootElements
        | RustXmlError::ContentOutsideRoot
        | RustXmlError::MissingRoot => XmlSyntaxError::new_err(message),
    }
}

/// Adds the exception types to the module so that Python can catch them by name.
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = module.py();
    module.add("XmlError", py.get_type::<XmlError>())?;
    module.add("XmlSyntaxError", py.get_type::<XmlSyntaxError>())?;
    module.add("XmlNamespaceError", py.get_type::<XmlNamespaceError>())?;
    module.add("XmlDocumentError", py.get_type::<XmlDocumentError>())?;
    module.add("XmlIoError", py.get_type::<XmlIoError>())?;
    module.add("XmlValidationError", py.get_type::<XmlValidationError>())?;
    Ok(())
}
