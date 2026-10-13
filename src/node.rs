//! Public node handles.
//!
//! A [`Node`] is an id into a document's arena, a counted external reference that keeps that slot
//! alive, and a clone of the [`Document`] handle. All data access goes through the document's
//! single lock, which is acquired and released inside each method; only cloning and dropping the
//! handle touch state outside the lock, and that state is a single atomic counter.
//!
//! # Cloning
//!
//! `Clone` copies the *handle*, not the node: the clone refers to the same arena slot and is
//! therefore [`Node::ptr_eq`]-equal to the original. It also keeps the node alive, so a handle is an
//! owning reference. The two operations that duplicate data are [`Node::shallow_clone`] (same
//! payload, no children) and [`Node::deep_clone`] (whole subtree) — plus their `_into` variants,
//! which copy the subtree into another document.

use std::fmt;
use std::hash::{Hash, Hasher};

use crate::cancellation::CancellationCheck;
use crate::document::Document;
use crate::element::Element;
use crate::error::{XmlError, XmlResult};
use crate::node_id::NodeId;
use crate::ref_count::NodeRef;
use crate::xml_spec::{CData, Comment, PiData, PiTarget, Text};

/// The kind of a node, without its payload.
///
/// This is the cheap discriminant counterpart of [`NodeContent`]: it is `Copy` and does not clone
/// anything.
///
/// Unlike the error and option types, this enum is exhaustive: the node kinds are fixed by the XML
/// data model, so a `match` over them needs no wildcard arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NodeKind {
    /// An element node.
    Element,
    /// A text node.
    Text,
    /// A comment node.
    Comment,
    /// A CDATA section.
    CData,
    /// A processing instruction.
    ProcessingInstruction,
}

/// The payload of a node, obtained with [`Node::content`].
///
/// Exhaustive for the same reason as [`NodeKind`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeContent {
    /// An element node, as a strongly typed [`Element`] handle.
    Element(Element),
    /// A text node.
    Text(Text),
    /// A comment node.
    Comment(Comment),
    /// A CDATA section.
    CData(CData),
    /// A processing instruction (target, content).
    ProcessingInstruction(PiTarget, PiData),
}

/// A handle to one node of a [`Document`].
///
/// The handle is `Send + Sync`, cheap to clone, and valid for as long as it exists: it counts as an
/// external reference to the node, so the node is never reclaimed while the handle is alive, not even
/// if it is detached from the tree or its whole subtree is discarded elsewhere.
///
/// A handle also keeps the ancestors of its node reachable, because it can walk upwards
/// ([`Node::parent`], [`Node::is_attached`], namespace resolution).
///
/// # Examples
///
/// ```rust
/// use biodivine_lib_xml_dom::{Document, QualifiedName};
///
/// let document = Document::new();
/// let parent = document.create_element(QualifiedName::without_namespace("parent").unwrap());
/// let child = document.create_element(QualifiedName::without_namespace("child").unwrap());
/// document.set_root(parent.clone());
///
/// parent.append_child(child.clone());
/// assert_eq!(parent.children().len(), 1);
/// assert!(child.is_attached());
/// ```
pub struct Node {
    /// The document this handle refers to.
    pub(crate) document: Document,
    /// The arena slot of this node.
    pub(crate) id: NodeId,
    /// The counted external reference that keeps the node alive.
    external: NodeRef,
}

impl Clone for Node {
    /// Clones the *handle*, not the node: the clone refers to the same arena slot and is therefore
    /// [`Node::ptr_eq`]-equal to the original.
    ///
    /// Cloning only increments an atomic counter; it never takes the arena lock. The node is kept
    /// alive by the clone just like by the original, which is why a handle is a cheap but *owning*
    /// reference to a node.
    fn clone(&self) -> Self {
        Self {
            document: self.document.clone(),
            id: self.id,
            external: self.external.clone(),
        }
    }
}

