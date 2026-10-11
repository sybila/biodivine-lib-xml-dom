//! Error types for XML DOM operations.
//!
//! Errors are grouped by *when* they can occur, because that is what a caller needs to branch on:
//!
//! - **construction / specification violations** — raised while validating a single value
//!   (a name, a string, a namespace). These are the *locally* decidable
//!   properties, and they are enforced before anything is stored in a document.
//! - **document well-formedness** — raised while parsing, i.e. properties that are still
//!   decidable from a single element but only become relevant in document context.
//! - **API misuse** — raised by document/tree operations (`append_child`, `replace_with`, …).
//!
//! Whole-document integrity is reported through [`XmlError::Validation`]: [`Document`](crate::Document)
//! collects every problem in one pass, so a caller sees all of them at once instead of one failed
//! operation at a time. Long-running operations can also be interrupted and fail with
//! [`XmlError::Cancelled`].

use crate::node_id::NodeId;
use crate::validation::XmlValidationErrors;
use thiserror::Error;

/// The error type returned by fallible operations of this crate.
#[derive(Error, Debug)]
pub enum XmlError {
    // ---------------------------------------------------------------------------------------
    // Locally decidable specification violations.
    // ---------------------------------------------------------------------------------------
    /// A string is not a legal XML `Name`, `NCName` or `QName`.
    #[error("invalid XML name: {0}")]
    InvalidName(String),
    /// A string contains characters that are not legal XML characters
    /// ([`rule.well-formedness.legal-characters`](crate::xml_spec::rule_set::rule_well_formedness_legal_characters)).
    #[error("invalid XML text: {0}")]
    InvalidText(String),
    /// A comment contains `--` or ends with `-` ([`rule.well-formedness.comment-no-double-hyphen`](crate::xml_spec::rule_set::rule_well_formedness_comment_no_double_hyphen)).
    #[error("invalid XML comment: {0}")]
    InvalidComment(String),
    /// CDATA content contains `]]>`
    /// ([`rule.document-structure.cdata-section-must-not-contain-cdend`](crate::xml_spec::rule_set::rule_document_structure_cdata_section_must_not_contain_cdend)).
    #[error("invalid CDATA section: {0}")]
    InvalidCData(String),
    /// A processing instruction is malformed
    /// ([`rule.well-formedness.pi-target-is-name`](crate::xml_spec::rule_set::rule_well_formedness_pi_target_is_name), [`rule.well-formedness.pi-no-contains-close`](crate::xml_spec::rule_set::rule_well_formedness_pi_no_contains_close)).
    #[error("invalid processing instruction: {0}")]
    InvalidProcessingInstruction(String),
    /// A namespace URI/prefix combination violates the Namespaces specification.
    #[error("invalid namespace: {0}")]
    InvalidNamespace(String),
    /// A reserved prefix (`xml`, `xmlns`) was used in a way the specification forbids.
    #[error("reserved namespace prefix: {0}")]
    ReservedPrefix(String),
    /// Two attributes of one element share the same expanded name
    /// ([`rule.elements-and-tags.unique-attribute-specification`](crate::xml_spec::rule_set::rule_elements_and_tags_unique_attribute_specification)).
    #[error("duplicate attribute `{0}`")]
    DuplicateAttribute(String),

