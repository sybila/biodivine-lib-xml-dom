use crate::element::Element;
use crate::error::{XmlError, XmlResult};
use crate::namespace::Namespace;
use crate::xml_spec::{self, NCName};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::sync::OnceLock;

/// Represents a qualified XML name (local name plus optional namespace).
///
/// Used for both element and attribute names. A qualified name consists of a local name
/// and an optional namespace. The namespace, if present, is represented by a [`Namespace`].
///
/// When resolving a qualified name string (e.g., `prefix:local`) into a [`QualifiedName`],
/// the rules differ for elements and attributes:
/// - **Elements**: an unprefixed name binds to the default namespace if one is in scope.
/// - **Attributes**: an unprefixed name always has no namespace, regardless of any default
///   namespace declaration (per XML Namespaces §6.2).
/// - In both cases, the predefined `xml` prefix is automatically bound to
///   `http://www.w3.org/XML/1998/namespace`, and the `xmlns` prefix is forbidden.
///
/// # Equality and Ordering
///
/// Two [`QualifiedName`] objects are considered equal if their local names are equal and their
/// namespaces are semantically equal (i.e., their URIs are equal, see [`Namespace::is_equal_ns`]).
/// Ordering is lexicographic by namespace URI (names with no namespace are placed first),
/// then by local name.
///
/// # Validity
///
/// - The local name must be a valid NCName per XML 1.0: non-empty, must start with a letter
///   (including Unicode), underscore, and may contain letters, digits, underscores, hyphens,
///   and periods (including Unicode). The local name must not contain a colon (`:`).
/// - The namespace, if present, must be a valid [`Namespace`].
///
/// # Examples
///
/// ```rust
/// use biodivine_lib_xml_dom::{Namespace, QualifiedName};
/// let ns = Namespace::without_prefix("http://example.com").unwrap();
/// let qn = QualifiedName::with_namespace("foo", &ns).unwrap();
/// assert_eq!(qn.local_name(), "foo");
/// assert_eq!(qn.namespace().unwrap().uri(), "http://example.com");
/// let qn2 = QualifiedName::without_namespace("bar").unwrap();
/// assert_eq!(qn2.local_name(), "bar");
/// assert!(qn2.namespace().is_none());
/// ```
#[derive(Debug, Clone)]
pub struct QualifiedName {
    local_name: NCName,
    namespace: Option<Namespace>,
}

impl QualifiedName {
    /// Create a new qualified name with a local name and optional namespace.
    ///
    /// This constructor takes an owned [`NCName`] to avoid allocating when the caller
    /// already has validated data (e.g., from [`xml_spec::split_qname`]). For more convenient
    /// constructors that accept `&str`, see [`QualifiedName::without_namespace`] and
    /// [`QualifiedName::with_namespace`].
    pub fn new(name: NCName, namespace: Option<Namespace>) -> Self {
        Self {
            local_name: name,
            namespace,
        }
    }

    /// Create a qualified name without a namespace.
    ///
    /// # Errors
    /// Returns [`XmlError::InvalidXml`] if the name is not a valid NCName (e.g., empty, starts with a digit,
    /// or contains disallowed characters).
    ///
    /// # Examples
    /// ```rust
    /// use biodivine_lib_xml_dom::QualifiedName;
    /// let qn = QualifiedName::without_namespace("foo").unwrap();
    /// assert_eq!(qn.local_name_str(), "foo");
    /// assert!(qn.namespace().is_none());
    /// ```
    pub fn without_namespace<S: AsRef<str>>(name: S) -> XmlResult<Self> {
        let ncname = NCName::try_from(name.as_ref())?;
        Ok(Self {
            local_name: ncname,
            namespace: None,
        })
    }

    /// Create a qualified name with a namespace.
    ///
    /// # Errors
    /// Returns [`XmlError::InvalidXml`] if the name is not a valid NCName (e.g., empty, starts with a digit,
    /// or contains disallowed characters).
    ///
    /// # Examples
    /// ```rust
    /// use biodivine_lib_xml_dom::{Namespace, QualifiedName};
    /// let ns = Namespace::without_prefix("http://example.com").unwrap();
    /// let qn = QualifiedName::with_namespace("foo", &ns).unwrap();
    /// assert_eq!(qn.local_name_str(), "foo");
    /// assert_eq!(qn.namespace().unwrap().uri(), "http://example.com");
    /// ```
    pub fn with_namespace<S: AsRef<str>>(name: S, namespace: &Namespace) -> XmlResult<Self> {
        let ncname = NCName::try_from(name.as_ref())?;
        Ok(Self {
            local_name: ncname,
            namespace: Some(namespace.clone()),
        })
    }

