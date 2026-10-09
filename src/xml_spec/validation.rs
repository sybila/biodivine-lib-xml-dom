//! The whole-document parts of the XML specifications.
//!
//! Everything in this module is a *rule* that needs the namespace scope of a node but not the tree
//! itself, so it can live with the rest of the specification logic. The traversal that feeds these
//! rules is in [`crate::validation`].
//!
//! Two rules are worth calling out because they are the ones a "no magic" editing API needs
//! (requirement (3)):
//!
//! * `rule.namespace-usage.prefix-declaration-scope` — a declaration applies from the start-tag in
//!   which it appears to the end of the corresponding end-tag, so it applies to the element itself
//!   and to all of its descendants, and an inner declaration shadows an outer one. This module
//!   models that with [`NamespaceScope`].
//! * `rule.namespace-usage.default-namespace-scope` — the same for the default namespace, with the
//!   extra twist that an *unprefixed element name* takes whatever default namespace is in scope,
//!   while an unprefixed *attribute* name never does. Moving an element into a
//!   default-namespace scope therefore changes the meaning of its name, which is exactly the kind of
//!   inconsistency the task description asks to detect here rather than to repair silently.

use std::collections::BTreeMap;

use crate::namespace::Namespace;
use crate::qualified_name::QualifiedName;
use crate::xml_spec::{NCName, RESERVED_XML_URI};

/// The namespace bindings that are visible at one node.
///
/// The map holds the *effective* binding of each prefix: [`NamespaceScope::from_declarations`]
/// should be fed the innermost-first list a node sees, so that a shadowed outer declaration never
/// overwrites the inner one.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NamespaceScope {
    /// Prefix (`None` = default namespace) to URI; `None` means "declared as empty"
    /// (`xmlns=""` or, for a prefixed name, "not bound").
    bindings: BTreeMap<Option<NCName>, Option<String>>,
}

impl NamespaceScope {
    /// An empty scope.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds a scope from declarations given innermost-first.
    ///
    /// The first declaration of a prefix wins, which is how shadowing works: the caller passes the
    /// declarations of the node itself first, then those of its ancestors.
    pub fn from_declarations(
        declarations: impl IntoIterator<Item = (Option<NCName>, Option<Namespace>)>,
    ) -> Self {
        let mut scope = Self::new();
        for (prefix, namespace) in declarations {
            scope
                .bindings
                .entry(prefix)
                .or_insert_with(|| namespace.map(|namespace| namespace.uri().to_string()));
        }
        scope
    }

    /// Declares `prefix` explicitly. Overwrites an existing binding.
    pub fn insert(&mut self, prefix: Option<NCName>, uri: Option<impl Into<String>>) {
        self.bindings.insert(prefix, uri.map(Into::into));
    }

    /// The raw binding of `prefix`: `None` if the prefix is not declared at all, `Some(None)` if it
    /// is declared as empty, `Some(Some(uri))` if it is bound to `uri`.
    pub fn get(&self, prefix: Option<&NCName>) -> Option<Option<&str>> {
        self.bindings
            .get(&prefix.cloned())
            .map(|binding| binding.as_deref())
    }

    /// The binding of `prefix` as seen by a name, i.e. including the predefined `xml` prefix.
    ///
    /// `rule.namespace-basics.xml-prefix-fixed-binding`: `xml` is bound to
    /// `http://www.w3.org/XML/1998/namespace` whether or not it is declared, and it must not be
    /// bound to anything else (which [`Namespace`] already refuses to construct).
    pub fn resolve(&self, prefix: Option<&NCName>) -> Option<Option<&str>> {
        if let Some(prefix) = prefix
            && prefix == "xml"
        {
            return Some(Some(RESERVED_XML_URI));
        }
        self.get(prefix)
    }

    /// Whether the scope is empty (no declarations at all).
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }
}

