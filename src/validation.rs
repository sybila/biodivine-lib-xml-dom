//! Whole-document validation.
//!
//! Some properties of a document can only be judged by looking at the document as a whole: is
//! every used prefix actually declared *here*, rather than somewhere else? Are all `xml:id` values
//! distinct? Those are the checks in this module, and they are deliberately separate from the
//! editing API:
//!
//! * editing never checks namespace integrity — you can remove a declaration that
//!   a subtree still relies on, or move an element into a scope where its prefix means something
//!   else, and nothing complains;
//! * [`Document::validate`] then reports *every* problem it can find in one pass, so a caller can
//!   fix them all in one sweep instead of discovering them one failed operation at a time.
//!
//! The rules themselves live in the `xml_spec` module, because they are statements about XML rather
//! than about this library. This module is the traversal that feeds them, plus the error type that
//! makes a *set* of problems usable as a single error.
//!
//! The other structural rules (the tree is a tree, the root is an element without a parent, the
//! namespace declarations respect the reserved bindings) are guaranteed by construction: the arena
//! maintains them on every mutation and [`crate::Namespace`] refuses an illegal binding, so they
//! are not part of the validation report.
//!
//! # What is checked
//!
//! | group | rule files |
//! | --- | --- |
//! | structure | [`rule.well-formedness.document-production`](crate::xml_spec::rule_set::rule_well_formedness_document_production) (a root element exists) |
//! | namespace scope | [`rule.namespace-usage.prefix-declared`](crate::xml_spec::rule_set::rule_namespace_usage_prefix_declared), `prefix-declaration-scope`, `default-namespace-scope`, `default-namespace-not-attributes`, [`rule.namespace-basics.xmlns-not-element-prefix`](crate::xml_spec::rule_set::rule_namespace_basics_xmlns_not_element_prefix), `xml-prefix-fixed-binding` |
//! | values | [`rule.attributes.id-must-be-name`](crate::xml_spec::rule_set::rule_attributes_id_must_be_name), `id-must-be-unique`, [`rule.document-structure.xml-lang-must-be-bcp47-or-empty`](crate::xml_spec::rule_set::rule_document_structure_xml_lang_must_be_bcp47_or_empty), `xml-space-must-be-enumerated-default-preserve` |
//!
//! # What is deliberately not checked
//!
//! Everything that needs DTD processing: attribute *types*, content models, and the two
//! `xml:lang`/`xml:space` "must be declared" constraints. Those are validity, not well-formedness,
//! and `DOCTYPE` validation is out of scope for this library; the two rules are marked as such in
//! the rule coverage list ([`documentation::rule_coverage`](crate::documentation::rule_coverage)).
//!
//! # Detached nodes
//!
//! A node that is not reachable from the root is *not* an error by itself: a detached node is a
//! normal state of the data model. Node-local rules (name resolution against the node's own scope,
//! `xml:lang`/`xml:space` values, `xml:id` syntax) are checked for detached nodes too, so a subtree
//! can be prepared and validated before it is attached. Document-wide rules (`xml:id` uniqueness)
//! are checked over the attached tree only: a detached `deep_clone` of a subtree would otherwise be
//! reported as a duplicate of its original.
//!
//! [`Document::validate`]: crate::Document::validate

use std::fmt;

use crate::arena::{Arena, NodeData};
use crate::cancellation::CancellationCheck;
use crate::error::XmlResult;
use crate::node_id::NodeId;
use crate::qualified_name::QualifiedName;
use crate::xml_spec::validation::{
    NamespaceScope, ScopeViolation, check_attribute_name, check_element_name,
    xml_id_attribute, xml_lang_attribute, xml_space_attribute,
};
use crate::xml_spec::{NCName, is_valid_language_tag, is_valid_xml_space_value};

