//! Whole-document validation.
//!
//! Requirement (4)(2): some properties of a document can only be judged by looking at the document
//! as a whole — is every used prefix actually declared *here*, rather than somewhere else? are all
//! `xml:id` values distinct? is the tree still a tree? Those are the checks in this module, and
//! they are deliberately separate from the editing API:
//!
//! * editing never checks namespace integrity (requirement (3)) — you can remove a declaration that
//!   a subtree still relies on, or move an element into a scope where its prefix means something
//!   else, and nothing complains;
//! * [`Document::validate`] then reports *every* problem it can find in one pass, so a caller can
//!   fix them all in one sweep instead of discovering them one failed operation at a time.
//!
//! The rules themselves live in [`crate::xml_spec::validation`], because they are statements about
//! XML rather than about this library. This module is the traversal that feeds them, plus the error
//! type that makes a *set* of problems usable as a single error.
//!
//! # What is checked
//!
//! | group | rule files |
//! | --- | --- |
//! | structure (self-check of the arena's invariants) | `rule.well-formedness.single-root-element`, `rule.well-formedness.elements-nest-properly` |
//! | namespace scope | `rule.namespace-usage.prefix-declared`, `prefix-declaration-scope`, `default-namespace-scope`, `default-namespace-not-attributes`, `rule.namespace-basics.xmlns-not-element-prefix`, `xml-prefix-fixed-binding` |
//! | namespace declarations | `rule.namespace-basics.*` reserved-binding rules |
//! | values | `rule.attributes.id-must-be-name`, `id-must-be-unique`, `rule.document-structure.xml-lang-must-be-bcp47-or-empty`, `xml-space-must-be-enumerated-default-preserve` |
//!
//! # What is deliberately not checked
//!
//! Everything that needs DTD processing: attribute *types*, content models, and the two
//! `xml:lang`/`xml:space` "must be declared" constraints. Those are validity, not well-formedness,
//! and AGENTS.md puts `DOCTYPE` validation out of scope; the two rules are marked as such in
//! `docs/design/evidence/rule-enforcement.md`.
//!
//! # Detached nodes
//!
//! A node that is not reachable from the root is *not* an error by itself — requirement (2) makes
//! detached nodes a normal state. Node-local rules (name resolution against the node's own scope,
//! `xml:lang`/`xml:space` values, `xml:id` syntax) are checked for detached nodes too, so a subtree
//! can be prepared and validated before it is attached. Document-wide rules (`xml:id` uniqueness)
//! are checked over the attached tree only: a detached `deep_clone` of a subtree would otherwise be
//! reported as a duplicate of its original.
//!
//! [`Document::validate`]: crate::Document::validate

use std::fmt;

use crate::arena::{Arena, NodeData, NodeId};
use crate::qualified_name::QualifiedName;
use crate::xml_spec::validation::{
    NamespaceScope, ScopeViolation, check_attribute_name, check_element_name,
    check_namespace_declaration, xml_id_attribute, xml_lang_attribute, xml_space_attribute,
};
use crate::xml_spec::{NCName, is_valid_language_tag, is_valid_xml_space_value};

