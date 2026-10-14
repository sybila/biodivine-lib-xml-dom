//! The `_sys` layer: a thin PyO3 mirror of `biodivine-lib-xml-dom`.
//!
//! # Where this layer sits
//!
//! The XML library reaches Python through four layers, and this crate is the second one:
//!
//! ```text
//! biodivine-lib-xml-dom         the Rust library (no PyO3 anywhere, by design)
//! biodivine-lib-xml-dom-py-sys  this crate: one Python class/function per Rust item, delegating
//!                               without adding behaviour
//! biodivine-sys                 the native extension of the Biodivine Python bindings: links
//!                               every `_sys` crate into one module and calls [`register`] to fill
//!                               its `biodivine_sys.xml_dom` submodule
//! biodivine                     the pure-Python package; `biodivine.xml_dom` adds argument
//!                               coercion, Pythonic protocols and conveniences - and nothing else
//! ```
//!
//! The last two layers live in the `biodivine-py` repository. This crate is not a Python extension
//! on its own: it is a library of `#[pyclass]` types that the aggregate extension links in. Because
//! every Biodivine `_sys` crate ends up in the same native module, a binding crate of another
//! library can depend on this one and accept or return [`PyDocument`], [`PyElement`] and the other
//! types directly, and Python sees one class for each of them rather than one per extension. That
//! is also why the crate is never published: it is only ever consumed by the aggregate extension
//! (and by binding crates that build on it), pinned to a matching revision of the core crate.
//!
//! The rules this layer follows:
//!
//! * **No policy.** A binding does what the Rust method does, with the same name (in
//!   `snake_case`), the same arguments and the same result. The rare exceptions are listed with a
//!   reason in [`documentation::bindings`](biodivine_lib_xml_dom::documentation::bindings).
//! * **No panics cross the boundary.** Rust has an ergonomic variant that panics and a `_checked`
//!   variant that returns a `Result`. Python has no panics, so this layer exposes the `_checked`
//!   semantics under the plain name: a failing operation raises. Nothing in this crate panics on
//!   input that came from Python, and the workspace sets no `panic = "abort"`, so even a bug that
//!   did panic would surface as `PanicException` rather than killing the interpreter.
//! * **No double locking.** No binding acquires two document locks: the cross-document copies go
//!   through `Node::deep_clone_into`, which snapshots the source, releases it and only then writes
//!   to the target. The deadlock-freedom argument of the Rust core therefore carries over to
//!   Python unchanged ("at most one lock is ever held").
//! * **The GIL is released around the slow operations, and signals can interrupt them.** The
//!   operations whose cost is proportional to the document run through the `interrupt` module;
//!   everything else is a lock acquisition and a small read, where releasing the GIL would cost
//!   more than it saves.
//!
//! # GIL policy
//!
//! | operation | GIL | why |
//! | --- | --- | --- |
//! | `parse_string`, `parse_bytes`, `parse_file` | released | parses a whole document |
//! | `write_string`, `write_file` | released | serializes a whole document |
//! | `Document.validate`, `is_valid`, `validation_errors` | released | walks the whole arena |
//! | `Document.collect_garbage` | released | walks the whole arena |
//! | `deep_clone`, `deep_clone_into` (on `Node` and `Element`) | released | copies a subtree |
//! | everything else | held | one lock acquisition and a bounded read or write |
//!
//! The closures that run without the GIL capture only owned Rust values (the Rust types are
//! `Send + Sync`), never a `Bound`/`Py` reference, which is what PyO3 requires.
#![deny(rustdoc::broken_intra_doc_links, rustdoc::private_intra_doc_links)]

mod document;
mod element;
mod error;
mod gc;
mod interrupt;
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
pub use crate::gc::PyGcStats;
pub use crate::namespace::PyNamespace;
pub use crate::node::{PyNode, PyNodeArg};
pub use crate::node_id::PyNodeId;
pub use crate::qualified_name::PyQualifiedName;
pub use crate::validation::{PyNodeKind, PyValidationError};
pub use crate::write_options::{
    PyDeclarationStyle, PyEmptyElementStyle, PyWriteOptions, PyXmlDeclaration,
};

/// Adds every class, exception and constant of the XML bindings to `module`.
///
/// The aggregate extension (`biodivine-sys`) creates the `biodivine_sys.xml_dom` submodule and
/// calls this function on it; the module path is also the one every class of this crate declares,
/// so the classes and the module agree on where they live.
///
/// # Errors
///
/// Fails with the Python exception raised by the interpreter when an attribute cannot be added to
/// `module`.
pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "__doc__",
        "Native bindings of biodivine-lib-xml-dom (the `_sys` layer).",
    )?;
    // The version comes from this crate's manifest, which inherits it from `[workspace.package]`,
    // so the Python package can report the version of the XML library it wraps.
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;

    module.add_class::<PyNodeId>()?;
    module.add_class::<PyGcStats>()?;
    module.add_class::<PyNamespace>()?;
    module.add_class::<PyQualifiedName>()?;
    module.add_class::<PyDocument>()?;
    module.add_class::<PyNode>()?;
    module.add_class::<PyElement>()?;
    module.add_class::<PyNodeKind>()?;
    module.add_class::<PyWriteOptions>()?;
    module.add_class::<PyDeclarationStyle>()?;
    module.add_class::<PyEmptyElementStyle>()?;
    module.add_class::<PyXmlDeclaration>()?;

    error::register(module)?;
    validation::register(module)?;

    Ok(())
}
