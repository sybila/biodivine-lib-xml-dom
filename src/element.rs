//! The element handle.
//!
//! [`Element`] is a newtype around [`Node`] whose payload is guaranteed to be an element. The
//! invariant is established at construction (the only ways to obtain one are
//! [`Node::as_element`] and [`Document::create_element`]), so none of the element methods has to
//! deal with "this is actually a text node" — the type system does it (requirement (4)(1) on the
//! API level).
//!
//! All tree operations ([`Node::append_child`], [`Node::detach`], …) are available through
//! `Deref<Target = Node>`.

use std::collections::BTreeMap;
use std::ops::Deref;
use std::sync::Arc;

use crate::document::Document;
use crate::error::{XmlError, XmlResult};
use crate::namespace::Namespace;
use crate::node::Node;
use crate::qualified_name::QualifiedName;
use crate::xml_spec::{NCName, Text};

/// A handle to an element node of a [`Document`].
///
/// `Clone` copies the handle, not the element; see [`Node`] for the cloning operations that
/// actually duplicate data.
#[derive(Clone)]
pub struct Element(Node);

impl Element {
    /// Wraps a node whose payload is known to be an element.
    ///
    /// The invariant is upheld by the two construction sites ([`Node::as_element`] and
    /// [`Document::create_element`]); this constructor is crate-internal precisely so that no
    /// other code can break it.
    pub(crate) fn new_unchecked(node: Node) -> Self {
        Self(node)
    }

    /// This element as a generic [`Node`] handle.
    pub fn node(&self) -> Node {
        self.0.clone()
    }

    /// Consumes this handle and returns the generic [`Node`].
    pub fn into_node(self) -> Node {
        self.0
    }

    /// The document this element belongs to.
    pub fn document(&self) -> Document {
        self.0.document()
    }

    // -------------------------------------------------------------------------------------
    // Name
    // -------------------------------------------------------------------------------------

    /// The expanded name of this element.
    ///
    /// The returned value is `Arc`-backed and cheap to clone. Use
    /// [`QualifiedName::local_name`] and [`QualifiedName::namespace`] for the parts.
    pub fn qualified_name(&self) -> QualifiedName {
        let id = self.0.id;
        self.0
            .document
            .read_arena("Element::qualified_name", |arena| {
                arena.element(id).name.clone()
            })
    }

    /// The local name of this element.
    pub fn local_name(&self) -> NCName {
        self.qualified_name().local_name().clone()
    }

    /// The namespace of this element, if it has one.
    pub fn namespace(&self) -> Option<Namespace> {
        self.qualified_name().namespace().cloned()
    }

    /// Changes the name of this element.
    ///
    /// As everywhere else, no namespace declaration is added or removed: the new name simply
    /// carries whatever namespace it was built with. `Document::validate` reports the situation if
    /// the namespace is no longer in scope (requirement (3)).
    pub fn set_qualified_name(&self, name: QualifiedName) {
        let id = self.0.id;
        self.0
            .document
            .write_arena("Element::set_qualified_name", |arena| {
                arena.element_mut(id).name = arena.intern_name(name);
            });
    }

    // -------------------------------------------------------------------------------------
    // Attributes
    // -------------------------------------------------------------------------------------

    /// All attributes of this element, keyed by expanded name.
    ///
    /// Because the attributes live in a map keyed by [`QualifiedName`], two attributes with the
    /// same expanded name are unrepresentable — requirement (4)(1) and
    /// `rule.elements-and-tags.unique-attribute-specification` are enforced by the storage type.
    pub fn attributes(&self) -> BTreeMap<QualifiedName, Arc<str>> {
        let id = self.0.id;
        self.0.document.read_arena("Element::attributes", |arena| {
            arena.element(id).attributes.clone()
        })
    }

    /// The value of the attribute with the given expanded name.
    pub fn attribute(&self, name: &QualifiedName) -> Option<Arc<str>> {
        let id = self.0.id;
        self.0.document.read_arena("Element::attribute", |arena| {
            arena.element(id).attributes.get(name).cloned()
        })
    }

