//! Wiring between the arena and the outside world: type [`Document`], the single document lock,
//! and the debug-only re-entrancy detector.
//!
//! See [`documentation::architecture`](crate::documentation::architecture) for the reasoning, and
//! the summary below:
//!
//! * **Locking invariant L**: a document owns exactly one lock ([`parking_lot::RwLock`] around the
//!   [`Arena`]). It is acquired only in the outermost frame of a public `Document`/`Node`/`Element`
//!   method, is held for the duration of that method, and is never acquired a second time while
//!   held. Internal helpers take `&Arena`/`&mut Arena` and therefore cannot lock at all.
//! * Because there is only one lock per document and a re-entrant acquisition is impossible by
//!   construction, there is no lock to deadlock on: no ordering, no upgrade, no nesting.
//! * Cross-document operations (subtree copies) never hold two locks: they snapshot the source
//!   under a read lock, release it, and insert into the target under a write lock.
//!
//! The debug-only [`reentry`] guard turns the "no re-entrancy" claim into something that fails
//! loudly in tests instead of silently hanging.
//!
//! The parsing and serialization entry points live in a child module, because they are a
//! self-contained slice of the public API of [`Document`] with a substantial contract of their own.

mod io;

use std::fmt;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::arena::{Arena, ElementData, GcConfig, GcStats, NodeData};
use crate::element::Element;
use crate::error::{XmlError, XmlResult};
use crate::node::Node;
use crate::node_id::NodeId;
use crate::qualified_name::QualifiedName;
use crate::xml_spec::{CData, Comment, PiData, PiTarget, Text, XmlDeclaration};

/// Debug-only detector for re-entrant access to one document's lock.
///
/// The guard is keyed by *document identity*, not by a plain per-thread depth: re-acquiring the
/// same document's lock is the actual hazard (it hangs, because `parking_lot` guards are not
/// re-entrant), while acquiring a *different* document's lock is a different concern (lock
/// ordering) that this crate avoids by construction rather than by this guard — every
/// cross-document operation snapshots the source and releases it before touching the target.
///
/// In release builds the guard compiles away.
#[cfg(debug_assertions)]
mod reentry {
    use std::cell::RefCell;

    thread_local! {
        /// Identities of the documents whose lock this thread currently holds.
        static HELD: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
    }

    /// Tracks that one document lock is held by the current thread.
    pub(super) struct Guard(usize);

    impl Guard {
        /// Enters the critical section of the document identified by `key`.
        ///
        /// # Panics
        ///
        /// Panics if this thread already holds the lock of the very same document, which would
        /// deadlock.
        #[track_caller]
        pub(super) fn enter(key: usize, operation: &'static str) -> Self {
            HELD.with(|held| {
                let mut held = held.borrow_mut();
                assert!(
                    !held.contains(&key),
                    "re-entrant access to the lock of one document in `{operation}`: the \
                     single-lock invariant is violated and this call would deadlock"
                );
                held.push(key);
            });
            Guard(key)
        }
    }

    impl Drop for Guard {
        fn drop(&mut self) {
            HELD.with(|held| {
                let mut held = held.borrow_mut();
                if let Some(position) = held.iter().rposition(|&key| key == self.0) {
                    held.remove(position);
                }
            });
        }
    }
}

/// Release-build counterpart of the re-entrancy guard: a zero-sized type that does nothing.
#[cfg(not(debug_assertions))]
mod reentry {
    /// No-op placeholder so that the critical sections are identical in both profiles.
    pub(super) struct Guard;

    impl Guard {
        /// Does nothing.
        #[inline(always)]
        pub(super) fn enter(_key: usize, _operation: &'static str) -> Self {
            Guard
        }
    }
}

/// The shared, synchronised state of one XML document.
#[derive(Debug)]
pub(crate) struct DocumentInner {
    /// The single lock of this document; see the module documentation.
    arena: RwLock<Arena>,
}