/// What kind of problem whole-document validation found.
///
/// Every variant is named after the rule it comes from; the corresponding rule file is named in
/// the variant's documentation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ValidationErrorKind {
    // -------------------------------------------------------------------------------------
    // Structure.
    // -------------------------------------------------------------------------------------
    /// The document has no root element ([`rule.well-formedness.document-production`](crate::xml_spec::rule_set::rule_well_formedness_document_production)).
    MissingRoot,

    // -------------------------------------------------------------------------------------
    // Namespace scope.
    // -------------------------------------------------------------------------------------
    /// A name uses a prefix that is not declared in scope ([`rule.namespace-usage.prefix-declared`](crate::xml_spec::rule_set::rule_namespace_usage_prefix_declared)).
    UndeclaredPrefix {
        /// The prefix of the name.
        prefix: NCName,
    },
    /// A name uses a prefix that is declared, but bound to a different URI
    /// ([`rule.namespace-usage.prefix-declaration-scope`](crate::xml_spec::rule_set::rule_namespace_usage_prefix_declaration_scope)).
    PrefixBoundToDifferentUri {
        /// The prefix of the name.
        prefix: NCName,
        /// The URI the name carries.
        expected: String,
        /// The URI actually bound in scope.
        actual: String,
    },
    /// An element name is in the default namespace, but no default namespace is in scope
    /// ([`rule.namespace-usage.default-namespace-scope`](crate::xml_spec::rule_set::rule_namespace_usage_default_namespace_scope)).
    MissingDefaultNamespace {
        /// The URI the name carries.
        expected: String,
    },
    /// An element name is in the default namespace, but a different one is in scope
    /// ([`rule.namespace-usage.default-namespace-scope`](crate::xml_spec::rule_set::rule_namespace_usage_default_namespace_scope)).
    DefaultNamespaceMismatch {
        /// The URI the name carries.
        expected: String,
        /// The URI that is declared in scope.
        actual: String,
    },
    /// An unprefixed element name has no namespace, but a default namespace is in scope, so the
    /// name would mean something else once written out
    /// ([`rule.namespace-usage.default-namespace-scope`](crate::xml_spec::rule_set::rule_namespace_usage_default_namespace_scope)).
    UnprefixedNameTakesDefaultNamespace {
        /// The default namespace that is in scope.
        default_uri: String,
    },
    /// An attribute name carries a namespace without a prefix, so it cannot be written
    /// ([`rule.namespace-usage.default-namespace-not-attributes`](crate::xml_spec::rule_set::rule_namespace_usage_default_namespace_not_attributes)).
    AttributeNamespaceWithoutPrefix {
        /// The URI the name carries.
        uri: String,
    },
    /// A name uses the reserved `xmlns` prefix
    /// ([`rule.namespace-basics.xmlns-not-element-prefix`](crate::xml_spec::rule_set::rule_namespace_basics_xmlns_not_element_prefix)).
    ReservedPrefix {
        /// The prefix of the name.
        prefix: NCName,
    },

    // -------------------------------------------------------------------------------------
    // Values.
    // -------------------------------------------------------------------------------------
    /// An `xml:id` value is not a valid `NCName` ([`rule.attributes.id-must-be-name`](crate::xml_spec::rule_set::rule_attributes_id_must_be_name)).
    XmlIdIsNotAName {
        /// The offending value.
        value: String,
    },
    /// Two elements carry the same `xml:id` value ([`rule.attributes.id-must-be-unique`](crate::xml_spec::rule_set::rule_attributes_id_must_be_unique)).
    DuplicateXmlId {
        /// The repeated value.
        id: NCName,
    },
    /// An `xml:lang` value is not a language tag and not empty
    /// ([`rule.document-structure.xml-lang-must-be-bcp47-or-empty`](crate::xml_spec::rule_set::rule_document_structure_xml_lang_must_be_bcp47_or_empty)).
    InvalidXmlLang {
        /// The offending value.
        value: String,
    },
    /// An `xml:space` value is neither `default` nor `preserve`
    /// ([`rule.document-structure.xml-space-must-be-enumerated-default-preserve`](crate::xml_spec::rule_set::rule_document_structure_xml_space_must_be_enumerated_default_preserve)).
    InvalidXmlSpace {
        /// The offending value.
        value: String,
    },
}