impl Node {
    /// Wraps a counted external reference that the arena handed out for `id`.
    ///
    /// This is deliberately the only constructor and it is crate-private: an external reference can
    /// only come from [`Document`]'s arena, which is what keeps the collector's root set complete.
    pub(crate) fn from_ref(document: Document, id: NodeId, external: NodeRef) -> Self {
        Self {
            document,
            id,
            external,
        }
    }

    /// The document this node belongs to.
    pub fn document(&self) -> Document {
        self.document.clone()
    }

    /// The arena id of this node.
    ///
    /// An id is only meaningful together with the document that produced it, and only while the node
    /// is alive: once the node is unreachable and reclaimed, the slot can hold a different node. Use
    /// [`Node::belongs_to`] or [`Node::ptr_eq`] to compare handles instead of raw ids, and
    /// [`Document::node`] to resolve an id back to a handle.
    pub fn id(&self) -> NodeId {
        self.id
    }

    /// Whether two handles refer to the same node of the same document.
    pub fn ptr_eq(&self, other: &Node) -> bool {
        self.document.ptr_eq(&other.document) && self.id == other.id
    }

    /// Whether this node belongs to `document`.
    pub fn belongs_to(&self, document: &Document) -> bool {
        self.document.ptr_eq(document)
    }

    /// The kind of this node.
    pub fn kind(&self) -> NodeKind {
        self.document
            .read_arena("Node::kind", |arena| match arena.data(self.id) {
                crate::arena::NodeData::Element(_) => NodeKind::Element,
                crate::arena::NodeData::Text(_) => NodeKind::Text,
                crate::arena::NodeData::Comment(_) => NodeKind::Comment,
                crate::arena::NodeData::CData(_) => NodeKind::CData,
                crate::arena::NodeData::ProcessingInstruction { .. } => {
                    NodeKind::ProcessingInstruction
                }
            })
    }

    /// The payload of this node.
    pub fn content(&self) -> NodeContent {
        use crate::arena::NodeData;
        let id = self.id;
        self.document
            .read_arena("Node::content", |arena| match arena.data(id) {
                NodeData::Element(_) => {
                    // Cloning the handle creates the counted reference for the element; it needs no
                    // arena lookup, so no reference is created outside the lock either.
                    NodeContent::Element(Element::new_unchecked(self.clone()))
                }
                NodeData::Text(text) => NodeContent::Text(text.clone()),
                NodeData::Comment(comment) => NodeContent::Comment(comment.clone()),
                NodeData::CData(cdata) => NodeContent::CData(cdata.clone()),
                NodeData::ProcessingInstruction { target, data } => {
                    NodeContent::ProcessingInstruction(target.clone(), data.clone())
                }
            })
    }

    /// This node as an [`Element`], or `None` if it has a different kind.
    pub fn as_element(&self) -> Option<Element> {
        match self.kind() {
            NodeKind::Element => Some(Element::new_unchecked(self.clone())),
            _ => None,
        }
    }

    /// The text content of this node, or `None` if it is not a text node.
    pub fn text(&self) -> Option<Text> {
        match self.content() {
            NodeContent::Text(text) => Some(text),
            _ => None,
        }
    }

    /// The content of this node as a comment, or `None` if it is not a comment.
    pub fn comment(&self) -> Option<Comment> {
        match self.content() {
            NodeContent::Comment(comment) => Some(comment),
            _ => None,
        }
    }

    /// The content of this node as CDATA, or `None` if it is not a CDATA section.
    pub fn cdata(&self) -> Option<CData> {
        match self.content() {
            NodeContent::CData(cdata) => Some(cdata),
            _ => None,
        }
    }

    /// The target and content of this node as a processing instruction, or `None` if it is not
    /// one.
    pub fn processing_instruction(&self) -> Option<(PiTarget, PiData)> {
        match self.content() {
            NodeContent::ProcessingInstruction(target, data) => Some((target, data)),
            _ => None,
        }
    }

    // -------------------------------------------------------------------------------------
    // Tree queries
    // -------------------------------------------------------------------------------------

    /// The parent of this node, or `None` if it is detached or is the document root.
    pub fn parent(&self) -> Option<Node> {
        self.document
            .read_one("Node::parent", |arena| arena.parent(self.id))
    }