    /// Get the local name as an [`NCName`].
    ///
    /// The returned [`NCName`] is guaranteed to be a valid XML NCName.
    pub fn local_name(&self) -> &NCName {
        &self.local_name
    }

    /// Get the local name as a string slice.
    ///
    /// This is a convenience method that returns the local name as `&str`.
    /// Prefer [`QualifiedName::local_name`] when you need the type-safe [`NCName`] representation.
    pub fn local_name_str(&self) -> &str {
        self.local_name.as_str()
    }

    /// Get the namespace, if any.
    pub fn namespace(&self) -> Option<&Namespace> {
        self.namespace.as_ref()
    }

    /// Resolve a qualified name for an **element** in the context of an [`Element`] and its
    /// namespace declarations.
    ///
    /// The namespace prefix must be declared on the element or one of its parents, unless it
    /// is the predefined `xml` prefix which is automatically bound to
    /// `http://www.w3.org/XML/1998/namespace`.
    ///
    /// Unprefixed element names are bound to the default namespace if one is in scope.
    /// For attribute resolution (where the default namespace does not apply), use
    /// [`QualifiedName::resolve_attribute`].
    ///
    /// # Errors
    /// Returns [`XmlError::InvalidXml`] if the QName is invalid or [`XmlError::NamespaceError`]
    /// if a prefix is not declared.
    ///
    /// # Examples
    /// ```rust
    /// use biodivine_lib_xml_dom::{Document, QualifiedName, Namespace};
    /// let doc = Document::empty();
    /// let el = doc.create_element(QualifiedName::without_namespace("foo").unwrap());
    /// el.declare_default_namespace(Namespace::without_prefix("http://default.com").unwrap());
    /// let qn = QualifiedName::resolve_element(&el, "bar").unwrap();
    /// assert_eq!(qn.local_name(), "bar");
    /// assert_eq!(qn.namespace().unwrap().uri(), "http://default.com");
    /// ```
    pub fn resolve_element(element: &Element, qualified_name: &str) -> XmlResult<Self> {
        Self::resolve_qname(element, qualified_name, true)
    }

    /// Resolve a qualified name for an **attribute** in the context of an [`Element`] and its
    /// namespace declarations.
    ///
    /// The namespace prefix must be declared on the element or one of its parents, unless it
    /// is the predefined `xml` prefix which is automatically bound to
    /// `http://www.w3.org/XML/1998/namespace`.
    ///
    /// Per the XML Namespaces specification (§6.2), default namespace declarations do not
    /// apply to attribute names. An unprefixed attribute always has no namespace, regardless
    /// of any default namespace declaration in scope.
    /// For element resolution (where the default namespace does apply), use
    /// [`QualifiedName::resolve_element`].
    ///
    /// # Errors
    /// Returns [`XmlError::InvalidXml`] if the QName is invalid or [`XmlError::NamespaceError`]
    /// if a prefix is not declared.
    ///
    /// # Examples
    /// ```rust
    /// use biodivine_lib_xml_dom::{Document, QualifiedName, Namespace};
    /// let doc = Document::empty();
    /// let el = doc.create_element(QualifiedName::without_namespace("foo").unwrap());
    /// el.declare_default_namespace(Namespace::without_prefix("http://default.com").unwrap());
    /// // Unprefixed attributes ignore the default namespace
    /// let qn = QualifiedName::resolve_attribute(&el, "bar").unwrap();
    /// assert_eq!(qn.local_name(), "bar");
    /// assert!(qn.namespace().is_none());
    /// ```
    pub fn resolve_attribute(element: &Element, qualified_name: &str) -> XmlResult<Self> {
        Self::resolve_qname(element, qualified_name, false)
    }