impl ValidationErrorKind {
    /// The rule this kind enforces, as its identifier (`rule.well-formedness.document-production`),
    /// for error messages and documentation. The summary itself is the corresponding module of
    /// [`crate::xml_spec::rule_set`].
    pub fn rule(&self) -> &'static str {
        match self {
            Self::MissingRoot => "rule.well-formedness.document-production",
            Self::UndeclaredPrefix { .. } => "rule.namespace-usage.prefix-declared",
            Self::PrefixBoundToDifferentUri { .. } => {
                "rule.namespace-usage.prefix-declaration-scope"
            }
            Self::MissingDefaultNamespace { .. }
            | Self::DefaultNamespaceMismatch { .. }
            | Self::UnprefixedNameTakesDefaultNamespace { .. } => {
                "rule.namespace-usage.default-namespace-scope"
            }
            Self::AttributeNamespaceWithoutPrefix { .. } => {
                "rule.namespace-usage.default-namespace-not-attributes"
            }
            Self::ReservedPrefix { .. } => {
                "rule.namespace-basics.xmlns-not-element-prefix"
            }
            Self::XmlIdIsNotAName { .. } => "rule.attributes.id-must-be-name",
            Self::DuplicateXmlId { .. } => "rule.attributes.id-must-be-unique",
            Self::InvalidXmlLang { .. } => {
                "rule.document-structure.xml-lang-must-be-bcp47-or-empty"
            }
            Self::InvalidXmlSpace { .. } => {
                "rule.document-structure.xml-space-must-be-enumerated-default-preserve"
            }
        }
    }
}

/// One problem found by [`crate::Document::validate`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    kind: ValidationErrorKind,
    node: Option<NodeId>,
    message: String,
}

impl ValidationError {
    /// Creates an issue attached to a node.
    fn at(kind: ValidationErrorKind, node: NodeId, message: impl Into<String>) -> Self {
        Self {
            kind,
            node: Some(node),
            message: message.into(),
        }
    }

    /// Creates an issue about the document as a whole.
    fn document(kind: ValidationErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            node: None,
            message: message.into(),
        }
    }

    /// What kind of problem this is.
    pub fn kind(&self) -> &ValidationErrorKind {
        &self.kind
    }

    /// The node the problem is attached to.
    ///
    /// `None` only for problems about the document as a whole (a missing root element), where there
    /// is no node to point at.
    pub fn node(&self) -> Option<NodeId> {
        self.node
    }

    /// A human-readable description of the problem.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.node {
            Some(node) => {
                write!(f, "node {node}: {} [{}]", self.message, self.kind.rule())
            }
            None => write!(f, "{} [{}]", self.message, self.kind.rule()),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Every problem [`crate::Document::validate`] found, as one error value.
///
/// This is the payload of [`crate::XmlError::Validation`], so it always holds at least one
/// problem. The list is deterministic (nodes in arena order, and within one node the checks run in
/// a fixed order), so comparing two validation runs is meaningful.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationErrors {
    errors: Vec<ValidationError>,
}

impl ValidationErrors {
    /// Wraps a list of problems that validation found.
    ///
    /// Crate-internal so that a [`ValidationErrors`] value always comes from a validation run and
    /// is never empty.
    pub(crate) fn new(errors: Vec<ValidationError>) -> Self {
        debug_assert!(
            !errors.is_empty(),
            "a validation error lists at least one problem"
        );
        Self { errors }
    }

    /// The issues, in deterministic order.
    pub fn as_slice(&self) -> &[ValidationError] {
        &self.errors
    }