    /// Whether this node is reachable from the document root.
    pub fn is_attached(&self) -> bool {
        self.document
            .read_arena("Node::is_attached", |arena| arena.is_attached(self.id))
    }

    /// Whether this node is a strict ancestor of `other`.
    ///
    /// Always `false` if the two nodes belong to different documents.
    pub fn is_ancestor_of(&self, other: &Node) -> bool {
        if !self.document.ptr_eq(&other.document) {
            return false;
        }
        self.document.read_arena("Node::is_ancestor_of", |arena| {
            arena.is_ancestor(self.id, other.id)
        })
    }

    /// The children of this node in document order (empty for non-elements).
    pub fn children(&self) -> Vec<Node> {
        self.document
            .read_many("Node::children", |arena| arena.children(self.id).to_vec())
    }

    /// The element children of this node in document order.
    pub fn child_elements(&self) -> Vec<Element> {
        self.children()
            .into_iter()
            .filter_map(|node| node.as_element())
            .collect()
    }

    /// The first child of this node, if any.
    pub fn first_child(&self) -> Option<Node> {
        self.document.read_one("Node::first_child", |arena| {
            arena.children(self.id).first().copied()
        })
    }

    /// The last child of this node, if any.
    pub fn last_child(&self) -> Option<Node> {
        self.document.read_one("Node::last_child", |arena| {
            arena.children(self.id).last().copied()
        })
    }

    /// The next sibling of this node, if any.
    pub fn next_sibling(&self) -> Option<Node> {
        self.sibling(1)
    }

    /// The previous sibling of this node, if any.
    pub fn previous_sibling(&self) -> Option<Node> {
        self.sibling(-1)
    }

    /// The sibling `offset` positions away, if any.
    ///
    /// The search is bounded by the length of the parent's child list, so it terminates even for
    /// a malformed document.
    fn sibling(&self, offset: isize) -> Option<Node> {
        self.document.read_one("Node::sibling", |arena| {
            let parent = arena.parent(self.id)?;
            let children = arena.children(parent);
            let position = children.iter().position(|&child| child == self.id)?;
            if offset.is_negative() {
                children.get(position.checked_sub(offset.unsigned_abs())?)
            } else {
                children.get(position.checked_add(offset.unsigned_abs())?)
            }
            .copied()
        })
    }

    /// The index of this node in its parent's child list, if it has a parent.
    pub fn index_in_parent(&self) -> Option<usize> {
        self.document.read_arena("Node::index_in_parent", |arena| {
            let parent = arena.parent(self.id)?;
            arena
                .children(parent)
                .iter()
                .position(|&child| child == self.id)
        })
    }

    /// All descendants of this node in pre-order (depth first, document order).
    ///
    /// The traversal is iterative, so it cannot overflow the stack, and it is bounded by the
    /// number of slots in the document.
    pub fn descendants(&self) -> Vec<Node> {
        self.document.read_many("Node::descendants", |arena| {
            let mut result = Vec::new();
            let mut stack: Vec<NodeId> =
                arena.children(self.id).iter().rev().copied().collect();
            while let Some(current) = stack.pop() {
                result.push(current);
                for &child in arena.children(current).iter().rev() {
                    stack.push(child);
                }
                if result.len() > arena.live() {
                    // Cannot happen for an acyclic tree; the bound keeps a hypothetical
                    // inconsistency from turning into an infinite loop.
                    break;
                }
            }
            result
        })
    }

    // -------------------------------------------------------------------------------------
    // Structural editing
    // -------------------------------------------------------------------------------------

    /// Detaches this node from its parent and returns the previous parent.
    ///
    /// The node itself stays in the arena in a fully usable state: it can be inspected, edited,
    /// cloned and attached again. Detaching a node that is already detached does nothing and
    /// returns `None`.
    pub fn detach(&self) -> Option<Node> {
        self.document
            .write_one("Node::detach", |arena| arena.detach(self.id))
    }

