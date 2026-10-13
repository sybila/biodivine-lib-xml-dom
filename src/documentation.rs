//! Project documentation: design notes, the bindings map and rule coverage.
//!
//! This module is documentation rather than API. It exists so that the reasoning behind the library
//! is reachable from the same place as the API reference, instead of living only in files that a
//! reader of the documentation would never find. Instructions for working on the repository itself
//! are in `docs/design/build.md`.
//!
//! | page | contents |
//! | --- | --- |
//! | [`architecture`] | how the library is built and why: the arena model, handles and the collector, the concurrency argument, the error model, the namespace contract, module layout, testing and trade-offs |
//! | [`bindings`] | the Rust ↔ Python mapping, item by item, with the deliberate differences |
//! | [`rule_coverage`] | which XML specification rules this library acts on, in which layer, and which are deliberately out of scope |
//!
//! The pages are the repository's design documents, included verbatim with [`include_str!`], so a
//! change to a document cannot leave the API documentation behind — and the documents stay readable
//! in a plain editor or on a code host.
//!
//! The XML rules themselves live next to the specification types in
//! [`xml_spec::rule_set`](crate::xml_spec::rule_set), and the documentation book that teaches the
//! library ships with the repository next to them.

/// How the library is built and why, including the trade-offs each decision accepts.
#[doc = include_str!("../docs/design/architecture.md")]
pub mod architecture {}

/// The Rust ↔ Python mapping: what is mirrored, and what is deliberately different.
#[doc = include_str!("../docs/design/bindings.md")]
pub mod bindings {}

/// Which specification rules this library acts on, in which layer, and what is out of scope.
#[doc = include_str!("../docs/design/rule-coverage.md")]
pub mod rule_coverage {}