/// What kind of problem whole-document validation found.
///
/// Every variant is named after the rule it comes from; the corresponding rule file is named in
/// the variant's documentation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationErrorKind {
    // -------------------------------------------------------------------------------------
    // Structure. Every variant below except `MissingRoot` is a *defensive self-check*: the arena
    // maintains those invariants on every mutation (parent and child links are written together,
    // `set_root` replaces the root rather than appending a second one, and cycles are rejected by
    // `Arena::attach`), so no sequence of public API calls can produce them. They are checked
    // anyway so that the guarantee does not depend on the maintaining code being correct, and so
    // that the corresponding document-level rules have an explicit enforcement point. Only
    // `MissingRoot` is reachable through the public API (a document to which no root was set).
    // -------------------------------------------------------------------------------------
    /// The document has no root element (`rule.well-formedness.document-production`).
    MissingRoot,
    /// The root of the document is not an element (`rule.well-formedness.single-root-element`).
    RootIsNotAnElement,
    /// The root of the document has a parent (`rule.well-formedness.single-root-element`).
    RootHasParent,
    /// A node's parent link and the parent's child list disagree
    /// (`rule.well-formedness.elements-nest-properly`).
    ParentChildMismatch,
    /// Following parent links from a node returns to it
    /// (`rule.well-formedness.elements-nest-properly`).
    CyclicStructure,

    // -------------------------------------------------------------------------------------
    // Namespace scope.
    // -------------------------------------------------------------------------------------
    /// A name uses a prefix that is not declared in scope (`rule.namespace-usage.prefix-declared`).
    UndeclaredPrefix {
        /// The prefix of the name.
        prefix: NCName,
    },
    /// A name uses a prefix that is declared, but bound to a different URI
    /// (`rule.namespace-usage.prefix-declaration-scope`).
    PrefixBoundToDifferentUri {
        /// The prefix of the name.
        prefix: NCName,
        /// The URI the name carries.
        expected: String,
        /// The URI actually bound in scope.
        actual: String,
    },
    /// An element name is in the default namespace, but no default namespace is in scope
    /// (`rule.namespace-usage.default-namespace-scope`).
    MissingDefaultNamespace {
        /// The URI the name carries.
        expected: String,
    },
    /// An element name is in the default namespace, but a different one is in scope
    /// (`rule.namespace-usage.default-namespace-scope`).
    DefaultNamespaceMismatch {
        /// The URI the name carries.
        expected: String,
        /// The URI that is declared in scope.
        actual: String,
    },
    /// An unprefixed element name has no namespace, but a default namespace is in scope, so the
    /// name would mean something else once written out
    /// (`rule.namespace-usage.default-namespace-scope`).
    UnprefixedNameTakesDefaultNamespace {
        /// The default namespace that is in scope.
        default_uri: String,
    },
    /// An attribute name carries a namespace without a prefix, so it cannot be written
    /// (`rule.namespace-usage.default-namespace-not-attributes`).
    AttributeNamespaceWithoutPrefix {
        /// The URI the name carries.
        uri: String,
    },
    /// A name uses the reserved `xmlns` prefix
    /// (`rule.namespace-basics.xmlns-not-element-prefix`).
    ReservedPrefix {
        /// The prefix of the name.
        prefix: NCName,
    },
    /// A namespace declaration violates one of the reserved-binding rules
    /// (`rule.namespace-basics.*`).
    IllegalNamespaceDeclaration {
        /// The prefix of the declaration, or `None` for the default namespace.
        prefix: Option<NCName>,
        /// The URI it binds.
        uri: String,
        /// The problem, as described by [`crate::Namespace`]'s validation rules.
        reason: String,
    },

    // -------------------------------------------------------------------------------------
    // Values.
    // -------------------------------------------------------------------------------------
    /// An `xml:id` value is not a valid `NCName` (`rule.attributes.id-must-be-name`).
    XmlIdIsNotAName {
        /// The offending value.
        value: String,
    },
    /// Two elements carry the same `xml:id` value (`rule.attributes.id-must-be-unique`).
    DuplicateXmlId {
        /// The repeated value.
        id: NCName,
    },
    /// An `xml:lang` value is not a language tag and not empty
    /// (`rule.document-structure.xml-lang-must-be-bcp47-or-empty`).
    InvalidXmlLang {
        /// The offending value.
        value: String,
    },
    /// An `xml:space` value is neither `default` nor `preserve`
    /// (`rule.document-structure.xml-space-must-be-enumerated-default-preserve`).
    InvalidXmlSpace {
        /// The offending value.
        value: String,
    },
}

impl ValidationErrorKind {
    /// The rule file this kind comes from, for error messages and documentation.
    pub fn rule(&self) -> &'static str {
        match self {
            Self::MissingRoot => "rule.well-formedness.document-production.md",
            Self::RootIsNotAnElement | Self::RootHasParent => {
                "rule.well-formedness.single-root-element.md"
            }
            Self::ParentChildMismatch | Self::CyclicStructure => {
                "rule.well-formedness.elements-nest-properly.md"
            }
            Self::UndeclaredPrefix { .. } => "rule.namespace-usage.prefix-declared.md",
            Self::PrefixBoundToDifferentUri { .. } => {
                "rule.namespace-usage.prefix-declaration-scope.md"
            }
            Self::MissingDefaultNamespace { .. }
            | Self::DefaultNamespaceMismatch { .. }
            | Self::UnprefixedNameTakesDefaultNamespace { .. } => {
                "rule.namespace-usage.default-namespace-scope.md"
            }
            Self::AttributeNamespaceWithoutPrefix { .. } => {
                "rule.namespace-usage.default-namespace-not-attributes.md"
            }
            Self::ReservedPrefix { .. } => "rule.namespace-basics.xmlns-not-element-prefix.md",
            Self::IllegalNamespaceDeclaration { .. } => {
                "rule.namespace-basics.xml-prefix-fixed-binding.md"
            }
            Self::XmlIdIsNotAName { .. } => "rule.attributes.id-must-be-name.md",
            Self::DuplicateXmlId { .. } => "rule.attributes.id-must-be-unique.md",
            Self::InvalidXmlLang { .. } => {
                "rule.document-structure.xml-lang-must-be-bcp47-or-empty.md"
            }
            Self::InvalidXmlSpace { .. } => {
                "rule.document-structure.xml-space-must-be-enumerated-default-preserve.md"
            }
        }
    }
}