    /// The value of the attribute with the given local name that is in *no* namespace.
    ///
    /// This is the common case (`<a href="…">`) and is the only way an unprefixed attribute can
    /// be spelled, because the default namespace never applies to attributes
    /// (`rule.namespace-usage.default-namespace-not-attributes`).
    pub fn attribute_local(&self, local_name: &NCName) -> Option<Arc<str>> {
        self.attribute(&QualifiedName::new(local_name.clone(), None))
    }

    /// Whether this element has an attribute with the given expanded name.
    pub fn has_attribute(&self, name: &QualifiedName) -> bool {
        self.attribute(name).is_some()
    }

    /// Sets an attribute, overwriting any previous value with the same expanded name.
    ///
    /// Overwriting is the documented default behaviour required by the task description (a map
    /// keyed by expanded name cannot hold two entries, and requiring an explicit removal first
    /// would make the common "update this attribute" case unnecessarily verbose).
    ///
    /// # Panics
    ///
    /// Panics if `value` contains characters that are not legal in XML documents
    /// ([`XmlError::InvalidText`]). Use [`Element::set_attribute_checked`] to handle that case
    /// without panicking.
    ///
    /// # Notes
    ///
    /// The value is stored verbatim; the serializer escapes it. In particular `<` and `&` are
    /// legal to store and are written back as `&lt;` and `&amp;`
    /// (`rule.well-formedness.escape-ampersand-and-lt`).
    #[track_caller]
    pub fn set_attribute(&self, name: QualifiedName, value: impl AsRef<str>) {
        match self.set_attribute_checked(name, value) {
            Ok(()) => {}
            Err(error) => panic!("Element::set_attribute failed: {error}"),
        }
    }

    /// Fallible variant of [`Element::set_attribute`].
    ///
    /// # Errors
    ///
    /// Returns [`XmlError::InvalidText`] if `value` contains characters that are not legal XML
    /// characters (`rule.well-formedness.legal-characters`).
    pub fn set_attribute_checked(
        &self,
        name: QualifiedName,
        value: impl AsRef<str>,
    ) -> XmlResult<()> {
        let value = Text::try_from(value.as_ref())?;
        let id = self.0.id;
        self.0
            .document
            .write_arena("Element::set_attribute_checked", |arena| {
                let name = arena.intern_name(name);
                let element = arena.element_mut(id);
                element.attributes.insert(name, Arc::from(value.as_str()));
            });
        Ok(())
    }

    /// Removes an attribute, returning its previous value.
    pub fn remove_attribute(&self, name: &QualifiedName) -> Option<Arc<str>> {
        let id = self.0.id;
        self.0
            .document
            .write_arena("Element::remove_attribute", |arena| {
                arena.element_mut(id).attributes.remove(name)
            })
    }

    /// Removes all attributes of this element.
    pub fn clear_attributes(&self) {
        let id = self.0.id;
        self.0
            .document
            .write_arena("Element::clear_attributes", |arena| {
                arena.element_mut(id).attributes.clear();
            });
    }

    // -------------------------------------------------------------------------------------
    // Namespaces
    // -------------------------------------------------------------------------------------

    /// The namespace declarations written on *this* element, without inheritance.
    ///
    /// The key is the prefix (`None` for the default namespace); the value is the binding, where
    /// `Some(None)` denotes an empty declaration (`xmlns=""`) that removes the default namespace
    /// from scope (`rule.namespace-usage.empty-default-namespace`).
    pub fn namespace_declarations(&self) -> BTreeMap<Option<NCName>, Option<Namespace>> {
        let id = self.0.id;
        self.0
            .document
            .read_arena("Element::namespace_declarations", |arena| {
                arena.element(id).namespace_declarations.clone()
            })
    }