    /// Appends `child` as the last child of this node.
    ///
    /// If `child` is already attached somewhere, it is *moved*: it is detached from its previous
    /// parent first. No namespace declarations are added, removed or rewritten by this operation.
    /// Editing does not repair namespaces: [`Document::validate`] reports the resulting
    /// inconsistency.
    ///
    /// # Panics
    ///
    /// Panics if `child` belongs to another document ([`XmlError::ForeignDocument`]), if `child`
    /// is `self` or an ancestor of `self` ([`XmlError::CycleDetected`]), if `child` is the
    /// document root ([`XmlError::CannotAttachRoot`]), or if `self` is not an element
    /// ([`XmlError::NotAnElement`]). The document is left unchanged in every one of these cases;
    /// use [`Node::append_child_checked`] to handle them without panicking.
    #[track_caller]
    pub fn append_child(&self, child: impl Into<Node>) {
        match self.append_child_checked(child) {
            Ok(()) => {}
            Err(error) => panic!("Node::append_child failed: {error}"),
        }
    }

    /// Fallible variant of [`Node::append_child`].
    ///
    /// # Errors
    ///
    /// - [`XmlError::ForeignDocument`] if `child` belongs to another document.
    /// - [`XmlError::CycleDetected`] if `child` is `self` or an ancestor of `self`.
    /// - [`XmlError::CannotAttachRoot`] if `child` is the document root.
    /// - [`XmlError::NotAnElement`] if `self` is not an element.
    ///
    /// The document is never modified when an error is returned.
    pub fn append_child_checked(&self, child: impl Into<Node>) -> XmlResult<()> {
        let child = child.into();
        self.check_attachable(&child)?;
        self.document
            .write_arena("Node::append_child_checked", |arena| {
                arena.append(self.id, child.id)
            })
    }

    /// Inserts `child` as the `index`-th child of this node.
    ///
    /// `index` is interpreted *after* `child` has been detached from any previous parent, so
    /// "move this node to position `i`" always means the same thing.
    ///
    /// # Panics
    ///
    /// As [`Node::append_child`], plus [`XmlError::IndexOutOfBounds`] if `index` is greater than
    /// the resulting number of children.
    #[track_caller]
    pub fn insert_child(&self, index: usize, child: impl Into<Node>) {
        match self.insert_child_checked(index, child) {
            Ok(()) => {}
            Err(error) => panic!("Node::insert_child failed: {error}"),
        }
    }

    /// Fallible variant of [`Node::insert_child`].
    ///
    /// # Errors
    ///
    /// As [`Node::append_child_checked`], plus [`XmlError::IndexOutOfBounds`].
    pub fn insert_child_checked(
        &self,
        index: usize,
        child: impl Into<Node>,
    ) -> XmlResult<()> {
        let child = child.into();
        self.check_attachable(&child)?;
        self.document
            .write_arena("Node::insert_child_checked", |arena| {
                arena.attach(self.id, index, child.id)
            })
    }

    /// Inserts `child` directly before the child `sibling` of this node.
    ///
    /// # Panics
    ///
    /// As [`Node::append_child`], plus if `sibling` is not a child of `self`
    /// ([`XmlError::NotAChild`]).
    #[track_caller]
    pub fn insert_before(&self, sibling: impl Into<Node>, child: impl Into<Node>) {
        match self.insert_before_checked(sibling, child) {
            Ok(()) => {}
            Err(error) => panic!("Node::insert_before failed: {error}"),
        }
    }

    /// Fallible variant of [`Node::insert_before`].
    ///
    /// # Errors
    ///
    /// As [`Node::append_child_checked`], plus [`XmlError::NotAChild`] if `sibling` is not a child
    /// of `self`. Inserting a node before itself is a no-op.
    pub fn insert_before_checked(
        &self,
        sibling: impl Into<Node>,
        child: impl Into<Node>,
    ) -> XmlResult<()> {
        self.insert_relative_checked(sibling.into(), child.into(), false)
    }