/// An XML document.
///
/// A [`Document`] is a cheap, `Send + Sync` handle to shared state: cloning it or passing it to
/// another thread does *not* copy the tree. All node payload of one document lives in a single
/// arena behind a single lock, and [`Node`]/[`Element`] handles are nothing but a counted reference
/// to one arena slot plus a clone of this handle.
///
/// # Identity
///
/// Two [`Document`] objects are equal if and only if they refer to the same underlying document
/// (pointer equality). This is what makes a [`Node`] comparable across threads and what lets the
/// API reject cross-document operations.
///
/// # Lifetime of nodes and garbage collection
///
/// A node lives for as long as *something outside the document* refers to it: a [`Node`] or
/// [`Element`] handle, or the document root itself. A node that is only reachable through another
/// node (a child, a parent, a detached subtree that no handle points at) is kept alive by that
/// node, and disappears together with it.
///
/// Unreachable nodes are reclaimed by a mark-and-sweep collection, which runs automatically when an
/// allocation finds the arena full and can also be requested with [`Document::collect_garbage`].
/// A reclaimed slot is handed out again by a later allocation, so [`NodeId`] values are only usable
/// while the node they were created for is alive: an id can be looked up with [`Document::node`],
/// which returns `None` once the slot has been reclaimed.
///
/// Two consequences are worth stating explicitly:
///
/// * the document root is a root of the mark phase but *not* a counted reference, so a tree stays
///   alive for as long as the document points at it, no matter which handles exist;
/// * a [`Node`] handle keeps its whole ancestry alive as well, because the handle can walk upwards
///   ([`Node::parent`], [`Node::is_attached`], namespace resolution).
///
/// # Capacity
///
/// The arena never compacts, so a slot index never moves, but the number of slots a document can
/// hold at one time is limited by the `u32` id space ([`MAX_NODES`](crate::MAX_NODES)). The
/// capacity itself is configurable and grows geometrically: see [`GcConfig`] for the thresholds and
/// [`Document::capacity`] for the current value.
///
/// # Thread safety
///
/// The document is guarded by exactly one reader-writer lock. Reads take the lock in shared mode,
/// every mutation takes it exclusively, and no operation ever acquires a second lock. There is
/// therefore no ordering between locks that could deadlock. In debug builds a re-entrancy guard
/// asserts the invariant and panics instead of hanging.
#[derive(Clone)]
pub struct Document {
    /// Shared state of this document.
    pub(crate) inner: Arc<DocumentInner>,
}

impl Default for Document {
    fn default() -> Self {
        Self::empty()
    }
}

impl Document {
    /// Creates a new, empty document (no root element) with the default [`GcConfig`].
    pub fn empty() -> Self {
        Self::with_gc_config(GcConfig::default())
    }

    /// Creates a new, empty document whose arena uses `config`.
    ///
    /// The configuration controls when the document collects garbage and how its arena grows; see
    /// [`GcConfig`]. The default is [`Document::empty`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use biodivine_lib_xml_dom::{Document, GcConfig};
    ///
    /// let document = Document::with_gc_config(GcConfig::new().with_initial_capacity(16));
    /// assert_eq!(document.capacity(), 16);
    /// ```
    pub fn with_gc_config(config: GcConfig) -> Self {
        Self {
            inner: Arc::new(DocumentInner {
                arena: RwLock::new(Arena::with_config(config)),
            }),
        }
    }

    /// The identity of the underlying document, used by the re-entrancy guard.
    fn lock_key(&self) -> usize {
        Arc::as_ptr(&self.inner) as usize
    }

    /// Runs `read` with shared access to the arena.
    ///
    /// This is the *only* way to read arena state. It is crate-internal so that the locking
    /// invariant can be audited by looking at the handful of call sites plus this helper.
    ///
    /// # Panics
    ///
    /// In debug builds, panics if the current thread already holds this document's lock (which
    /// would deadlock).
    #[track_caller]
    pub(crate) fn read_arena<R>(
        &self,
        operation: &'static str,
        read: impl FnOnce(&Arena) -> R,
    ) -> R {
        let _guard = reentry::Guard::enter(self.lock_key(), operation);
        let arena = self.inner.arena.read();
        read(&arena)
    }