    /// Iterates over the issues, in deterministic order.
    pub fn iter(&self) -> std::slice::Iter<'_, ValidationError> {
        self.errors.iter()
    }

    /// How many issues were found.
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Whether no issues were found.
    ///
    /// Always `false` for a value returned by this crate; provided because a length without
    /// `is_empty` is unidiomatic.
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// The issues, as an owned vector.
    pub fn into_vec(self) -> Vec<ValidationError> {
        self.errors
    }
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} validation problem", self.errors.len())?;
        if self.errors.len() != 1 {
            write!(f, "s")?;
        }
        for error in &self.errors {
            write!(f, "\n  - {error}")?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationErrors {}

impl IntoIterator for ValidationErrors {
    type Item = ValidationError;
    type IntoIter = std::vec::IntoIter<ValidationError>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl<'a> IntoIterator for &'a ValidationErrors {
    type Item = &'a ValidationError;
    type IntoIter = std::slice::Iter<'a, ValidationError>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.iter()
    }
}

/// Runs every whole-document check over `arena` and collects all problems.
///
/// This is the implementation behind [`crate::Document::validate`]; it takes the arena rather than
/// the document so that it cannot acquire the lock again (see the single-lock invariant in
/// the [design notes](crate::documentation::architecture)).
///
/// The walk honours cancellation: a trigger registered on this thread aborts it with
/// [`crate::XmlError::Cancelled`] instead of scanning the whole document.
pub(crate) fn validate_arena(arena: &Arena) -> XmlResult<Vec<ValidationError>> {
    let mut errors = Vec::new();
    let mut check = CancellationCheck::new();
    check_root(arena, &mut errors);
    check_nodes(arena, &mut errors, &mut check)?;
    check_xml_id_uniqueness(arena, &mut errors, &mut check)?;
    Ok(errors)
}

/// The document has a root element.
///
/// The other structural properties (one root, an acyclic tree whose parent and child links agree)
/// are invariants that [`Arena`] maintains on every mutation, so there is nothing to report; debug
/// builds assert them after every collection.
fn check_root(arena: &Arena, errors: &mut Vec<ValidationError>) {
    if arena.root().is_none() {
        errors.push(ValidationError::document(
            ValidationErrorKind::MissingRoot,
            "the document has no root element",
        ));
    }
}

/// The per-node checks: names against their scope, and attribute values.
fn check_nodes(
    arena: &Arena,
    errors: &mut Vec<ValidationError>,
    check: &mut CancellationCheck,
) -> XmlResult<()> {
    for id in arena.occupied_ids() {
        check.tick()?;
        let NodeData::Element(element) = arena.data(id) else {
            continue;
        };
        let scope = scope_of(arena, id);

        if let Err(violation) = check_element_name(&element.name, &scope) {
            errors.push(scope_error(id, &element.name, violation, false));
        }
        for name in element.attributes.keys() {
            if let Err(violation) = check_attribute_name(name, &scope) {
                errors.push(scope_error(id, name, violation, true));
            }
        }

        check_xml_attribute_values(id, element, errors);
    }
    Ok(())
}

/// The `xml:id`, `xml:lang` and `xml:space` value checks for one element.
///
/// These are node-local rules, so they apply to detached nodes as well: a subtree can be prepared
/// and validated before it is attached.
fn check_xml_attribute_values(
    id: NodeId,
    element: &crate::arena::ElementData,
    errors: &mut Vec<ValidationError>,
) {
    if let Some(value) = xml_id_attribute(element)
        && NCName::try_from(value).is_err()
    {
        // rule: rule.attributes.id-must-be-name
        errors.push(ValidationError::at(
            ValidationErrorKind::XmlIdIsNotAName {
                value: value.to_string(),
            },
            id,
            format!("`xml:id` value {value:?} is not a valid NCName"),
        ));
    }
    if let Some(value) = xml_lang_attribute(element)
        && !is_valid_language_tag(value)
    {
        // rule: rule.document-structure.xml-lang-must-be-bcp47-or-empty
        errors.push(ValidationError::at(
            ValidationErrorKind::InvalidXmlLang {
                value: value.to_string(),
            },
            id,
            format!(
                "`xml:lang` value {value:?} is not a language tag and is not empty"
            ),
        ));
    }
    if let Some(value) = xml_space_attribute(element)
        && !is_valid_xml_space_value(value)
    {
        // rule: rule.document-structure.xml-space-must-be-enumerated-default-preserve
        errors.push(ValidationError::at(
            ValidationErrorKind::InvalidXmlSpace {
                value: value.to_string(),
            },
            id,
            format!("`xml:space` value {value:?} is neither `default` nor `preserve`"),
        ));
    }
}

/// `xml:id` uniqueness over the attached tree.
///
/// Uniqueness is a property of the *document*, i.e. of the tree reachable from the root, so a
/// detached node is never compared with an attached one. That is what makes the normal workflow
/// "clone a subtree, edit the copy, attach it later" possible: the detached `deep_clone` carries
/// the same `xml:id` values as its original, and only the act of attaching it creates the
/// duplicate that the rule is about. Each node beyond the first one carrying a value gets exactly
/// one issue, so a value shared by three attached elements produces two issues.
fn check_xml_id_uniqueness(
    arena: &Arena,
    errors: &mut Vec<ValidationError>,
    check: &mut CancellationCheck,
) -> XmlResult<()> {
    let Some(root) = arena.root() else {
        return Ok(());
    };
    // Breadth-first over the attached tree, so detached nodes are not compared: a detached
    // `deep_clone` of a subtree legitimately carries the same `xml:id` values as its original.
    let mut seen: std::collections::BTreeMap<String, NodeId> =
        std::collections::BTreeMap::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(root);
    while let Some(id) = queue.pop_front() {
        check.tick()?;
        if let NodeData::Element(element) = arena.data(id)
            && let Some(value) = xml_id_attribute(element)
            && let Ok(name) = NCName::try_from(value)
        {
            // rule: rule.attributes.id-must-be-unique
            if let Some(previous) = seen.insert(value.to_string(), id) {
                errors.push(ValidationError::at(
                    ValidationErrorKind::DuplicateXmlId { id: name },
                    id,
                    format!(
                        "`xml:id` value {value:?} is already used by node {previous}"
                    ),
                ));
            }
        }
        for &child in arena.children(id) {
            queue.push_back(child);
        }
    }
    Ok(())
}

/// The namespace scope of one node: its own declarations first, then those of its ancestors.
fn scope_of(arena: &Arena, id: NodeId) -> NamespaceScope {
    NamespaceScope::from_declarations(arena.namespaces_in_scope(id))
}

/// Turns a [`ScopeViolation`] into an issue.
fn scope_error(
    node: NodeId,
    name: &QualifiedName,
    violation: ScopeViolation,
    attribute: bool,
) -> ValidationError {
    let what = if attribute { "attribute" } else { "element" };
    let (kind, message) = match violation {
        ScopeViolation::UndeclaredPrefix { prefix } => (
            ValidationErrorKind::UndeclaredPrefix {
                prefix: prefix.clone(),
            },
            format!(
                "the {what} name `{name}` uses the prefix `{prefix}`, which is not declared in scope"
            ),
        ),
        ScopeViolation::PrefixBoundToDifferentUri {
            prefix,
            expected,
            actual,
        } => (
            ValidationErrorKind::PrefixBoundToDifferentUri {
                prefix: prefix.clone(),
                expected: expected.clone(),
                actual: actual.clone(),
            },
            format!(
                "the {what} name `{name}` uses the prefix `{prefix}`, which is bound to `{actual}` rather than `{expected}`"
            ),
        ),
        ScopeViolation::MissingDefaultNamespace { expected } => (
            ValidationErrorKind::MissingDefaultNamespace {
                expected: expected.clone(),
            },
            format!(
                "the {what} name `{name}` is in the default namespace `{expected}`, but no default namespace is declared in scope"
            ),
        ),
        ScopeViolation::DefaultNamespaceMismatch { expected, actual } => (
            ValidationErrorKind::DefaultNamespaceMismatch {
                expected: expected.clone(),
                actual: actual.clone(),
            },
            format!(
                "the {what} name `{name}` is in the namespace `{expected}`, but the default namespace in scope is `{actual}`"
            ),
        ),
        ScopeViolation::UnprefixedNameTakesDefaultNamespace { default_uri } => (
            ValidationErrorKind::UnprefixedNameTakesDefaultNamespace {
                default_uri: default_uri.clone(),
            },
            format!(
                "the element name `{name}` has no namespace, but the default namespace `{default_uri}` is in scope, so writing it out would place it in that namespace"
            ),
        ),
        ScopeViolation::AttributeNamespaceWithoutPrefix { uri } => (
            ValidationErrorKind::AttributeNamespaceWithoutPrefix { uri: uri.clone() },
            format!(
                "the attribute name `{name}` is in the namespace `{uri}` but has no prefix, so it cannot be written"
            ),
        ),
        ScopeViolation::ReservedPrefix { prefix } => (
            ValidationErrorKind::ReservedPrefix {
                prefix: prefix.clone(),
            },
            format!("the {what} name `{name}` uses the reserved prefix `{prefix}`"),
        ),
    };
    ValidationError::at(kind, node, message)
}