/// One problem found by [`crate::Document::validate`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlValidationError {
    kind: ValidationErrorKind,
    node: Option<NodeId>,
    message: String,
}

impl XmlValidationError {
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

impl fmt::Display for XmlValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.node {
            Some(node) => write!(f, "node {node}: {} [{}]", self.message, self.kind.rule()),
            None => write!(f, "{} [{}]", self.message, self.kind.rule()),
        }
    }
}

impl std::error::Error for XmlValidationError {}

/// Every problem [`crate::Document::validate`] found, in one value.
///
/// The list is deterministic (nodes in arena order, and within one node the checks run in a fixed
/// order), so comparing two validation runs is meaningful.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlValidationErrors {
    errors: Vec<XmlValidationError>,
}

impl XmlValidationErrors {
    /// The issues, in deterministic order.
    pub fn as_slice(&self) -> &[XmlValidationError] {
        &self.errors
    }

    /// Iterates over the issues, in deterministic order.
    pub fn iter(&self) -> std::slice::Iter<'_, XmlValidationError> {
        self.errors.iter()
    }

    /// How many issues were found.
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Whether no issues were found.
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// The issues, as an owning iterator.
    pub fn into_errors(self) -> Vec<XmlValidationError> {
        self.errors
    }
}

impl fmt::Display for XmlValidationErrors {
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

impl std::error::Error for XmlValidationErrors {}

impl From<Vec<XmlValidationError>> for XmlValidationErrors {
    fn from(errors: Vec<XmlValidationError>) -> Self {
        Self { errors }
    }
}

impl<'a> IntoIterator for &'a XmlValidationErrors {
    type Item = &'a XmlValidationError;
    type IntoIter = std::slice::Iter<'a, XmlValidationError>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.iter()
    }
}

/// Runs every whole-document check over `arena` and collects all problems.
///
/// This is the implementation behind [`crate::Document::validate`]; it takes the arena rather than
/// the document so that it cannot acquire the lock again (see the single-lock invariant in
/// `docs/design/PLAN.md` §3.1).
pub(crate) fn validate_arena(arena: &Arena) -> Vec<XmlValidationError> {
    let mut errors = Vec::new();
    check_structure(arena, &mut errors);
    check_nodes(arena, &mut errors);
    check_xml_id_uniqueness(arena, &mut errors);
    errors
}

/// The structural self-checks.
///
/// Only the missing-root check can fire for a document built through this crate's API; the others
/// verify invariants that [`Arena`] maintains on every mutation (see the note on
/// [`ValidationErrorKind`]'s structural variants). They are cheap, they are what the corresponding
/// specification rules are about, and they mean a future refactor cannot break the tree model
/// silently.
fn check_structure(arena: &Arena, errors: &mut Vec<XmlValidationError>) {
    match arena.root() {
        None => errors.push(XmlValidationError::document(
            ValidationErrorKind::MissingRoot,
            "the document has no root element",
        )),
        Some(root) => {
            if !arena.is_element(root) {
                errors.push(XmlValidationError::at(
                    ValidationErrorKind::RootIsNotAnElement,
                    root,
                    "the root of the document is not an element",
                ));
            }
            if arena.parent(root).is_some() {
                errors.push(XmlValidationError::at(
                    ValidationErrorKind::RootHasParent,
                    root,
                    "the root of the document has a parent",
                ));
            }
        }
    }

    for id in arena.ids() {
        let Some(parent) = arena.parent(id) else {
            continue;
        };
        let occurrences = arena
            .children(parent)
            .iter()
            .filter(|&&child| child == id)
            .count();
        if occurrences != 1 {
            errors.push(XmlValidationError::at(
                ValidationErrorKind::ParentChildMismatch,
                id,
                format!(
                    "node {id} claims parent {parent}, but appears {occurrences} times in its child list"
                ),
            ));
        }

        // Walk up, bounded by the number of slots: a cycle would otherwise loop forever.
        let mut current = Some(parent);
        let mut steps = 0usize;
        while let Some(node) = current {
            if node == id {
                errors.push(XmlValidationError::at(
                    ValidationErrorKind::CyclicStructure,
                    id,
                    format!("node {id} is its own ancestor"),
                ));
                break;
            }
            steps += 1;
            if steps > arena.len() {
                errors.push(XmlValidationError::at(
                    ValidationErrorKind::CyclicStructure,
                    id,
                    format!("the parent chain of node {id} does not terminate"),
                ));
                break;
            }
            current = arena.parent(node);
        }
    }
}