    /// Runs `write` with exclusive access to the arena. See [`Document::read_arena`].
    ///
    /// # Panics
    ///
    /// In debug builds, panics if the current thread already holds this document's lock (which
    /// would deadlock).
    #[track_caller]
    pub(crate) fn write_arena<R>(
        &self,
        operation: &'static str,
        write: impl FnOnce(&mut Arena) -> R,
    ) -> R {
        let _guard = reentry::Guard::enter(self.lock_key(), operation);
        let mut arena = self.inner.arena.write();
        write(&mut arena)
    }

    /// Looks up one node and returns a handle for it, creating the counted reference while the
    /// arena lock is still held.
    ///
    /// Every method that hands a node to the outside world goes through this helper (or through
    /// [`Document::read_many`]/[`Document::write_one`]), because a bare [`NodeId`] must never cross
    /// a critical section: a collection running between the lookup and the creation of the handle
    /// could otherwise reclaim the slot and hand the id to a different node.
    #[track_caller]
    pub(crate) fn read_one(
        &self,
        operation: &'static str,
        lookup: impl FnOnce(&Arena) -> Option<NodeId>,
    ) -> Option<Node> {
        self.read_arena(operation, |arena| {
            let id = lookup(arena)?;
            Some(Node::from_ref(self.clone(), id, arena.external_ref(id)?))
        })
    }

    /// Looks up several nodes and returns handles for them, all created inside one critical section.
    ///
    /// Ids that do not refer to a live node are skipped; the callers only pass ids they obtained
    /// from the same arena, so this is a safety net rather than an expected case.
    #[track_caller]
    pub(crate) fn read_many(
        &self,
        operation: &'static str,
        lookup: impl FnOnce(&Arena) -> Vec<NodeId>,
    ) -> Vec<Node> {
        self.read_arena(operation, |arena| {
            lookup(arena)
                .into_iter()
                .filter_map(|id| {
                    Some(Node::from_ref(self.clone(), id, arena.external_ref(id)?))
                })
                .collect()
        })
    }

    /// Edits the arena and returns a handle for the node the edit produced, created inside the same
    /// critical section. See [`Document::read_one`] for why that matters.
    ///
    /// # Panics
    ///
    /// In debug builds, panics if the current thread already holds this document's lock.
    #[track_caller]
    pub(crate) fn write_one(
        &self,
        operation: &'static str,
        edit: impl FnOnce(&mut Arena) -> Option<NodeId>,
    ) -> Option<Node> {
        self.write_arena(operation, |arena| {
            let id = edit(arena)?;
            Some(Node::from_ref(self.clone(), id, arena.external_ref(id)?))
        })
    }

