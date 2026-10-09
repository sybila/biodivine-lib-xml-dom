//! The `_sys` layer: a thin PyO3 mirror of `biodivine-lib-xml-dom`.
//!
//! # Where this layer sits
//!
//! The project ships three layers, and this is the middle one:
//!
//! ```text
//! biodivine-lib-xml-dom         the Rust library (no PyO3 anywhere, by design)
//! biodivine-lib-xml-dom-py-sys  this crate: one Python class/function per Rust item, delegating
//!                               without adding behaviour
//! biodivine_lib_xml_dom         the pure-Python package (see `python/`): argument coercion,
//!                               Pythonic protocols, conveniences - and nothing else
//! ```
//!
//! The rules this layer follows:
//!
//! * **No policy.** A binding does what the Rust method does, with the same name (in
//!   `snake_case`), the same arguments and the same result. The rare exceptions are listed with a
//!   reason in `docs/design/BINDINGS.md`.
//! * **No panics cross the boundary.** Rust has an ergonomic variant that panics and a `_checked`
//!   variant that returns a `Result`. Python has no panics, so this layer exposes the `_checked`
//!   semantics under the plain name: a failing operation raises. Nothing in this crate panics on
//!   input that came from Python, and the workspace sets no `panic = "abort"`, so even a bug that
//!   did panic would surface as `PanicException` rather than killing the interpreter.
//! * **No double locking.** No binding acquires two document locks: the cross-document copies go
//!   through `Node::deep_clone_into`, which snapshots the source, releases it and only then writes
//!   to the target. The deadlock-freedom argument of the Rust core therefore carries over to
//!   Python unchanged ("at most one lock is ever held").
//! * **The GIL is released around the slow operations.** Parsing, serializing, validating and the
//!   cross-document copies hold the document lock for a time proportional to the document, so they
//!   run under [`Python::detach`]. Everything else is a lock acquisition and a small read, where
//!   releasing the GIL would cost more than it saves; see the "GIL policy" section below.
//!
//! # GIL policy
//!
//! | operation | GIL | why |
//! | --- | --- | --- |
//! | `parse_string`, `parse_bytes`, `parse_file` | released | parses a whole document |
//! | `write_string`, `write_file` | released | serializes a whole document |
//! | `Document.validate` | released | walks the whole arena |
//! | `Node.deep_clone_into`, `Element.deep_clone_into` | released | snapshots and rebuilds a subtree |
//! | everything else | held | one lock acquisition and a bounded read or write |
//!
//! The closures passed to [`Python::detach`] capture only owned Rust values (the Rust types are
//! `Send + Sync`), never a `Bound`/`Py` reference, which is what PyO3 requires.

mod document;
mod element;
mod error;
mod io;
mod namespace;
mod node;
mod node_id;
mod qualified_name;
mod validation;
mod write_options;

#[cfg(test)]
mod tests;

use pyo3::prelude::*;

pub use crate::document::PyDocument;
pub use crate::element::PyElement;
pub use crate::io::{
    parse_bytes, parse_file, parse_string, write_file, write_file_with, write_string,
    write_string_with,
};
pub use crate::namespace::PyNamespace;
pub use crate::node::{PyNode, PyNodeArg};
pub use crate::node_id::PyNodeId;
pub use crate::qualified_name::PyQualifiedName;
pub use crate::validation::{PyNodeKind, PyXmlValidationError, PyXmlValidationErrors};
pub use crate::write_options::{PyDeclarationStyle, PyEmptyElementStyle, PyWriteOptions};

/// The native module, imported by the pure-Python package as `biodivine_lib_xml_dom._sys`.
///
/// Everything it exposes is documented on the Rust side; the Python package mirrors the names so
/// that `biodivine_lib_xml_dom.Document` is the Pythonic wrapper of `_sys.Document`.
#[pymodule]
fn _sys(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "__doc__",
        "Native bindings of biodivine-lib-xml-dom (the `_sys` layer).",
    )?;
    // The version comes from this crate's manifest, which inherits it from `[workspace.package]`,
    // so the Python package can report the same number without a literal of its own.
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;

    module.add_class::<PyNodeId>()?;
    module.add_class::<PyNamespace>()?;
    module.add_class::<PyQualifiedName>()?;
    module.add_class::<PyDocument>()?;
    module.add_class::<PyNode>()?;
    module.add_class::<PyElement>()?;
    module.add_class::<PyNodeKind>()?;
    module.add_class::<PyWriteOptions>()?;
    module.add_class::<PyDeclarationStyle>()?;
    module.add_class::<PyEmptyElementStyle>()?;
    module.add_class::<crate::write_options::PyXmlDeclaration>()?;

    module.add_function(wrap_pyfunction!(parse_string, module)?)?;
    module.add_function(wrap_pyfunction!(parse_bytes, module)?)?;
    module.add_function(wrap_pyfunction!(parse_file, module)?)?;
    module.add_function(wrap_pyfunction!(write_string, module)?)?;
    module.add_function(wrap_pyfunction!(write_string_with, module)?)?;
    module.add_function(wrap_pyfunction!(write_file, module)?)?;
    module.add_function(wrap_pyfunction!(write_file_with, module)?)?;

    error::register(module)?;
    validation::register(module)?;

    Ok(())
}
