//! The storage of one arena slot and the payloads it can hold.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::namespace::Namespace;
use crate::node_id::NodeId;
use crate::qualified_name::QualifiedName;
use crate::ref_count::ExternalRefCount;
use crate::xml_spec::{CData, Comment, NCName, PiData, PiTarget, Text};

/// One slot of the arena.
///
/// A slot is either free (it can be handed out again by the next allocation, see
/// [`Arena::alloc`](crate::arena::Arena::alloc)) or occupied by a node. Keeping the two states in
/// one enum makes "read the payload of a free slot" impossible to express.
#[derive(Debug)]
pub(crate) enum Slot {
    /// The slot holds no node: it is either not used yet or was reclaimed by a collection.
    Free,
    /// The slot holds a node.
    Occupied(NodeSlot),
}

impl Slot {
    /// The occupied slot, or `None` if the slot is free.
    pub(crate) fn occupied(&self) -> Option<&NodeSlot> {
        match self {
            Self::Free => None,
            Self::Occupied(slot) => Some(slot),
        }
    }

    /// The occupied slot, or `None` if the slot is free.
    pub(crate) fn occupied_mut(&mut self) -> Option<&mut NodeSlot> {
        match self {
            Self::Free => None,
            Self::Occupied(slot) => Some(slot),
        }
    }
}

/// One occupied slot: the payload plus the link to the parent and the external reference count.
#[derive(Debug)]
pub(crate) struct NodeSlot {
    /// The parent node, or `None` if the node is detached or is the document root.
    pub(crate) parent: Option<NodeId>,
    /// The node payload.
    pub(crate) data: NodeData,
    /// The number of live external references to this slot; see [`ExternalRefCount`].
    pub(crate) external: Arc<ExternalRefCount>,
}

/// The payload of a node.
#[derive(Debug)]
pub(crate) enum NodeData {
    /// An element: a name, attributes, namespace declarations and children.
    Element(ElementData),
    /// A text node. The payload is validated at construction
    /// ([`rule.well-formedness.legal-characters`](crate::xml_spec::rule_set::rule_well_formedness_legal_characters)).
    Text(Text),
    /// A comment node ([`rule.well-formedness.comment-no-double-hyphen`](crate::xml_spec::rule_set::rule_well_formedness_comment_no_double_hyphen)).
    Comment(Comment),
    /// A CDATA section ([`rule.document-structure.cdata-section-must-not-contain-cdend`](crate::xml_spec::rule_set::rule_document_structure_cdata_section_must_not_contain_cdend)).
    CData(CData),
    /// A processing instruction.
    ProcessingInstruction {
        /// The PI target ([`rule.well-formedness.pi-target-is-name`](crate::xml_spec::rule_set::rule_well_formedness_pi_target_is_name)).
        target: PiTarget,
        /// The PI content ([`rule.well-formedness.pi-no-contains-close`](crate::xml_spec::rule_set::rule_well_formedness_pi_no_contains_close)).
        data: PiData,
    },
}

/// The payload of an element node.
#[derive(Debug)]
pub(crate) struct ElementData {
    /// The element's expanded name (local name plus optional namespace).
    pub(crate) name: QualifiedName,
    /// Attributes, keyed by expanded name.
    ///
    /// A map keyed by [`QualifiedName`] (whose equality compares the local name and the namespace
    /// *URI*) makes two attributes with the same expanded name unrepresentable, and
    /// [`rule.elements-and-tags.unique-attribute-specification`](crate::xml_spec::rule_set::rule_elements_and_tags_unique_attribute_specification) at the storage level. Setting an
    /// attribute that is already present therefore *overwrites* the old value.
    pub(crate) attributes: BTreeMap<QualifiedName, Arc<str>>,
    /// Namespace declarations written on *this* element.
    ///
    /// The key is the prefix (`None` for the default namespace) and the value is the binding.
    /// `Some(None)` is an empty declaration (`xmlns=""`), which removes the default namespace
    /// from scope ([`rule.namespace-usage.empty-default-namespace`](crate::xml_spec::rule_set::rule_namespace_usage_empty_default_namespace)).
    pub(crate) namespace_declarations: BTreeMap<Option<NCName>, Option<Namespace>>,
    /// The children, in document order.
    pub(crate) children: Vec<NodeId>,
}