/// The per-node checks: names against their scope, and attribute values.
fn check_nodes(arena: &Arena, errors: &mut Vec<XmlValidationError>) {
    for id in arena.ids() {
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
        for (prefix, namespace) in &element.namespace_declarations {
            if let Some(namespace) = namespace
                && let Err(reason) = check_namespace_declaration(prefix.as_ref(), namespace.uri())
            {
                errors.push(XmlValidationError::at(
                    ValidationErrorKind::IllegalNamespaceDeclaration {
                        prefix: prefix.clone(),
                        uri: namespace.uri().to_string(),
                        reason: reason.clone(),
                    },
                    id,
                    format!("illegal namespace declaration: {reason}"),
                ));
            }
        }

        check_xml_attribute_values(id, element, errors);
    }
}

/// The `xml:id`, `xml:lang` and `xml:space` value checks for one element.
///
/// These are node-local rules, so they apply to detached nodes as well: a subtree can be prepared
/// and validated before it is attached.
fn check_xml_attribute_values(
    id: NodeId,
    element: &crate::arena::ElementData,
    errors: &mut Vec<XmlValidationError>,
) {
    if let Some(value) = xml_id_attribute(element)
        && NCName::try_from(value).is_err()
    {
        // rule: rule.attributes.id-must-be-name.md
        errors.push(XmlValidationError::at(
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
        // rule: rule.document-structure.xml-lang-must-be-bcp47-or-empty.md
        errors.push(XmlValidationError::at(
            ValidationErrorKind::InvalidXmlLang {
                value: value.to_string(),
            },
            id,
            format!("`xml:lang` value {value:?} is not a language tag and is not empty"),
        ));
    }
    if let Some(value) = xml_space_attribute(element)
        && !is_valid_xml_space_value(value)
    {
        // rule: rule.document-structure.xml-space-must-be-enumerated-default-preserve.md
        errors.push(XmlValidationError::at(
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
fn check_xml_id_uniqueness(arena: &Arena, errors: &mut Vec<XmlValidationError>) {
    let Some(root) = arena.root() else {
        return;
    };
    // Breadth-first over the attached tree, so detached nodes are not compared: a detached
    // `deep_clone` of a subtree legitimately carries the same `xml:id` values as its original.
    let mut seen: std::collections::BTreeMap<String, NodeId> = std::collections::BTreeMap::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(root);
    let mut visited = 0usize;
    while let Some(id) = queue.pop_front() {
        visited += 1;
        if visited > arena.len() {
            // Structure problems are reported by `check_structure`; do not loop here as well.
            return;
        }
        if let NodeData::Element(element) = arena.data(id)
            && let Some(value) = xml_id_attribute(element)
            && let Ok(name) = NCName::try_from(value)
        {
            // rule: rule.attributes.id-must-be-unique.md
            if let Some(previous) = seen.insert(value.to_string(), id) {
                errors.push(XmlValidationError::at(
                    ValidationErrorKind::DuplicateXmlId { id: name },
                    id,
                    format!("`xml:id` value {value:?} is already used by node {previous}"),
                ));
            }
        }
        for &child in arena.children(id) {
            queue.push_back(child);
        }
    }
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
) -> XmlValidationError {
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
    XmlValidationError::at(kind, node, message)
}
