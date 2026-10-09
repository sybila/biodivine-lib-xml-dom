//! Error types for XML DOM operations.
//!
//! Errors are grouped by *when* they can occur, because that is what a caller needs to branch on:
//!
//! - **construction / specification violations** — raised while validating a single value
//!   (a name, a string, a namespace). Requirement (4)(1): these are the *locally* decidable
//!   properties, and they are enforced before anything is stored in a document.
//! - **document well-formedness** — raised while parsing, i.e. properties that are still
//!   decidable from a single element but only become relevant in document context.
//! - **API misuse** — raised by document/tree operations (`append_child`, `replace_with`, …).
//!
//! Whole-document integrity (requirement (4)(2)) is *not* reported through this type: it is
//! collected by `Document::validate` so that every issue can be reported at once.

use crate::arena::NodeId;
use thiserror::Error;

/// The error type returned by fallible operations of this crate.
#[derive(Error, Debug)]
pub enum XmlError {
    // ---------------------------------------------------------------------------------------
    // Locally decidable specification violations (requirement (4)(1)).
    // ---------------------------------------------------------------------------------------
    /// A string is not a legal XML `Name`, `NCName` or `QName`.
    #[error("invalid XML name: {0}")]
    InvalidName(String),
    /// A string contains characters that are not legal XML characters
    /// (`rule.well-formedness.legal-characters`).
    #[error("invalid XML text: {0}")]
    InvalidText(String),
    /// A comment contains `--` or ends with `-` (`rule.well-formedness.comment-no-double-hyphen`).
    #[error("invalid XML comment: {0}")]
    InvalidComment(String),
    /// CDATA content contains `]]>`
    /// (`rule.document-structure.cdata-section-must-not-contain-cdend`).
    #[error("invalid CDATA section: {0}")]
    InvalidCData(String),
    /// A processing instruction is malformed
    /// (`rule.well-formedness.pi-target-is-name`, `rule.well-formedness.pi-no-contains-close`).
    #[error("invalid processing instruction: {0}")]
    InvalidProcessingInstruction(String),
    /// A namespace URI/prefix combination violates the Namespaces specification.
    #[error("invalid namespace: {0}")]
    InvalidNamespace(String),
    /// A reserved prefix (`xml`, `xmlns`) was used in a way the specification forbids.
    #[error("reserved namespace prefix: {0}")]
    ReservedPrefix(String),
    /// Two attributes of one element share the same expanded name
    /// (`rule.elements-and-tags.unique-attribute-specification`).
    #[error("duplicate attribute `{0}`")]
    DuplicateAttribute(String),

    // ---------------------------------------------------------------------------------------
    // Document well-formedness (raised while parsing).
    // ---------------------------------------------------------------------------------------
    /// A namespace prefix is used but never declared (`rule.namespace-usage.prefix-declared`).
    #[error("undeclared namespace prefix `{0}`")]
    UndeclaredPrefix(String),
    /// The document has no root element (`rule.well-formedness.document-production`).
    #[error("the document has no root element")]
    MissingRoot,
    /// The input contains more than one root element
    /// (`rule.well-formedness.single-root-element`).
    #[error("the document has more than one root element")]
    MultipleRootElements,
    /// Non-whitespace content appeared outside the root element
    /// (`rule.well-formedness.document-production`).
    #[error("content is not allowed outside the root element")]
    ContentOutsideRoot,
    /// The declared encoding is not supported (this crate is UTF-8 only).
    #[error("unsupported XML encoding `{0}`: only UTF-8 is supported")]
    UnsupportedEncoding(String),
    /// The declared XML version is not 1.0.
    #[error("unsupported XML version `{0}`: only 1.0 is supported")]
    UnsupportedXmlVersion(String),
    /// A general entity reference that is not one of the five predefined entities
    /// (`rule.attributes.no-undeclared-entity-refs`, `rule.entities.predefined-entities-recognized`).
    #[error("undeclared entity reference `&{0};` (only the predefined entities are supported)")]
    UndeclaredEntityReference(String),
    /// A character reference denotes a character that is not legal in XML
    /// (`rule.entities.charref-legal-character`).
    #[error("invalid character reference `{0}`")]
    InvalidCharacterReference(String),
    /// The input is not well-formed XML.
    #[error("malformed XML: {0}")]
    MalformedXml(String),
    /// The input is not valid UTF-8 (`rule.entities.illegal-byte-sequence-fatal`).
    #[error("invalid UTF-8: {0}")]
    InvalidUtf8(String),

    // ---------------------------------------------------------------------------------------
    // API misuse.
    // ---------------------------------------------------------------------------------------
    /// The node belongs to a different [`crate::Document`] than the target of the operation.
    ///
    /// Attaching nodes across documents is not possible by design (requirement (2)); use
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
    // I/O.
    // ---------------------------------------------------------------------------------------
    /// An underlying I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for XML DOM operations.
pub type XmlResult<T> = Result<T, XmlError>;