/// Why a name cannot be resolved from a namespace scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeViolation {
    /// The name uses a prefix that is not declared in scope.
    ///
    /// `rule.namespace-usage.prefix-declared`.
    UndeclaredPrefix {
        /// The prefix of the name.
        prefix: NCName,
    },
    /// The name uses a prefix that is declared, but bound to a different URI.
    PrefixBoundToDifferentUri {
        /// The prefix of the name.
        prefix: NCName,
        /// The URI the name carries.
        expected: String,
        /// The URI actually bound in scope.
        actual: String,
    },
    /// The name is in the default namespace, but no default namespace is declared in scope.
    MissingDefaultNamespace {
        /// The URI the name carries.
        expected: String,
    },
    /// The name is in the default namespace, but the in-scope default namespace is a different URI.
    DefaultNamespaceMismatch {
        /// The URI the name carries.
        expected: String,
        /// The URI that is declared in scope.
        actual: String,
    },
    /// The name has no namespace while a default namespace is in scope, so writing the name would
    /// place it into that namespace.
    ///
    /// `rule.namespace-usage.default-namespace-scope`.
    UnprefixedNameTakesDefaultNamespace {
        /// The default namespace that is in scope.
        default_uri: String,
    },
    /// An attribute name carries a namespace without a prefix. Attributes are always written with
    /// a prefix or without a namespace at all, so such a name cannot be written.
    AttributeNamespaceWithoutPrefix {
        /// The URI the name carries.
        uri: String,
    },
    /// The reserved `xmlns` prefix is used as a name prefix.
    ///
    /// `rule.namespace-basics.xmlns-not-element-prefix`.
    ReservedPrefix {
        /// The prefix of the name.
        prefix: NCName,
    },
}

/// Checks an *element* name against the namespace scope of the element.
///
/// The element's own declarations are part of its scope: a declaration applies from the start-tag
/// in which it appears (`rule.namespace-usage.prefix-declaration-scope`), so the caller must
/// include them.
///
/// # Errors
///
/// Returns the [`ScopeViolation`] describing why the name is not resolvable.
pub fn check_element_name(
    name: &QualifiedName,
    scope: &NamespaceScope,
) -> Result<(), ScopeViolation> {
    match name.namespace() {
        None => match scope.resolve(None) {
            // rule: rule.namespace-usage.default-namespace-scope.md
            Some(Some(default_uri)) => Err(ScopeViolation::UnprefixedNameTakesDefaultNamespace {
                default_uri: default_uri.to_string(),
            }),
            _ => Ok(()),
        },
        Some(namespace) => match namespace.prefix() {
            Some(prefix) => resolve(prefix, namespace.uri(), scope),
            // An unprefixed name in the default namespace.
            None => match scope.resolve(None) {
                Some(Some(bound)) if bound == namespace.uri() => Ok(()),
                Some(Some(bound)) => Err(ScopeViolation::DefaultNamespaceMismatch {
                    expected: namespace.uri().to_string(),
                    actual: bound.to_string(),
                }),
                _ => Err(ScopeViolation::MissingDefaultNamespace {
                    expected: namespace.uri().to_string(),
                }),
            },
        },
    }
}

/// Checks an *attribute* name against the namespace scope of the element that carries it.
///
/// The default namespace never applies to attributes
/// (`rule.namespace-usage.default-namespace-not-attributes`), so an unprefixed attribute name is
/// always fine, but an attribute *with* a namespace must have a prefix.
///
/// # Errors
///
/// Returns the [`ScopeViolation`] describing why the name is not resolvable.
pub fn check_attribute_name(
    name: &QualifiedName,
    scope: &NamespaceScope,
) -> Result<(), ScopeViolation> {
    match name.namespace() {
        None => Ok(()),
        Some(namespace) => match namespace.prefix() {
            None => Err(ScopeViolation::AttributeNamespaceWithoutPrefix {
                uri: namespace.uri().to_string(),
            }),
            Some(prefix) => resolve(prefix, namespace.uri(), scope),
        },
    }
}

/// The shared prefix-resolution rule for names that carry a prefix.
fn resolve(prefix: &NCName, uri: &str, scope: &NamespaceScope) -> Result<(), ScopeViolation> {
    if prefix == "xmlns" {
        // rule: rule.namespace-basics.xmlns-not-element-prefix.md
        //
        // Unreachable through the public API: `Namespace::prefixed` refuses the reserved prefixes,
        // so no `QualifiedName` can carry `xmlns`. The check is kept because the rule is a
        // document-level one and because the *other* way to reach it (a name built by a future
        // constructor that bypasses `Namespace`) would otherwise be silent.
        return Err(ScopeViolation::ReservedPrefix {
            prefix: prefix.clone(),
        });
    }
    match scope.resolve(Some(prefix)) {
        Some(Some(bound)) if bound == uri => Ok(()),
        Some(Some(bound)) => Err(ScopeViolation::PrefixBoundToDifferentUri {
            prefix: prefix.clone(),
            expected: uri.to_string(),
            actual: bound.to_string(),
        }),
        // rule: rule.namespace-usage.prefix-declared.md
        _ => Err(ScopeViolation::UndeclaredPrefix {
            prefix: prefix.clone(),
        }),
    }
}