    /// All namespace bindings visible to this element, innermost first.
    ///
    /// Namespace declarations are inherited down the tree
    /// (`rule.namespace-usage.prefix-declaration-scope`); an inner declaration shadows an outer
    /// one, and an empty default declaration (`xmlns=""`) appears in the result as a binding to
    /// "no namespace".
    pub fn namespaces_in_scope(&self) -> Vec<(Option<NCName>, Option<Namespace>)> {
        let id = self.0.id;
        self.0
            .document
            .read_arena("Element::namespaces_in_scope", |arena| {
                arena.namespaces_in_scope(id)
            })
    }

    /// Declares a namespace on this element, overwriting any previous binding for that prefix.
    ///
    /// The prefix is taken from the [`Namespace`] itself; a namespace without a prefix declares
    /// (or replaces) the default namespace.
    ///
    /// Overwriting is the documented default behaviour required by the task description. Use
    /// [`Element::declare_namespace_checked`] to get an error instead.
    ///
    /// No other node is touched: declarations are never propagated into or out of the subtree
    /// (requirement (3)).
    pub fn declare_namespace(&self, namespace: Namespace) {
        let id = self.0.id;
        let prefix = namespace.prefix().cloned();
        self.0
            .document
            .write_arena("Element::declare_namespace", |arena| {
                let namespace = arena.intern_namespace(namespace);
                arena
                    .element_mut(id)
                    .namespace_declarations
                    .insert(prefix, Some(namespace));
            });
    }

    /// Fallible variant of [`Element::declare_namespace`] that refuses to *change* an existing
    /// binding.
    ///
    /// # Errors
    ///
    /// Returns [`XmlError::InvalidNamespace`] if the prefix is already declared on this element
    /// with a different namespace URI. Declaring the same prefix/URI pair again is allowed and
    /// does nothing.
    ///
    /// The element is never modified when an error is returned.
    pub fn declare_namespace_checked(&self, namespace: Namespace) -> XmlResult<()> {
        let id = self.0.id;
        let prefix = namespace.prefix().cloned();
        self.0
            .document
            .write_arena("Element::declare_namespace_checked", |arena| {
                let existing = arena
                    .element(id)
                    .namespace_declarations
                    .get(&prefix)
                    .cloned();
                if let Some(Some(existing)) = existing
                    && !existing.is_equal_ns(&namespace)
                {
                    return Err(XmlError::InvalidNamespace(format!(
                        "the prefix `{}` is already declared with a different URI (`{}`)",
                        prefix
                            .as_ref()
                            .map_or("(default)", |prefix| prefix.as_str()),
                        existing.uri()
                    )));
                }
                let namespace = arena.intern_namespace(namespace);
                arena
                    .element_mut(id)
                    .namespace_declarations
                    .insert(prefix, Some(namespace));
                Ok(())
            })
    }

    /// Declares `xmlns=""` on this element, removing the default namespace from its scope
    /// (`rule.namespace-usage.empty-default-namespace`).
    pub fn undeclare_default_namespace(&self) {
        let id = self.0.id;
        self.0
            .document
            .write_arena("Element::undeclare_default_namespace", |arena| {
                arena
                    .element_mut(id)
                    .namespace_declarations
                    .insert(None, None);
            });
    }

    /// Removes a namespace declaration from this element.
    ///
    /// `prefix` is `None` for the default namespace. Returns the previous binding (with
    /// `Some(None)` denoting an empty declaration), or `None` if this element had no such
    /// declaration.
    ///
    /// Removing a declaration never checks whether anything in the subtree still relies on it
    /// (requirement (3)); `Document::validate` reports the resulting dangling prefixes.
    pub fn remove_namespace_declaration(
        &self,
        prefix: Option<&NCName>,
    ) -> Option<Option<Namespace>> {
        let id = self.0.id;
        self.0
            .document
            .write_arena("Element::remove_namespace_declaration", |arena| {
                arena
                    .element_mut(id)
                    .namespace_declarations
                    .remove(&prefix.cloned())
            })
    }