    /// Whether two handles refer to the same document.
    pub fn ptr_eq(&self, other: &Document) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }

    /// The XML declaration of this document, if it was parsed from a document that had one (or set
    /// explicitly with [`Document::set_xml_declaration`]).
    ///
    /// The declaration is metadata about the document, not a node: it is not part of the tree, it
    /// is not a child of anything, and it is written back by the serializer according to
    /// [`WriteOptions`](crate::WriteOptions).
    pub fn xml_declaration(&self) -> Option<XmlDeclaration> {
        self.read_arena("Document::xml_declaration", |arena| arena.declaration())
    }

    /// Replaces the XML declaration of this document.
    ///
    /// The declaration is not validated here; use
    /// [`crate::xml_spec::declaration::encoding_is_utf8`] and compare
    /// [`crate::xml_spec::XML_VERSION`] if the value does not come from this crate's parser.
    pub fn set_xml_declaration(&self, declaration: Option<XmlDeclaration>) {
        self.write_arena("Document::set_xml_declaration", |arena| {
            arena.set_declaration(declaration);
        });
    }

    /// The root element of this document, if it has one.
    ///
    /// The returned handle keeps the root and its subtree alive; the document root is a mark root
    /// of the collector on its own, so the tree also survives while the document does.
    pub fn root(&self) -> Option<Element> {
        self.read_one("Document::root", |arena| arena.root())
            .map(Element::new_unchecked)
    }

    /// Sets the root element, returning the previous root (if any).
    ///
    /// The previous root is *not* deleted: it stays in the arena as a detached subtree, so
    /// existing handles to it remain valid and it can be re-attached elsewhere.
    ///
    /// # Panics
    ///
    /// Panics if `root` belongs to a different document, if it is not an element, or if it is
    /// already attached to a parent. Use [`Document::set_root_checked`] for the fallible variant.
    #[track_caller]
    pub fn set_root(&self, root: Element) -> Option<Element> {
        match self.set_root_checked(root) {
            Ok(previous) => previous,
            Err(error) => panic!("Document::set_root failed: {error}"),
        }
    }

    /// Fallible variant of [`Document::set_root`].
    ///
    /// # Errors
    ///
    /// - [`XmlError::ForeignDocument`] if `root` belongs to another document.
    /// - [`XmlError::RootHasParent`] if `root` is already attached to a parent.
    ///
    /// (`root` is an [`Element`], so the "not an element" case cannot occur here.)
    pub fn set_root_checked(&self, root: Element) -> XmlResult<Option<Element>> {
        if !root.belongs_to(self) {
            return Err(XmlError::ForeignDocument);
        }
        self.write_arena("Document::set_root_checked", |arena| {
            let previous = arena.set_root(root.id())?;
            Ok(previous.and_then(|id| {
                let external = arena.external_ref(id)?;
                Some(Element::new_unchecked(Node::from_ref(
                    self.clone(),
                    id,
                    external,
                )))
            }))
        })
    }

    /// Removes the root element of this document and returns it.
    ///
    /// The subtree is preserved in the arena; the document simply stops pointing at it.
    pub fn clear_root(&self) -> Option<Element> {
        self.write_one("Document::clear_root", |arena| arena.clear_root())
            .map(Element::new_unchecked)
    }

    /// Creates a new, detached element with the given name.
    pub fn create_element(&self, name: QualifiedName) -> Element {
        let node = self
            .write_one("Document::create_element", |arena| {
                Some(arena.alloc_element(ElementData {
                    name,
                    attributes: Default::default(),
                    namespace_declarations: Default::default(),
                    children: Vec::new(),
                }))
            })
            .expect("the element was just allocated");
        Element::new_unchecked(node)
    }

    /// Creates a new, detached text node.
    ///
    /// # Errors
    ///
    /// Returns [`XmlError::InvalidText`] if the content contains characters that are not legal in
    /// XML ([`rule.well-formedness.legal-characters`](crate::xml_spec::rule_set::rule_well_formedness_legal_characters)), which is checked here so that an invalid
    /// text node cannot be created at all, because validity is enforced by the type.
    pub fn create_text(&self, text: impl AsRef<str>) -> XmlResult<Node> {
        let text = Text::try_from(text.as_ref())?;
        let node = self
            .write_one("Document::create_text", |arena| {
                Some(arena.alloc(NodeData::Text(text)))
            })
            .expect("the node was just allocated");
        Ok(node)
    }

    /// Creates a new, detached comment node.
    ///
    /// # Errors
    ///
    /// Returns [`XmlError::InvalidComment`] if the content contains `--` or ends with `-`
    /// ([`rule.well-formedness.comment-no-double-hyphen`](crate::xml_spec::rule_set::rule_well_formedness_comment_no_double_hyphen)).
    pub fn create_comment(&self, comment: impl AsRef<str>) -> XmlResult<Node> {
        let comment = Comment::try_from(comment.as_ref())?;
        let node = self
            .write_one("Document::create_comment", |arena| {
                Some(arena.alloc(NodeData::Comment(comment)))
            })
            .expect("the node was just allocated");
        Ok(node)
    }

    /// Creates a new, detached CDATA section.
    ///
    /// # Errors
    ///
    /// Returns [`XmlError::InvalidCData`] if the content contains `]]>`
    /// ([`rule.document-structure.cdata-section-must-not-contain-cdend`](crate::xml_spec::rule_set::rule_document_structure_cdata_section_must_not_contain_cdend)).
    pub fn create_cdata(&self, cdata: impl AsRef<str>) -> XmlResult<Node> {
        let cdata = CData::try_from(cdata.as_ref())?;
        let node = self
            .write_one("Document::create_cdata", |arena| {
                Some(arena.alloc(NodeData::CData(cdata)))
            })
            .expect("the node was just allocated");
        Ok(node)
    }

    /// Creates a new, detached processing instruction.
    ///
    /// # Errors
    ///
    /// Returns [`XmlError::InvalidProcessingInstruction`] if the target is not a valid XML `Name`
    /// or matches `xml` case-insensitively ([`rule.well-formedness.pi-target-is-name`](crate::xml_spec::rule_set::rule_well_formedness_pi_target_is_name),
    /// [`rule.well-formedness.pi-target-not-xml`](crate::xml_spec::rule_set::rule_well_formedness_pi_target_not_xml)), or if the content contains `?>`
    /// ([`rule.well-formedness.pi-no-contains-close`](crate::xml_spec::rule_set::rule_well_formedness_pi_no_contains_close)).
    pub fn create_processing_instruction(
        &self,
        target: impl AsRef<str>,
        data: impl AsRef<str>,
    ) -> XmlResult<Node> {
        let target = PiTarget::try_from(target.as_ref())?;
        let data = PiData::try_from(data.as_ref())?;
        let node = self
            .write_one("Document::create_processing_instruction", |arena| {
                Some(arena.alloc(NodeData::ProcessingInstruction { target, data }))
            })
            .expect("the node was just allocated");
        Ok(node)
    }

    /// Checks the whole document and reports **every** problem it can find.
    ///
    /// Whole-document validation: the checks that need a holistic view — namespace scope
    /// integrity, the structural invariants, and the document-wide value rules — are collected in
    /// one pass instead of failing at the first problem, so that a caller can fix everything in one
    /// sweep. The checks themselves are described in the [crate documentation](crate#validation) and
    /// the individual findings are listed in [`ValidationErrorKind`](crate::ValidationErrorKind).
    ///
    /// Editing operations deliberately do *not* perform these checks: removing a
    /// namespace declaration that a subtree relies on, or moving an element into a scope where its
    /// prefix means something else, succeeds silently and is reported here.
    ///
    /// # Scope of the checks
    ///
    /// * *Node-local* rules — whether a name resolves against the namespace declarations in scope
    ///   for the node, and the shapes of `xml:id`, `xml:lang` and `xml:space` — are checked for
    ///   every node, **including detached ones**, so a subtree can be prepared and validated before
    ///   it is attached.
    /// * *Document-wide* rules — currently `xml:id` uniqueness — are checked over the tree
    ///   reachable from the root only. A detached [`Node::deep_clone`] legitimately repeats its
    ///   original's `xml:id` values, and attaching it is what creates the duplicate that
    ///   [`rule.attributes.id-must-be-unique`](crate::xml_spec::rule_set::rule_attributes_id_must_be_unique) is about.
    /// * Being *detached* is never an error by itself (a detached node is a normal state of the data
    ///   model), and neither is a missing root: [`crate::ValidationErrorKind::MissingRoot`] is
    ///   reported, and the rest of the document is still checked.
    ///
    /// The structural diagnostics ([`crate::ValidationErrorKind::ParentChildMismatch`],
    /// [`crate::ValidationErrorKind::CyclicStructure`], [`crate::ValidationErrorKind::RootHasParent`],
    /// [`crate::ValidationErrorKind::RootIsNotAnElement`]) are *defensive self-checks* of invariants the
    /// arena maintains on every mutation, so no sequence of public API calls can produce them;
    /// only [`crate::ValidationErrorKind::MissingRoot`] is reachable from the API. They exist so that the
    /// guarantee does not depend on the maintaining code being correct.
    ///
    /// # Errors
    ///
    /// Returns [`crate::XmlValidationErrors`] containing one [`crate::XmlValidationError`] per
    /// problem. Each carries a [`crate::ValidationErrorKind`], the offending [`NodeId`] (or `None`
    /// for a problem about the document as a whole, such as a missing root element) and a
    /// human-readable message. The list is deterministic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use biodivine_lib_xml_dom::{Document, Namespace, QualifiedName};
    ///
    /// let document = Document::empty();
    /// let ex = Namespace::prefixed("http://example.com", "ex").unwrap();
    /// let root = document.create_element(QualifiedName::with_namespace("item", &ex).unwrap());
    /// document.set_root(root.clone());
    ///
    /// // The name carries a namespace, but nothing declares the prefix.
    /// let errors = document.validate().unwrap_err();
    /// assert_eq!(errors.len(), 1);
    ///
    /// root.declare_namespace(ex);
    /// assert!(document.validate().is_ok());
    /// ```
    pub fn validate(&self) -> Result<(), crate::XmlValidationErrors> {
        let errors = self.read_arena("Document::validate", |arena| {
            crate::validation::validate_arena(arena)
        });
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.into())
        }
    }

    /// Whether the document passes [`Document::validate`].
    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }

    /// The number of nodes this document currently holds.
    ///
    /// Attached and detached nodes are counted alike; nodes that have been reclaimed by a collection
    /// are not. The number can therefore go down, which is the point of the collector.
    pub fn node_count(&self) -> usize {
        self.read_arena("Document::node_count", |arena| arena.live())
    }

    /// All nodes this document currently holds, in slot order.
    ///
    /// The handles that are returned are the only thing keeping nodes without a path to the document
    /// root alive, so calling this method turns the whole live set into roots until the vector is
    /// dropped. That is occasionally useful (enumerating a document) and expensive for the same
    /// reason; prefer a traversal from [`Document::root`] when it is possible.
    pub fn nodes(&self) -> Vec<Node> {
        self.read_many("Document::nodes", |arena| arena.occupied_ids().collect())
    }

    /// Looks up a node by id.
    ///
    /// Returns `None` if the slot of `id` holds no node, either because the id was never produced by
    /// this document, because it is out of range, or because the node has been reclaimed by a
    /// collection. On success the returned handle keeps the node alive.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use biodivine_lib_xml_dom::{Document, NodeId};
    ///
    /// let document = Document::empty();
    /// let node = document.create_text("kept").unwrap();
    ///
    /// assert!(document.node(node.id()).is_some());
    /// assert!(document.node(NodeId::from(4_000_000)).is_none());
    /// ```
    pub fn node(&self, id: NodeId) -> Option<Node> {
        self.read_one("Document::node", |arena| arena.occupied(id).map(|_| id))
    }

    // -------------------------------------------------------------------------------------
    // Capacity policy and garbage collection
    // -------------------------------------------------------------------------------------

    /// Reclaims every node that is unreachable from a handle or from the document root.
    ///
    /// The collector runs automatically when an allocation finds the arena full, so calling it by
    /// hand is a way to bound memory earlier, or to observe what is still alive. The capacity is not
    /// changed by this call.
    ///
    /// A reference that another thread drops while this runs may not be observed, in which case the
    /// node it referred to survives this collection and is reclaimed by the next one. This is safe
    /// (a node is never reclaimed while a handle to it exists), it only makes collection slightly
    /// conservative.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use biodivine_lib_xml_dom::Document;
    ///
    /// let document = Document::empty();
    /// let discarded = document.create_text("temporary").unwrap();
    ///
    /// // A handle keeps the node ...
    /// assert_eq!(document.collect_garbage().collected, 0);
    /// assert_eq!(document.node_count(), 1);
    ///
    /// // ... and dropping it makes the node collectable.
    /// drop(discarded);
    /// assert_eq!(document.collect_garbage().collected, 1);
    /// assert_eq!(document.node_count(), 0);
    /// ```
    pub fn collect_garbage(&self) -> GcStats {
        self.write_arena("Document::collect_garbage", |arena| arena.collect())
    }

    /// The number of nodes this document may hold before it considers its arena full.
    ///
    /// The capacity grows geometrically (see [`GcConfig`]) and never shrinks, so it is a high-water
    /// mark rather than the current memory usage; [`Document::node_count`] is the number of nodes.
    pub fn capacity(&self) -> usize {
        self.read_arena("Document::capacity", |arena| arena.capacity())
    }

    /// The capacity policy of this document.
    pub fn gc_config(&self) -> GcConfig {
        self.read_arena("Document::gc_config", |arena| arena.config())
    }

    /// Replaces the capacity policy of this document.
    ///
    /// The new thresholds apply to the next allocation and the next growth. Because the capacity
    /// never shrinks, a configuration with a smaller initial capacity does not release memory; if
    /// its initial capacity is *larger* than the current capacity, the capacity is raised to it.
    pub fn set_gc_config(&self, config: GcConfig) {
        self.write_arena("Document::set_gc_config", |arena| arena.set_config(config));
    }
}