    /// Returns the cached namespace for the predefined `xml` prefix.
    fn xml_namespace() -> &'static Namespace {
        static XML_NS: OnceLock<Namespace> = OnceLock::new();
        XML_NS.get_or_init(|| {
            Namespace::prefixed(xml_spec::RESERVED_XML_URI, "xml")
                .expect("xml namespace should always be valid")
        })
    }

    /// Construct a [`QualifiedName`] for the predefined `xml` prefix.
    fn resolve_xml_prefix(local_name: NCName) -> XmlResult<Self> {
        Ok(Self {
            local_name,
            namespace: Some(Self::xml_namespace().clone()),
        })
    }

    /// Internal resolver shared by [`resolve_element`] and [`resolve_attribute`].
    ///
    /// When `apply_default_namespace` is `true`, unprefixed names are bound to the default
    /// namespace if one is in scope (element behavior). When `false`, unprefixed names
    /// always have no namespace (attribute behavior).
    fn resolve_qname(
        element: &Element,
        qualified_name: &str,
        apply_default_namespace: bool,
    ) -> XmlResult<Self> {
        Self::resolve_with_lookup(
            qualified_name,
            apply_default_namespace,
            |prefix| element.get_namespace(Some(prefix)),
            || element.get_namespace(None),
        )
    }

    /// Internal resolver shared by the map-based resolution methods.
    fn resolve_with_map(
        qualified_name: &str,
        ns_map: &HashMap<Option<NCName>, String>,
        apply_default_namespace: bool,
    ) -> XmlResult<Self> {
        Self::resolve_with_lookup(
            qualified_name,
            apply_default_namespace,
            |prefix| {
                ns_map
                    .get(&Some(prefix.clone()))
                    .and_then(|uri| Namespace::prefixed(uri, prefix).ok())
            },
            || {
                ns_map
                    .get(&None)
                    .and_then(|uri| Namespace::without_prefix(uri).ok())
            },
        )
    }

    /// Common resolver that delegates namespace lookup to closures.
    ///
    /// `lookup_prefix` resolves a non-xml prefix to its namespace.
    /// `lookup_default` resolves the default namespace (empty prefix).
    fn resolve_with_lookup<F, G>(
        qualified_name: &str,
        apply_default_namespace: bool,
        lookup_prefix: F,
        lookup_default: G,
    ) -> XmlResult<Self>
    where
        F: FnOnce(&NCName) -> Option<Namespace>,
        G: FnOnce() -> Option<Namespace>,
    {
        let (prefix, local_name) = xml_spec::split_qname(qualified_name)?;

        let namespace = if let Some(prefix) = prefix {
            if prefix == "xml" {
                return Self::resolve_xml_prefix(local_name);
            }

            let ns = lookup_prefix(&prefix).ok_or_else(|| {
                XmlError::NamespaceError(format!("Undefined namespace prefix: {prefix}"))
            })?;

            xml_spec::validate_resolved_prefix(prefix.as_str(), Some(ns.uri()))?;
            Some(ns)
        } else if apply_default_namespace {
            lookup_default()
        } else {
            None
        };

        Ok(Self {
            local_name,
            namespace,
        })
    }

    /// Resolve a qualified name for an **element** using a namespace map (prefix -> URI).
    ///
    /// Unprefixed names are bound to the default namespace (`None` key) if present.
    /// The `xml` prefix is automatically recognized and bound to its reserved URI.
    /// For attribute resolution, use [`QualifiedName::resolve_attribute_with_namespace_map`].
    ///
    /// # Errors
    /// Returns [`XmlError::InvalidXml`] if the QName is invalid or [`XmlError::NamespaceError`]
    /// if a prefix is not found in the map.
    ///
    /// # Examples
    /// ```rust
    /// use std::collections::HashMap;
    /// use biodivine_lib_xml_dom::QualifiedName;
    /// use biodivine_lib_xml_dom::xml_spec::NCName;
    /// let mut ns_map: HashMap<Option<NCName>, String> = HashMap::new();
    /// ns_map.insert(Some("ex".try_into().unwrap()), "http://example.com".to_string());
    /// let qn = QualifiedName::resolve_element_with_namespace_map("ex:foo", &ns_map).unwrap();
    /// assert_eq!(qn.local_name(), "foo");
    /// assert_eq!(qn.namespace().unwrap().uri(), "http://example.com");
    /// ```
    pub fn resolve_element_with_namespace_map(
        qualified_name: &str,
        ns_map: &HashMap<Option<NCName>, String>,
    ) -> XmlResult<Self> {
        Self::resolve_with_map(qualified_name, ns_map, true)
    }

    /// Resolve a qualified name for an **attribute** using a namespace map (prefix -> URI).
    ///
    /// Unprefixed names always have no namespace, per XML Namespaces §6.2.
    /// The `xml` prefix is automatically recognized and bound to its reserved URI.
    /// For element resolution, use [`QualifiedName::resolve_element_with_namespace_map`].
    ///
    /// # Errors
    /// Returns [`XmlError::InvalidXml`] if the QName is invalid or [`XmlError::NamespaceError`]
    /// if a prefix is not found in the map.
    ///
    /// # Examples
    /// ```rust
    /// use std::collections::HashMap;
    /// use biodivine_lib_xml_dom::QualifiedName;
    /// use biodivine_lib_xml_dom::xml_spec::NCName;
    /// let mut ns_map: HashMap<Option<NCName>, String> = HashMap::new();
    /// ns_map.insert(None, "http://default.com".to_string());
    /// // Unprefixed attributes ignore the default namespace
    /// let qn = QualifiedName::resolve_attribute_with_namespace_map("foo", &ns_map).unwrap();
    /// assert_eq!(qn.local_name(), "foo");
    /// assert!(qn.namespace().is_none());
    /// ```
    pub fn resolve_attribute_with_namespace_map(
        qualified_name: &str,
        ns_map: &HashMap<Option<NCName>, String>,
    ) -> XmlResult<Self> {
        Self::resolve_with_map(qualified_name, ns_map, false)
    }
}