    /// Inserts `child` directly after the child `sibling` of this node.
    ///
    /// # Panics
    ///
    /// As [`Node::insert_before`].
    #[track_caller]
    pub fn insert_after(&self, sibling: impl Into<Node>, child: impl Into<Node>) {
        match self.insert_after_checked(sibling, child) {
            Ok(()) => {}
            Err(error) => panic!("Node::insert_after failed: {error}"),
        }
    }

    /// Fallible variant of [`Node::insert_after`].
    ///
    /// # Errors
    ///
    /// As [`Node::insert_before_checked`].
    pub fn insert_after_checked(
        &self,
        sibling: impl Into<Node>,
        child: impl Into<Node>,
    ) -> XmlResult<()> {
        self.insert_relative_checked(sibling.into(), child.into(), true)
    }

    /// Shared implementation of the two relative insertions.
    fn insert_relative_checked(
        &self,
        sibling: Node,
        child: Node,
        after: bool,
    ) -> XmlResult<()> {
        self.check_attachable(&child)?;
        if !sibling.belongs_to(&self.document) {
            return Err(XmlError::ForeignDocument);
        }
        self.document
            .write_arena("Node::insert_relative_checked", |arena| {
                arena.attach_relative(self.id, sibling.id, child.id, after)
            })
    }

    /// Replaces this node with `replacement` in this node's parent and returns this node.
    ///
    /// The returned value is the *old* node (this handle), which ends up detached: the
    /// replacement takes its place in the parent's child list.
    ///
    /// # Panics
    ///
    /// Panics if this node has no parent ([`XmlError::NodeHasNoParent`]), if `replacement`
    /// belongs to another document ([`XmlError::ForeignDocument`]), if `replacement` is the
    /// document root ([`XmlError::CannotAttachRoot`]), or if the replacement would create a cycle
    /// ([`XmlError::CycleDetected`]).
    ///
    /// Replacing a node by itself is a documented no-op: it does not panic, leaves the document
    /// unchanged and returns this node (still attached).
    #[track_caller]
    pub fn replace_with(&self, replacement: impl Into<Node>) -> Node {
        match self.replace_with_checked(replacement) {
            Ok(()) => self.clone(),
            Err(error) => panic!("Node::replace_with failed: {error}"),
        }
    }

    /// Fallible variant of [`Node::replace_with`].
    ///
    /// # Errors
    ///
    /// - [`XmlError::ForeignDocument`] if `replacement` belongs to another document.
    /// - [`XmlError::NodeHasNoParent`] if this node is detached, so there is nothing to replace it
    ///   in.
    /// - [`XmlError::CycleDetected`] if `replacement` is an ancestor of the parent, or an
    ///   ancestor of this node.
    /// - [`XmlError::CannotAttachRoot`] if `replacement` is the document root.
    ///
    /// Replacing a node by itself is a **no-op that reports success**
    /// ([`Ok`](XmlResult::Ok)), whether or not the node is attached; the document is left
    /// unchanged. This mirrors [`Node::insert_before`]/[`Node::insert_after`], which are also
    /// no-ops when the sibling and the child are the same node. The document is never modified
    /// when an error is returned.
    pub fn replace_with_checked(&self, replacement: impl Into<Node>) -> XmlResult<()> {
        let replacement = replacement.into();
        if !replacement.belongs_to(&self.document) {
            return Err(XmlError::ForeignDocument);
        }
        self.document
            .write_arena("Node::replace_with_checked", |arena| {
                arena.replace(self.id, replacement.id)
            })
    }

    /// The common "can `child` become a child of `self`" precondition.
    fn check_attachable(&self, child: &Node) -> XmlResult<()> {
        if !child.belongs_to(&self.document) {
            Err(XmlError::ForeignDocument)
        } else {
            Ok(())
        }
    }

    // -------------------------------------------------------------------------------------
    // Cloning
    // -------------------------------------------------------------------------------------