    // ---------------------------------------------------------------------------------------
    // Document well-formedness (raised while parsing).
    // ---------------------------------------------------------------------------------------
    /// A namespace prefix is used but never declared ([`rule.namespace-usage.prefix-declared`](crate::xml_spec::rule_set::rule_namespace_usage_prefix_declared)).
    #[error("undeclared namespace prefix `{0}`")]
    UndeclaredPrefix(String),
    /// The document has no root element ([`rule.well-formedness.document-production`](crate::xml_spec::rule_set::rule_well_formedness_document_production)).
    #[error("the document has no root element")]
    MissingRoot,
    /// The input contains more than one root element
    /// ([`rule.well-formedness.single-root-element`](crate::xml_spec::rule_set::rule_well_formedness_single_root_element)).
    #[error("the document has more than one root element")]
    MultipleRootElements,
    /// Non-whitespace content appeared outside the root element
    /// ([`rule.well-formedness.document-production`](crate::xml_spec::rule_set::rule_well_formedness_document_production)).
    #[error("content is not allowed outside the root element")]
    ContentOutsideRoot,
    /// The declared encoding is not supported (this crate is UTF-8 only).
    #[error("unsupported XML encoding `{0}`: only UTF-8 is supported")]
    UnsupportedEncoding(String),
    /// The declared XML version is not 1.0.
    #[error("unsupported XML version `{0}`: only 1.0 is supported")]
    UnsupportedXmlVersion(String),
    /// A general entity reference that is not one of the five predefined entities
    /// ([`rule.attributes.no-undeclared-entity-refs`](crate::xml_spec::rule_set::rule_attributes_no_undeclared_entity_refs), [`rule.entities.predefined-entities-recognized`](crate::xml_spec::rule_set::rule_entities_predefined_entities_recognized)).
    #[error(
        "undeclared entity reference `&{0};` (only the predefined entities are supported)"
    )]
    UndeclaredEntityReference(String),
    /// A character reference denotes a character that is not legal in XML
    /// ([`rule.entities.charref-legal-character`](crate::xml_spec::rule_set::rule_entities_charref_legal_character)).
    #[error("invalid character reference `{0}`")]
    InvalidCharacterReference(String),
    /// The input is not well-formed XML.
    #[error("malformed XML: {0}")]
    MalformedXml(String),
    /// The input is not valid UTF-8 ([`rule.entities.illegal-byte-sequence-fatal`](crate::xml_spec::rule_set::rule_entities_illegal_byte_sequence_fatal)).
    #[error("invalid UTF-8: {0}")]
    InvalidUtf8(String),

    // ---------------------------------------------------------------------------------------
    // API misuse.
    // ---------------------------------------------------------------------------------------
    /// The node belongs to a different [`crate::Document`] than the target of the operation.
    ///
    /// Attaching nodes across documents is not possible by design; use
    /// [`crate::Node::deep_clone_into`] to copy a subtree into another document instead.
    #[error("the node belongs to a different document")]
    ForeignDocument,
    /// The operation would create a cycle in the document tree.
    #[error("the operation would create a cycle in the document tree")]
    CycleDetected,
    /// The referenced node does not exist in this document.
    #[error("no node with id {0} in this document")]
    NodeNotFound(NodeId),
    /// The operation requires an element, but the node has a different kind.
    #[error("node {0} is not an element")]
    NotAnElement(NodeId),
    /// The node has no parent, so it cannot be replaced or removed from one.
    #[error("node {0} has no parent")]
    NodeHasNoParent(NodeId),
    /// The node is not a child of the given parent.
    #[error("node {0} is not a child of node {1}")]
    NotAChild(NodeId, NodeId),
    /// The insertion index is out of range.
    #[error("child index {index} is out of range for a node with {len} children")]
    IndexOutOfBounds {
        /// The requested index.
        index: usize,
        /// The number of children the parent actually has.
        len: usize,
    },
    /// The new root element is already attached to a parent.
    #[error("the new root element is already attached to a parent")]
    RootHasParent,
    /// The document root cannot become a child of another node.
    #[error("the document root cannot be attached as a child")]
    CannotAttachRoot,

    // ---------------------------------------------------------------------------------------
    // Cancellation and whole-document validation.
    // ---------------------------------------------------------------------------------------
    /// The operation was cancelled by a [`cancel_this`] cancellation trigger.
    ///
    /// The long-running operations of this crate — parsing, serialization, validation,
    /// cross-document copies and explicit garbage collection — periodically check the cancellation
    /// triggers registered on the current thread and return this error instead of running to
    /// completion. Register a trigger around a call with [`cancel_this::on_trigger`] (or one of its
    /// helpers, such as [`cancel_this::on_timeout`], [`cancel_this::on_atomic`] or, for the Python
    /// interpreter, [`cancel_this::on_python`]); with no trigger registered the checks are nearly
    /// free and never fire.
    #[error(transparent)]
    Cancelled(#[from] cancel_this::Cancelled),

    /// Whole-document validation found one or more problems.
    ///
    /// This is what [`crate::Document::validate`] returns; the individual findings are in the
    /// [`XmlValidationErrors`].
    #[error(transparent)]
    Validation(#[from] XmlValidationErrors),

    // ---------------------------------------------------------------------------------------
    // I/O.
    // ---------------------------------------------------------------------------------------
    /// An underlying I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for XML DOM operations.
pub type XmlResult<T> = Result<T, XmlError>;