impl fmt::Display for QualifiedName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(prefix) = self.namespace().and_then(|ns| ns.prefix()) {
            write!(f, "{prefix}:{}", self.local_name())
        } else {
            write!(f, "{}", self.local_name())
        }
    }
}

impl PartialEq for QualifiedName {
    fn eq(&self, other: &Self) -> bool {
        self.local_name == other.local_name
            && match (&self.namespace, &other.namespace) {
                (Some(a), Some(b)) => a.is_equal_ns(b),
                (None, None) => true,
                _ => false,
            }
    }
}

impl Eq for QualifiedName {}

impl PartialOrd for QualifiedName {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QualifiedName {
    fn cmp(&self, other: &Self) -> Ordering {
        let ns_a = self.namespace.as_ref().map(|ns| ns.uri());
        let ns_b = other.namespace.as_ref().map(|ns| ns.uri());
        match ns_a.cmp(&ns_b) {
            Ordering::Equal => self.local_name.cmp(&other.local_name),
            ord => ord,
        }
    }
}

impl std::hash::Hash for QualifiedName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.local_name.hash(state);
        if let Some(ns) = &self.namespace {
            ns.uri().hash(state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Document;
    use crate::namespace::Namespace;
    use crate::xml_spec::nc_name;
    use std::collections::BTreeSet;
    use std::hash::{Hash, Hasher};

    // Utility function for tests: create a namespace from a string and panic on error.
    fn ns(uri: &str) -> Namespace {
        Namespace::without_prefix(uri).unwrap()
    }
    // Utility function for tests: create a namespace with prefix and panic on error.
    fn pns(uri: &str, prefix: &str) -> Namespace {
        Namespace::prefixed(uri, prefix).unwrap()
    }
    // Utility function for tests: create a qualified name without namespace.
    fn q_name(name: &str) -> XmlResult<QualifiedName> {
        QualifiedName::without_namespace(name)
    }
    // Utility function for tests: create a qualified name with namespace.
    fn q_ns_name(name: &str, ns: &Namespace) -> XmlResult<QualifiedName> {
        QualifiedName::with_namespace(name, ns)
    }

    #[test]
    fn test_creation_and_error() {
        let ns = ns("http://example.com");
        let qn = q_ns_name("foo", &ns).unwrap();
        assert_eq!(qn.local_name_str(), "foo");
        assert_eq!(qn.namespace().unwrap().uri(), "http://example.com");
        // new() takes NCName, so we test via try_from which will fail for invalid input
        assert!(NCName::try_from("foo:bar").is_err());
        assert!(QualifiedName::with_namespace("foo:bar", &ns).is_err());
        assert!(QualifiedName::without_namespace("foo:bar").is_err());
    }

    #[test]
    fn test_equality_and_ordering() {
        let ns1 = ns("http://example.com");
        let ns2 = ns("http://other.com");
        let a = q_ns_name("foo", &ns1).unwrap();
        let b = q_ns_name("foo", &ns1).unwrap();
        let c = q_ns_name("foo", &ns2).unwrap();
        let d = q_ns_name("bar", &ns1).unwrap();
        let e = q_name("foo").unwrap();
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, d);
        assert_ne!(a, e);
        let mut set = BTreeSet::new();
        set.insert(a.clone());
        set.insert(b.clone());
        set.insert(c.clone());
        set.insert(d.clone());
        set.insert(e.clone());
        assert_eq!(set.len(), 4);
        assert!(a < c);
        assert!(d < a);
        assert!(e < d);
    }

    #[test]
    fn test_hashing_semantic_equality() {
        let ns1 = pns("http://example.com", "ex");
        let ns2 = ns("http://example.com");
        let a = q_ns_name("foo", &ns1).unwrap();
        let b = q_ns_name("foo", &ns2).unwrap();
        let mut hasher_a = std::collections::hash_map::DefaultHasher::new();
        let mut hasher_b = std::collections::hash_map::DefaultHasher::new();
        a.hash(&mut hasher_a);
        b.hash(&mut hasher_b);
        assert_eq!(hasher_a.finish(), hasher_b.finish());
    }

    #[test]
    fn test_resolve_no_prefix() {
        let doc = Document::empty();
        let el = doc.create_element(q_name("foo").unwrap());
        el.declare_default_namespace(Namespace::without_prefix("http://default.com").unwrap());
        let qn = QualifiedName::resolve_element(&el, "bar").unwrap();
        assert_eq!(qn.local_name(), "bar");
        assert_eq!(qn.namespace().unwrap().uri(), "http://default.com");
    }

    #[test]
    fn test_resolve_with_prefix() {
        let doc = Document::empty();
        let el = doc.create_element(q_name("foo").unwrap());
        el.declare_namespace(
            &nc_name("ex"),
            Namespace::prefixed("http://example.com", "ex").unwrap(),
        );
        let qn = QualifiedName::resolve_element(&el, "ex:bar").unwrap();
        assert_eq!(qn.local_name(), "bar");
        assert_eq!(qn.namespace().unwrap().uri(), "http://example.com");
        assert_eq!(qn.namespace().unwrap().prefix_str(), Some("ex"));
    }

    #[test]
    fn test_resolve_with_parent_ns() {
        // Test that a namespace declared on a parent element is used for resolution.
        let doc = Document::empty();
        let parent = doc.create_element(q_name("parent").unwrap());
        parent.declare_namespace(
            &nc_name("ex"),
            Namespace::prefixed("http://parent.com", "ex").unwrap(),
        );
        let child = doc.create_element(q_name("child").unwrap());
        // Attach child to parent
        parent.add_child_element(child.clone()).unwrap();
        // Now resolve a qualified name on the child, should use parent's namespace
        let qn = QualifiedName::resolve_element(&child, "ex:bar").unwrap();
        assert_eq!(qn.local_name(), "bar");
        assert_eq!(qn.namespace().unwrap().uri(), "http://parent.com");
    }

    #[test]
    fn test_resolve_undefined_prefix() {
        let doc = Document::empty();
        let el = doc.create_element(q_name("foo").unwrap());
        let err = QualifiedName::resolve_element(&el, "ex:bar").unwrap_err();
        assert!(matches!(err, XmlError::NamespaceError(_)));
    }

    #[test]
    fn test_resolve_xml_prefix_auto() {
        // xml prefix should resolve without being declared
        let doc = Document::empty();
        let el = doc.create_element(q_name("foo").unwrap());
        let qn = QualifiedName::resolve_element(&el, "xml:lang").unwrap();
        assert_eq!(qn.local_name(), "lang");
        assert_eq!(
            qn.namespace().unwrap().uri(),
            "http://www.w3.org/XML/1998/namespace"
        );
    }

    #[test]
    fn test_resolve_xmlns_prefix_rejected() {
        // xmlns prefix should be rejected
        let doc = Document::empty();
        let el = doc.create_element(q_name("foo").unwrap());
        let err = QualifiedName::resolve_element(&el, "xmlns:bar").unwrap_err();
        assert!(matches!(err, XmlError::NamespaceError(_)));
    }

    #[test]
    fn test_qualified_name_string_no_prefix_ns() {
        // Namespace without prefix should produce just the local name
        let ns = ns("http://example.com");
        let qn = q_ns_name("foo", &ns).unwrap();
        assert_eq!(qn.to_string(), "foo");
    }

    #[test]
    fn test_resolve_attribute_ignores_default_ns() {
        // Unprefixed attributes should not inherit the default namespace
        let doc = Document::empty();
        let el = doc.create_element(q_name("foo").unwrap());
        el.declare_default_namespace(Namespace::without_prefix("http://default.com").unwrap());
        let qn = QualifiedName::resolve_attribute(&el, "bar").unwrap();
        assert_eq!(qn.local_name(), "bar");
        assert!(qn.namespace().is_none());
    }

    #[test]
    fn test_resolve_attribute_with_prefix() {
        // Prefixed attributes should resolve normally
        let doc = Document::empty();
        let el = doc.create_element(q_name("foo").unwrap());
        el.declare_namespace(
            &nc_name("ex"),
            Namespace::prefixed("http://example.com", "ex").unwrap(),
        );
        let qn = QualifiedName::resolve_attribute(&el, "ex:bar").unwrap();
        assert_eq!(qn.local_name(), "bar");
        assert_eq!(qn.namespace().unwrap().uri(), "http://example.com");
    }

    #[test]
    fn test_resolve_attribute_xml_prefix_auto() {
        // xml prefix should resolve without being declared for attributes too
        let doc = Document::empty();
        let el = doc.create_element(q_name("foo").unwrap());
        let qn = QualifiedName::resolve_attribute(&el, "xml:lang").unwrap();
        assert_eq!(qn.local_name(), "lang");
        assert_eq!(
            qn.namespace().unwrap().uri(),
            "http://www.w3.org/XML/1998/namespace"
        );
    }

    #[test]
    fn test_resolve_with_map_xml_prefix_auto() {
        let ns_map: HashMap<Option<NCName>, String> = HashMap::new();
        let qn = QualifiedName::resolve_element_with_namespace_map("xml:lang", &ns_map).unwrap();
        assert_eq!(qn.local_name(), "lang");
        assert_eq!(
            qn.namespace().unwrap().uri(),
            "http://www.w3.org/XML/1998/namespace"
        );
    }

    #[test]
    fn test_resolve_attribute_with_map_ignores_default_ns() {
        let mut ns_map: HashMap<Option<NCName>, String> = HashMap::new();
        ns_map.insert(None, "http://default.com".to_string());
        let qn = QualifiedName::resolve_attribute_with_namespace_map("bar", &ns_map).unwrap();
        assert_eq!(qn.local_name(), "bar");
        assert!(qn.namespace().is_none());
    }

    #[test]
    fn test_resolve_with_map_prefixed() {
        let mut ns_map: HashMap<Option<NCName>, String> = HashMap::new();
        ns_map.insert(Some(nc_name("ex")), "http://example.com".to_string());
        let qn = QualifiedName::resolve_element_with_namespace_map("ex:foo", &ns_map).unwrap();
        assert_eq!(qn.local_name(), "foo");
        assert_eq!(qn.namespace().unwrap().uri(), "http://example.com");
        assert_eq!(qn.namespace().unwrap().prefix_str(), Some("ex"));
    }

    #[test]
    fn test_resolve_attribute_with_map_prefixed() {
        let mut ns_map: HashMap<Option<NCName>, String> = HashMap::new();
        ns_map.insert(Some(nc_name("ex")), "http://example.com".to_string());
        let qn = QualifiedName::resolve_attribute_with_namespace_map("ex:foo", &ns_map).unwrap();
        assert_eq!(qn.local_name(), "foo");
        assert_eq!(qn.namespace().unwrap().uri(), "http://example.com");
        assert_eq!(qn.namespace().unwrap().prefix_str(), Some("ex"));
    }

    #[test]
    fn test_ord_consistent_with_partial_eq() {
        let ns1 = ns("http://example.com");
        let ns2 = ns("http://other.com");
        let a = q_ns_name("foo", &ns1).unwrap();
        let b = q_ns_name("foo", &ns1).unwrap();
        let c = q_ns_name("bar", &ns1).unwrap();
        let d = q_name("foo").unwrap();

        // Equal items must compare equal in ordering
        assert_eq!(a.cmp(&b), Ordering::Equal);
        assert_eq!(a, b);

        // Unnamespaced sorts before namespaced
        assert!(d < a);

        // Same namespace, different local name
        assert!(c < a);

        // Different namespaces
        let e = q_ns_name("foo", &ns2).unwrap();
        assert!(a < e);
    }

    #[test]
    fn test_resolve_with_map_undefined_prefix() {
        let ns_map = HashMap::new();
        let err = QualifiedName::resolve_element_with_namespace_map("ex:bar", &ns_map).unwrap_err();
        assert!(matches!(err, XmlError::NamespaceError(_)));
    }
}