    /// Creates a detached, shallow copy of this node in the same document.
    ///
    /// The copy has the same name/attributes/namespace declarations (elements) or the same
    /// content (other kinds), but no children. The payload is shared rather than copied, so this
    /// is cheap, and it is never cancelled.
    ///
    /// # Panics
    ///
    /// Panics if the copy would take its document past [`MAX_NODES`](crate::MAX_NODES) nodes.
    pub fn shallow_clone(&self) -> Node {
        self.copy_within(false)
            .expect("a shallow copy is never cancelled")
    }

    /// Creates a detached, deep copy of this subtree in the same document.
    ///
    /// # Errors
    ///
    /// Returns [`XmlError::Cancelled`] if a cancellation trigger registered on this thread fires
    /// while the subtree is being copied. A cancelled copy creates nothing.
    ///
    /// # Panics
    ///
    /// Panics if the copy would take its document past [`MAX_NODES`](crate::MAX_NODES) nodes.
    pub fn deep_clone(&self) -> XmlResult<Node> {
        self.copy_within(true)
    }

    /// Creates a detached, shallow copy of this node in `target`.
    ///
    /// Target and source may be the same document, in which case this behaves exactly like
    /// [`Node::shallow_clone`]. No namespace declarations are added or removed.
    ///
    /// # Panics
    ///
    /// Panics if the copy would take its document past [`MAX_NODES`](crate::MAX_NODES) nodes.
    pub fn shallow_clone_into(&self, target: &Document) -> Node {
        self.copy_into(target, false)
            .expect("a shallow copy is never cancelled")
    }

    /// Creates a detached, deep copy of this subtree in `target`.
    ///
    /// This is the sanctioned way to move a tree between documents: attaching a node to a parent
    /// of another document is an error ([`XmlError::ForeignDocument`]), whereas copying is always
    /// possible.
    ///
    /// Target and source may be the same document, in which case this behaves exactly like
    /// [`Node::deep_clone`].
    ///
    /// # Consistency
    ///
    /// The copy is a *point-in-time* snapshot of the source. The source document's lock is taken
    /// only for the duration of the snapshot and is released before the target is modified, so
    /// (a) two document locks are never held at the same time and the operation can never fail
    /// because of lock contention, and (b) concurrent mutations of the source after the snapshot
    /// has been taken are simply not part of the copy — they can never corrupt the target.
    ///
    /// Names and namespaces of the copy are interned into `target`, so the copy is independent of
    /// the source document; only immutable payloads (names, text) may still share an allocation
    /// with it.
    ///
    /// # Errors
    ///
    /// Returns [`XmlError::Cancelled`] if a cancellation trigger registered on this thread fires
    /// while the subtree is being copied. A cancelled copy creates nothing in `target`.
    ///
    /// # Panics
    ///
    /// Panics if the copy would take its document past [`MAX_NODES`](crate::MAX_NODES) nodes.
    pub fn deep_clone_into(&self, target: &Document) -> XmlResult<Node> {
        self.copy_into(target, true)
    }

    /// Shared implementation of the same-document clones.
    fn copy_within(&self, with_children: bool) -> XmlResult<Node> {
        self.document
            .write_one_result("Node::copy_within", |arena| {
                arena.copy_within(self.id, with_children)
            })
    }

    /// Shared implementation of the cross-document clones.
    fn copy_into(&self, target: &Document, with_children: bool) -> XmlResult<Node> {
        if target.ptr_eq(&self.document) {
            return self.copy_within(with_children);
        }
        // Snapshot under the source lock ...
        let snapshot = self
            .document
            .read_arena("Node::copy_into (snapshot)", |arena| {
                arena.snapshot(self.id, with_children)
            })?;
        // ... release it, and only then take the target lock.
        let mut check = if with_children {
            CancellationCheck::new()
        } else {
            CancellationCheck::disabled()
        };
        target.write_one_result("Node::copy_into (insert)", |arena| {
            arena.insert_snapshot(snapshot, &mut check)
        })
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Node({:?}, id={}, attached={})",
            self.kind(),
            self.id,
            self.is_attached()
        )
    }
}

