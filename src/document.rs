//! Wiring between the arena and the outside world: type [`Document`], the single document lock,
//! and the debug-only re-entrancy detector.
//!
//! See `docs/design/PLAN.md` §3 for the reasoning. In short:
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

use std::fmt;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::arena::{Arena, ElementData, NodeData, NodeId};
use crate::element::Element;
use crate::error::{XmlError, XmlResult};
use crate::node::Node;
use crate::qualified_name::QualifiedName;
use crate::xml_spec::{CData, Comment, PiData, PiTarget, Text};

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
/// arena behind a single lock, and [`Node`]/[`Element`] handles are nothing
/// but an id into that arena plus a clone of this handle.
///
/// # Identity
///
/// Two [`Document`] objects are equal if and only if they refer to the same underlying document
/// (pointer equality). This is what makes a [`Node`] comparable across threads and what lets the
/// API reject cross-document operations (requirement (2)).
///
/// # Capacity
///
/// Node ids are `u32` indices and slots are never reused, so a document can hold at most
/// `u32::MAX` nodes over its whole lifetime, and detached nodes keep occupying their slot. See
/// [`NodeId`] and `docs/design/PLAN.md` risk R3.
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
    /// Creates a new, empty document (no root element).
    pub fn empty() -> Self {
        Self {
            inner: Arc::new(DocumentInner {
                arena: RwLock::new(Arena::new()),
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

    /// Whether two handles refer to the same document.
    pub fn ptr_eq(&self, other: &Document) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }

    /// The root element of this document, if it has one.
    pub fn root(&self) -> Option<Element> {
        self.read_arena("Document::root", |arena| arena.root())
            .map(|id| Element::new_unchecked(Node::new(self.clone(), id)))
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
            arena.set_root(root.id())
        })
        .map(|previous| previous.map(|id| Element::new_unchecked(Node::new(self.clone(), id))))
    }

    /// Removes the root element of this document and returns it.
    ///
    /// The subtree is preserved in the arena; the document simply stops pointing at it.
    pub fn clear_root(&self) -> Option<Element> {
        self.write_arena("Document::clear_root", |arena| arena.clear_root())
            .map(|id| Element::new_unchecked(Node::new(self.clone(), id)))
    }

    /// Creates a new, detached element with the given name.
    pub fn create_element(&self, name: QualifiedName) -> Element {
        let id = self.write_arena("Document::create_element", |arena| {
            arena.alloc_element(ElementData {
                name,
                attributes: Default::default(),
                namespace_declarations: Default::default(),
                children: Vec::new(),
            })
        });
        Element::new_unchecked(Node::new(self.clone(), id))
    }

    /// Creates a new, detached text node.
    ///
    /// # Errors
    ///
    /// Returns [`XmlError::InvalidText`] if the content contains characters that are not legal in
    /// XML (`rule.well-formedness.legal-characters`), which is checked here so that an invalid
    /// text node cannot be created at all — requirement (4)(1).
    pub fn create_text(&self, text: impl AsRef<str>) -> XmlResult<Node> {
        let text = Text::try_from(text.as_ref())?;
        let id = self.write_arena("Document::create_text", |arena| {
            arena.alloc(NodeData::Text(text))
        });
        Ok(Node::new(self.clone(), id))
    }

    /// Creates a new, detached comment node.
    ///
    /// # Errors
    ///
    /// Returns [`XmlError::InvalidComment`] if the content contains `--` or ends with `-`
    /// (`rule.well-formedness.comment-no-double-hyphen`).
    pub fn create_comment(&self, comment: impl AsRef<str>) -> XmlResult<Node> {
        let comment = Comment::try_from(comment.as_ref())?;
        let id = self.write_arena("Document::create_comment", |arena| {
            arena.alloc(NodeData::Comment(comment))
        });
        Ok(Node::new(self.clone(), id))
    }

    /// Creates a new, detached CDATA section.
    ///
    /// # Errors
    ///
    /// Returns [`XmlError::InvalidCData`] if the content contains `]]>`
    /// (`rule.document-structure.cdata-section-must-not-contain-cdend`).
    pub fn create_cdata(&self, cdata: impl AsRef<str>) -> XmlResult<Node> {
        let cdata = CData::try_from(cdata.as_ref())?;
        let id = self.write_arena("Document::create_cdata", |arena| {
            arena.alloc(NodeData::CData(cdata))
        });
        Ok(Node::new(self.clone(), id))
    }

    /// Creates a new, detached processing instruction.
    ///
    /// # Errors
    ///
    /// Returns [`XmlError::InvalidProcessingInstruction`] if the target is not a valid XML `Name`
    /// or matches `xml` case-insensitively (`rule.well-formedness.pi-target-is-name`,
    /// `rule.well-formedness.pi-target-not-xml`), or if the content contains `?>`
    /// (`rule.well-formedness.pi-no-contains-close`).
    pub fn create_processing_instruction(
        &self,
        target: impl AsRef<str>,
        data: impl AsRef<str>,
    ) -> XmlResult<Node> {
        let target = PiTarget::try_from(target.as_ref())?;
        let data = PiData::try_from(data.as_ref())?;
        let id = self.write_arena("Document::create_processing_instruction", |arena| {
            arena.alloc(NodeData::ProcessingInstruction { target, data })
        });
        Ok(Node::new(self.clone(), id))
    }

    /// The number of arena slots this document uses.
    ///
    /// Because slots are never reclaimed, this counts every node ever created in this document,
    /// including detached ones. See `docs/design/PLAN.md` risk R3.
    pub fn node_count(&self) -> usize {
        self.read_arena("Document::node_count", |arena| arena.len())
    }

    /// All nodes of this document, attached and detached alike, in creation order.
    pub fn nodes(&self) -> Vec<Node> {
        self.read_arena("Document::nodes", |arena| arena.ids().collect::<Vec<_>>())
            .into_iter()
            .map(|id| Node::new(self.clone(), id))
            .collect()
    }

    /// Looks up a node by id.
    ///
    /// Returns `None` if `id` was produced by a different document and happens to be out of range
    /// here. Passing a foreign id that is in range yields *some* node of this document; ids are
    /// only meaningful together with the document that produced them, which is why
    /// [`Node`] pairs the two.
    pub fn node(&self, id: NodeId) -> Option<Node> {
        self.read_arena("Document::node", |arena| {
            if id.index() < arena.len() {
                Some(id)
            } else {
                None
            }
        })
        .map(|id| Node::new(self.clone(), id))
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
            let inner = second.read_arena("second", |second_arena| second_arena.len());
            (first_arena.len(), inner)
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
