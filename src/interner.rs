//! Per-document deduplication of repetitive data.
//!
//! Requirement (1) asks to keep the `Arc`-based scheme for deduplication of namespaces and other
//! repetitive data. [`Namespace`] and [`QualifiedName`] already share their payload through an
//! `Arc`, so *cloning* them is cheap; the interner additionally makes sure that a document never
//! holds two distinct `Arc` allocations for the same value.
//!
//! The interner is keyed by value, not by pointer: both types implement `Hash`/`Eq` structurally
//! (`Namespace` by URI and prefix, `QualifiedName` by local name and namespace URI). Nothing in
//! the public API depends on the pointer identity of a name or namespace — interning is a pure
//! memory optimisation, so a name that was *not* interned (e.g. one coming from another document)
//! behaves exactly like an interned one.

use crate::namespace::Namespace;
use crate::qualified_name::QualifiedName;
use crate::xml_spec::NCName;
use std::collections::HashMap;

/// Deduplicates the [`Namespace`] and [`QualifiedName`] values stored in one document.
#[derive(Debug, Default)]
pub(crate) struct Interner {
    namespaces: HashMap<Namespace, Namespace>,
    /// Keyed by the *structural* name, i.e. local name plus namespace *including* its prefix; see
    /// the module documentation for why the prefix has to be part of the key.
    names: HashMap<(NCName, Option<Namespace>), QualifiedName>,
}

impl Interner {
    /// Returns the document's canonical value for `namespace`, inserting it if necessary.
    pub(crate) fn namespace(&mut self, namespace: Namespace) -> Namespace {
        self.namespaces
            .entry(namespace.clone())
            .or_insert(namespace)
            .clone()
    }

    /// Returns the document's canonical value for `name`, inserting it if necessary.
    ///
    /// The returned name is value-equal to the argument *and* carries the same prefix.
    pub(crate) fn name(&mut self, name: QualifiedName) -> QualifiedName {
        let key = (name.local_name().clone(), name.namespace().cloned());
        self.names.entry(key).or_insert(name).clone()
    }

    /// Number of distinct namespaces currently interned.
    #[cfg(test)]
    pub(crate) fn namespace_count(&self) -> usize {
        self.namespaces.len()
    }

    /// Number of distinct qualified names currently interned.
    #[cfg(test)]
    pub(crate) fn name_count(&self) -> usize {
        self.names.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Interning must never change the name it is given: two names that are value-equal but carry
    /// different prefixes must both come back unchanged, or a document would silently lose (or
    /// gain) a prefix. Found through the Python bindings, where the rewritten prefix was visible.
    #[test]
    fn interning_never_rewrites_a_prefix() {
        let mut interner = Interner::default();
        let prefixed = QualifiedName::with_namespace(
            "a",
            &Namespace::prefixed("http://example.com", "ex").unwrap(),
        )
        .unwrap();
        let default = QualifiedName::with_namespace(
            "a",
            &Namespace::without_prefix("http://example.com").unwrap(),
        )
        .unwrap();

        let first = interner.name(prefixed.clone());
        let second = interner.name(default.clone());
        assert_eq!(first.to_string(), "ex:a");
        assert_eq!(second.to_string(), "a");
        // The same pair in the other order gives the same answer.
        let mut interner = Interner::default();
        interner.name(default.clone());
        assert_eq!(interner.name(prefixed.clone()).to_string(), "ex:a");
        assert_eq!(interner.name(default).to_string(), "a");
    }

    #[test]
    fn interning_deduplicates_by_value() {
        let mut interner = Interner::default();

        let first = interner.namespace(Namespace::prefixed("http://example.com", "ex").unwrap());
        let second = interner.namespace(Namespace::prefixed("http://example.com", "ex").unwrap());
        assert_eq!(interner.namespace_count(), 1);
        // Both handles share one allocation.
        assert!(first.shares_data_with(&second));

        // A different prefix for the same URI is a different value.
        interner.namespace(Namespace::without_prefix("http://example.com").unwrap());
        assert_eq!(interner.namespace_count(), 2);

        let name = QualifiedName::with_namespace("item", &first).unwrap();
        let same = QualifiedName::with_namespace("item", &second).unwrap();
        let first = interner.name(name);
        let second = interner.name(same);
        assert_eq!(interner.name_count(), 1);
        assert!(first.shares_data_with(&second));
    }
}