    /// The namespace bound to `prefix` in the scope of this element.
    ///
    /// `prefix` is `None` for the default namespace. Returns `None` when the prefix is not bound
    /// at all, including when the default namespace has been removed by an empty declaration.
    ///
    /// The predefined `xml` prefix is *not* resolved here: it is implicitly bound even without a
    /// declaration (Namespaces 1.0 §3, `rule.namespace-basics.xml-prefix-fixed-binding`), which
    /// [`Element::resolve_qualified_name`] handles.
    pub fn get_namespace(&self, prefix: Option<&NCName>) -> Option<Namespace> {
        let id = self.0.id;
        self.0
            .document
            .read_arena("Element::get_namespace", |arena| {
                arena.resolve_prefix(id, prefix)
            })
            .flatten()
    }

    /// Resolves an element name written as a string (`prefix:local` or `local`) against the
    /// namespace declarations that are in scope for this element.
    ///
    /// Unprefixed names are bound to the default namespace if one is in scope, and the predefined
    /// `xml` prefix is always available.
    ///
    /// # Errors
    ///
    /// - [`XmlError::InvalidName`] if the string is not a syntactically valid `QName`
    ///   (`rule.namespace-usage.qname-format`, `rule.namespace-usage.zero-or-one-colon`).
    /// - [`XmlError::UndeclaredPrefix`] if the prefix is not declared in scope
    ///   (`rule.namespace-usage.prefix-declared`).
    /// - [`XmlError::ReservedPrefix`] for the `xmlns` prefix
    ///   (`rule.namespace-basics.xmlns-not-element-prefix`).
    pub fn resolve_qualified_name(&self, name: &str) -> XmlResult<QualifiedName> {
        QualifiedName::resolve_element(self, name)
    }

    /// Resolves an attribute name written as a string against the namespace declarations that
    /// are in scope for this element.
    ///
    /// The default namespace never applies to attributes
    /// (`rule.namespace-usage.default-namespace-not-attributes`).
    ///
    /// # Errors
    ///
    /// As [`Element::resolve_qualified_name`].
    pub fn resolve_attribute_name(&self, name: &str) -> XmlResult<QualifiedName> {
        QualifiedName::resolve_attribute(self, name)
    }

    // -------------------------------------------------------------------------------------
    // Cloning
    // -------------------------------------------------------------------------------------

    /// Creates a detached, shallow copy of this element in the same document.
    ///
    /// Shadowing [`Node::shallow_clone`] so that the result is an [`Element`] again: the copy of
    /// an element is always an element.
    pub fn shallow_clone(&self) -> Element {
        Element::new_unchecked(self.0.shallow_clone())
    }

    /// Creates a detached, deep copy of this element in the same document.
    ///
    /// See [`Node::deep_clone`] for the details.
    pub fn deep_clone(&self) -> Element {
        Element::new_unchecked(self.0.deep_clone())
    }

    /// Creates a detached, shallow copy of this element in `target`.
    ///
    /// See [`Node::shallow_clone_into`].
    pub fn shallow_clone_into(&self, target: &Document) -> Element {
        Element::new_unchecked(self.0.shallow_clone_into(target))
    }

    /// Creates a detached, deep copy of this element in `target`.
    ///
    /// This is the sanctioned way to copy a subtree into another document; see
    /// [`Node::deep_clone_into`].
    pub fn deep_clone_into(&self, target: &Document) -> Element {
        Element::new_unchecked(self.0.deep_clone_into(target))
    }

    /// Detaches this element from its parent and returns it.
    ///
    /// Shadowing [`Node::remove`] so that the result stays an [`Element`].
    pub fn remove(&self) -> Element {
        self.0.detach();
        self.clone()
    }
}

impl Deref for Element {
    type Target = Node;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Debug for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Element({}, id={}, attached={})",
            self.qualified_name(),
            self.0.id,
            self.is_attached()
        )
    }
}

impl std::fmt::Display for Element {
    /// Renders this element (and its subtree) as XML.
    ///
    /// This delegates to the module-level XML serializer, which writes exactly the namespace
    /// declarations stored in the tree and never invents new ones (requirement (3)).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&crate::io::write_element_to_string(self))
    }
}

impl PartialEq for Element {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Element {}

impl std::hash::Hash for Element {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl From<Element> for Node {
    fn from(element: Element) -> Self {
        element.0
    }
}