impl fmt::Display for Node {
    /// Renders this node as an XML fragment, exactly as the serializer writes it inside a
    /// document: elements with their subtree, text escaped, and comments, CDATA sections and
    /// processing instructions with their delimiters. Rendering is never cancelled.
    ///
    /// Use [`Node::text`] for the unescaped content of a text node.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::io::write_node_to_string(self))
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.ptr_eq(other)
    }
}

impl Eq for Node {}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (std::sync::Arc::as_ptr(&self.document.inner) as usize).hash(state);
        self.id.hash(state);
    }
}

impl From<&Node> for Node {
    fn from(node: &Node) -> Self {
        node.clone()
    }
}

impl From<&Element> for Node {
    fn from(element: &Element) -> Self {
        element.node()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qualified_name::QualifiedName;
    use crate::xml_spec::nc_name;

    /// Names and namespaces keep their shared `Arc` payload when they are copied. A subtree copied into another
    /// document is re-interned there, so the target document ends up with exactly one allocation
    /// per distinct name/namespace.
    ///
    /// Sharing the immutable payload with the *source* is intentional: `QualifiedName` and
    /// `Namespace` are immutable values that carry no document identity (a URI and a prefix, plus
    /// a local name), so a shared allocation cannot expose or retain the source document's state.
    /// What must not happen is the copy dragging the source *document* along, and that is what the
    /// second half of this test checks: the source is dropped before the copy is used.
    #[test]
    fn re_interns_names_in_the_target_document() {
        let target = Document::new();

        let (copy, source_name, source_namespace) = {
            let source = Document::new();
            let ex = crate::Namespace::prefixed("http://example.com", "ex").unwrap();
            let name = QualifiedName::with_namespace("item", &ex).unwrap();

            let root = source.create_element(name.clone());
            root.declare_namespace(ex.clone());
            let copy = root.deep_clone_into(&target).unwrap();
            (copy, name, ex)
        };

        // The source document is gone, but the copy is fully usable.
        let copy_name = copy.qualified_name();
        assert_eq!(copy_name.local_name(), "item");
        assert!(target.nodes().iter().any(|node| node.ptr_eq(&copy.node())));

        // Canonical values of the target document.
        let interned_name = target.create_element(source_name.clone()).qualified_name();
        let interned_namespace = target
            .create_element(
                QualifiedName::with_namespace("other", &source_namespace).unwrap(),
            )
            .qualified_name()
            .namespace()
            .cloned()
            .unwrap();

        assert!(
            copy_name.shares_data_with(&interned_name),
            "the copy's name is not the target document's canonical allocation"
        );
        let declarations = copy.namespace_declarations();
        let declared = declarations
            .get(&Some(nc_name("ex")))
            .cloned()
            .flatten()
            .expect("the declaration must survive the copy");
        assert!(
            declared.shares_data_with(&interned_namespace),
            "the copy's namespace is not the target document's canonical allocation"
        );

        // Immutable payload sharing across documents is intentional and observable.
        assert!(copy_name.shares_data_with(&source_name));
        assert!(declared.shares_data_with(&source_namespace));
    }

    /// Creating many elements with the same name must not allocate a new name each time: the
    /// document's interner keeps exactly one canonical allocation per distinct value.
    #[test]
    fn repeated_names_share_one_allocation() {
        let document = Document::new();
        let first =
            document.create_element(QualifiedName::without_namespace("item").unwrap());
        for _ in 0..1000 {
            let other = document
                .create_element(QualifiedName::without_namespace("item").unwrap());
            assert!(
                first
                    .qualified_name()
                    .shares_data_with(&other.qualified_name())
            );
        }
        assert_eq!(document.node_count(), 1001);
    }

    /// Interned values are value-equal, so nothing in the API depends on which allocation won.
    #[test]
    fn interning_does_not_change_equality() {
        let document = Document::new();
        let first =
            document.create_element(QualifiedName::without_namespace("x").unwrap());
        let second =
            document.create_element(QualifiedName::without_namespace("x").unwrap());
        assert!(
            first
                .qualified_name()
                .shares_data_with(&second.qualified_name())
        );
        assert_eq!(first.qualified_name(), second.qualified_name());
    }
}