/// The `xml:id` attribute of an element, if it has one.
///
/// `xml:id` is the only attribute this library can prove to be of type `ID` without DTD processing,
/// so it is the one used for `rule.attributes.id-must-be-name` and
/// `rule.attributes.id-must-be-unique`. Crate-internal because it exposes the arena's element
/// payload, which is itself crate-internal.
pub(crate) fn xml_id_attribute(element: &crate::arena::ElementData) -> Option<&str> {
    let name = QualifiedName::new(
        crate::xml_spec::nc_name("id"),
        Some(
            Namespace::prefixed(RESERVED_XML_URI, "xml")
                .expect("the reserved xml namespace is always valid"),
        ),
    );
    element.attributes.get(&name).map(|value| value.as_ref())
}

/// The `xml:lang` attribute of an element, if it has one.
pub(crate) fn xml_lang_attribute(element: &crate::arena::ElementData) -> Option<&str> {
    attribute_in_xml_namespace(element, "lang")
}

/// The `xml:space` attribute of an element, if it has one.
pub(crate) fn xml_space_attribute(element: &crate::arena::ElementData) -> Option<&str> {
    attribute_in_xml_namespace(element, "space")
}

/// The value of the attribute `local` in the reserved XML namespace, if present.
fn attribute_in_xml_namespace<'a>(
    element: &'a crate::arena::ElementData,
    local: &str,
) -> Option<&'a str> {
    let name = QualifiedName::new(
        crate::xml_spec::nc_name(local),
        Some(
            Namespace::prefixed(RESERVED_XML_URI, "xml")
                .expect("the reserved xml namespace is always valid"),
        ),
    );
    element.attributes.get(&name).map(|value| value.as_ref())
}