impl PartialEq for Document {
    fn eq(&self, other: &Self) -> bool {
        self.ptr_eq(other)
    }
}

impl Eq for Document {}

impl std::hash::Hash for Document {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (Arc::as_ptr(&self.inner) as usize).hash(state);
    }
}

impl fmt::Debug for Document {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Document")
            .field("id", &(Arc::as_ptr(&self.inner) as usize))
            .field("nodes", &self.node_count())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn documents_are_equal_by_identity() {
        let a = Document::empty();
        let b = Document::empty();
        let a2 = a.clone();
        assert!(a == a2);
        assert!(a != b);
        assert!(a.ptr_eq(&a2));
        assert!(!a.ptr_eq(&b));
    }

    #[test]
    fn empty_documents_have_no_root() {
        let document = Document::empty();
        assert!(document.root().is_none());
        assert_eq!(document.node_count(), 0);
        assert!(document.clear_root().is_none());
    }

    /// The re-entrancy guard must catch a nested acquisition of the *same* document's lock:
    /// this is the operation that would deadlock.
    ///
    /// Debug-only, because the guard itself is: in release builds it compiles away, so there is
    /// nothing to observe. `cargo test --release` found this the hard way.
    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "re-entrant access to the lock of one document")]
    fn re_entrant_access_to_the_same_document_panics_in_debug_builds() {
        let document = Document::empty();
        document.read_arena("outer", |_| {
            document.read_arena("inner", |_| ());
        });
    }

    /// Acquiring the lock of a *different* document is not re-entrancy. Production code never
    /// does this (cross-document copies snapshot the source first), but the guard is keyed by
    /// document identity precisely so that it reports the real hazard and not this one.
    #[test]
    fn nesting_access_to_two_different_documents_is_allowed() {
        let first = Document::empty();
        let second = Document::empty();
        let result = first.read_arena("first", |first_arena| {
            let inner = second.read_arena("second", |second_arena| second_arena.live());
            (first_arena.live(), inner)
        });
        assert_eq!(result, (0, 0));
    }

    #[test]
    fn handles_are_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Document>();
        assert_send_sync::<Node>();
        assert_send_sync::<Element>();
        assert_send_sync::<crate::Namespace>();
        assert_send_sync::<QualifiedName>();
        assert_send_sync::<NodeId>();
    }
}