/// Whether a namespace declaration is one the specifications forbid.
///
/// [`Namespace`] already refuses to construct such a declaration, so this can only fail for a value
/// that did not come from [`Namespace`]; it is checked anyway so that the validation pass is
/// self-contained (and because the rule is a document-level one:
/// `rule.namespace-basics.xml-prefix-fixed-binding`, `xmlns-prefix-not-declared`,
/// `xml-namespace-not-default`, `xmlns-namespace-not-default`,
/// `no-other-prefix-to-xml-namespace`, `no-other-prefix-to-xmlns-namespace`).
pub fn check_namespace_declaration(prefix: Option<&NCName>, uri: &str) -> Result<(), String> {
    crate::xml_spec::validate_namespace(uri, prefix).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml_spec::rules::assert_rule_exists;

    fn scope(entries: &[(&str, &str)]) -> NamespaceScope {
        let mut scope = NamespaceScope::new();
        for (prefix, uri) in entries {
            scope.insert(
                if prefix.is_empty() {
                    None
                } else {
                    Some(NCName::try_from(*prefix).unwrap())
                },
                Some(*uri),
            );
        }
        scope
    }

    fn name(prefix: Option<&str>, uri: Option<&str>, local: &str) -> QualifiedName {
        match (prefix, uri) {
            (None, None) => QualifiedName::without_namespace(local).unwrap(),
            (_, Some(uri)) => QualifiedName::with_namespace(
                local,
                &match prefix {
                    Some(prefix) => Namespace::prefixed(uri, prefix).unwrap(),
                    None => Namespace::without_prefix(uri).unwrap(),
                },
            )
            .unwrap(),
            (Some(_), None) => unreachable!("a prefix implies a namespace"),
        }
    }

    #[test]
    fn prefixed_names_must_be_declared() {
        // rule: rule.namespace-usage.prefix-declared.md
        assert_rule_exists("rule.namespace-usage.prefix-declared.md");
        let scope = scope(&[("ex", "http://example.com")]);
        assert_eq!(
            check_element_name(&name(Some("ex"), Some("http://example.com"), "a"), &scope),
            Ok(())
        );
        assert_eq!(
            check_element_name(&name(Some("nope"), Some("http://x"), "a"), &scope),
            Err(ScopeViolation::UndeclaredPrefix {
                prefix: NCName::try_from("nope").unwrap()
            })
        );
        assert_eq!(
            check_element_name(&name(Some("ex"), Some("http://other"), "a"), &scope),
            Err(ScopeViolation::PrefixBoundToDifferentUri {
                prefix: NCName::try_from("ex").unwrap(),
                expected: "http://other".to_string(),
                actual: "http://example.com".to_string(),
            })
        );
    }

    #[test]
    fn the_xml_prefix_needs_no_declaration() {
        // rule: rule.namespace-basics.xml-prefix-fixed-binding.md
        assert_rule_exists("rule.namespace-basics.xml-prefix-fixed-binding.md");
        let scope = NamespaceScope::new();
        assert_eq!(
            check_element_name(&name(Some("xml"), Some(RESERVED_XML_URI), "a"), &scope),
            Ok(())
        );
    }

    #[test]
    fn default_namespace_rules() {
        // rule: rule.namespace-usage.default-namespace-scope.md
        assert_rule_exists("rule.namespace-usage.default-namespace-scope.md");

        let with_default = scope(&[("", "http://default")]);
        // A name in the default namespace matches.
        assert_eq!(
            check_element_name(&name(None, Some("http://default"), "a"), &with_default),
            Ok(())
        );
        // A name in a different default namespace does not.
        assert_eq!(
            check_element_name(&name(None, Some("http://other"), "a"), &with_default),
            Err(ScopeViolation::DefaultNamespaceMismatch {
                expected: "http://other".to_string(),
                actual: "http://default".to_string(),
            })
        );
        // A name with no namespace inside a default-namespace scope would change meaning.
        assert_eq!(
            check_element_name(&name(None, None, "a"), &with_default),
            Err(ScopeViolation::UnprefixedNameTakesDefaultNamespace {
                default_uri: "http://default".to_string(),
            })
        );
        // Without a default namespace, both are fine.
        let empty = NamespaceScope::new();
        assert_eq!(check_element_name(&name(None, None, "a"), &empty), Ok(()));
        assert_eq!(
            check_element_name(&name(None, Some("http://default"), "a"), &empty),
            Err(ScopeViolation::MissingDefaultNamespace {
                expected: "http://default".to_string(),
            })
        );
    }

    #[test]
    fn attributes_ignore_the_default_namespace() {
        // rule: rule.namespace-usage.default-namespace-not-attributes.md
        assert_rule_exists("rule.namespace-usage.default-namespace-not-attributes.md");
        let with_default = scope(&[("", "http://default"), ("ex", "http://example.com")]);

        // Unprefixed attribute: fine even inside a default-namespace scope.
        assert_eq!(
            check_attribute_name(&name(None, None, "a"), &with_default),
            Ok(())
        );
        // Prefixed attribute: resolved normally.
        assert_eq!(
            check_attribute_name(
                &name(Some("ex"), Some("http://example.com"), "a"),
                &with_default
            ),
            Ok(())
        );
        // An attribute carrying the default namespace without a prefix cannot be written.
        assert_eq!(
            check_attribute_name(&name(None, Some("http://default"), "a"), &with_default),
            Err(ScopeViolation::AttributeNamespaceWithoutPrefix {
                uri: "http://default".to_string()
            })
        );
    }

    #[test]
    fn the_xmlns_prefix_can_never_become_a_name_prefix() {
        // rule: rule.namespace-basics.xmlns-not-element-prefix.md
        assert_rule_exists("rule.namespace-basics.xmlns-not-element-prefix.md");

        // The rule is enforced at *construction*: no `Namespace` can carry the reserved prefix, so
        // no name can either, and `ScopeViolation::ReservedPrefix` can only be reached by a name
        // built outside `Namespace` (which the type system does not currently allow). The
        // layer-C check in `resolve` is therefore a redundant self-check; this test pins the
        // construction-time enforcement that actually does the work, plus the reachable parts.
        assert!(Namespace::prefixed("http://x", "xmlns").is_err());
        assert!(Namespace::prefixed(RESERVED_XML_URI, "xmlns").is_err());

        let scope = scope(&[("ex", "http://example.com")]);
        assert_eq!(
            check_attribute_name(&name(Some("ex"), Some("http://example.com"), "a"), &scope),
            Ok(())
        );
    }

    #[test]
    fn inner_declarations_shadow_outer_ones() {
        // rule: rule.namespace-usage.prefix-declaration-scope.md
        assert_rule_exists("rule.namespace-usage.prefix-declaration-scope.md");
        let inner = Namespace::prefixed("http://inner", "ex").unwrap();
        let outer = Namespace::prefixed("http://outer", "ex").unwrap();
        let scope = NamespaceScope::from_declarations([
            (Some(NCName::try_from("ex").unwrap()), Some(inner)),
            (Some(NCName::try_from("ex").unwrap()), Some(outer)),
        ]);
        assert_eq!(
            scope.get(Some(&NCName::try_from("ex").unwrap())),
            Some(Some("http://inner"))
        );
    }
}
