use crate::element::Element;
use crate::error::{XmlError, XmlResult};
use crate::namespace::Namespace;
use std::cmp::Ordering;
use std::collections::HashMap;

/// Represents a qualified XML name (local name + optional namespace).
///
/// Used for both element and attribute names. A qualified name consists of a local name
/// and an optional namespace. The namespace, if present, is represented by a [`Namespace`].
///
/// # Equality and Ordering
///
/// Two `QualifiedName`s are considered equal if their local names are equal and their namespaces
/// are semantically equal (i.e., their URIs are equal, see [`Namespace::is_equal_ns`]).
/// Ordering is lexicographic by namespace URI, then by local name.
///
/// # Validity
///
/// - The local name must not contain a colon (`:`).
/// - The namespace, if present, must be a valid [`Namespace`].
///
/// # Examples
///
/// ```rust
/// use biodivine_lib_xml_dom::{Namespace, QualifiedName};
/// let ns = Namespace::default("http://example.com").unwrap();
/// let qn = QualifiedName::with_namespace("foo", &ns);
/// assert_eq!(qn.name(), "foo");
/// assert_eq!(qn.namespace().unwrap().uri(), "http://example.com");
/// let qn2 = QualifiedName::without_namespace("bar");
/// assert_eq!(qn2.name(), "bar");
/// assert!(qn2.namespace().is_none());
/// ```
#[derive(Debug, Clone)]
pub struct QualifiedName {
    name: String,
    namespace: Option<Namespace>,
}

impl QualifiedName {
    /// Create a new qualified name with a local name and optional namespace.
    ///
    /// # Errors
    /// Returns an error if the name contains a colon (`:`).
    fn new<S: AsRef<str>>(name: S, namespace: Option<Namespace>) -> XmlResult<Self> {
        let name = name.as_ref();
        if name.contains(':') {
            return Err(XmlError::NamespaceError(
                "Qualified name must not contain ':'; use resolved name and namespace".to_string(),
            ));
        }
        Ok(Self {
            name: name.to_string(),
            namespace,
        })
    }

    /// Create a qualified name without a namespace.
    ///
    /// # Examples
    /// ```rust
    /// use biodivine_lib_xml_dom::QualifiedName;
    /// let qn = QualifiedName::without_namespace("foo");
    /// assert_eq!(qn.name(), "foo");
    /// assert!(qn.namespace().is_none());
    /// ```
    pub fn without_namespace<S: AsRef<str>>(name: S) -> Self {
        Self::new(name, None).unwrap()
    }

    /// Create a qualified name with a namespace.
    ///
    /// # Examples
    /// ```rust
    /// use biodivine_lib_xml_dom::{Namespace, QualifiedName};
    /// let ns = Namespace::default("http://example.com").unwrap();
    /// let qn = QualifiedName::with_namespace("foo", &ns);
    /// assert_eq!(qn.name(), "foo");
    /// assert_eq!(qn.namespace().unwrap().uri(), "http://example.com");
    /// ```
    pub fn with_namespace<S: AsRef<str>>(name: S, namespace: &Namespace) -> Self {
        Self::new(name, Some(namespace.clone())).unwrap()
    }

    /// Get the local name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the namespace, if any.
    pub fn namespace(&self) -> Option<&Namespace> {
        self.namespace.as_ref()
    }

    /// Resolve a qualified name in the context of an element and its namespace declarations.
    ///
    /// The namespace prefix must be declared on the element or one of its parents.
    ///
    /// # Examples
    /// ```rust
    /// use biodivine_lib_xml_dom::{Document, QualifiedName};
    /// let doc = Document::new();
    /// let el = doc.create_element(QualifiedName::without_namespace("foo"));
    /// el.declare_default_namespace("http://default.com".to_string());
    /// let qn = QualifiedName::resolve(&el, "bar").unwrap();
    /// assert_eq!(qn.name(), "bar");
    /// assert_eq!(qn.namespace().unwrap().uri(), "http://default.com");
    /// ```
    pub fn resolve(element: &Element, qualified_name: &str) -> XmlResult<Self> {
        if let Some(colon_pos) = qualified_name.find(':') {
            let prefix = &qualified_name[..colon_pos];
            let local_name = &qualified_name[colon_pos + 1..];
            let uri = element.get_namespace_uri(prefix);
            if let Some(uri) = uri {
                let ns = Namespace::prefixed(uri, prefix)
                    .map_err(|e| XmlError::NamespaceError(e.to_string()))?;
                QualifiedName::new(local_name, Some(ns))
            } else {
                Err(XmlError::NamespaceError(format!(
                    "Undefined namespace prefix: {}",
                    prefix
                )))
            }
        } else {
            let uri = element.get_namespace_uri("");
            if let Some(uri) = uri {
                let ns =
                    Namespace::default(uri).map_err(|e| XmlError::NamespaceError(e.to_string()))?;
                QualifiedName::new(qualified_name, Some(ns))
            } else {
                QualifiedName::new(qualified_name, None)
            }
        }
    }

    /// Resolve a qualified name using a namespace map (prefix -> URI).
    ///
    /// # Examples
    /// ```rust
    /// use std::collections::HashMap;
    /// use biodivine_lib_xml_dom::QualifiedName;
    /// let mut ns_map = HashMap::new();
    /// ns_map.insert("ex".to_string(), "http://example.com".to_string());
    /// let qn = QualifiedName::resolve_with_namespace_map("ex:foo", &ns_map).unwrap();
    /// assert_eq!(qn.name(), "foo");
    /// assert_eq!(qn.namespace().unwrap().uri(), "http://example.com");
    /// ```
    pub fn resolve_with_namespace_map(
        qualified_name: &str,
        ns_map: &HashMap<String, String>,
    ) -> XmlResult<Self> {
        if let Some(colon_pos) = qualified_name.find(':') {
            let prefix = &qualified_name[..colon_pos];
            let local_name = &qualified_name[colon_pos + 1..];
            if let Some(uri) = ns_map.get(prefix) {
                let ns = Namespace::prefixed(uri, prefix)
                    .map_err(|e| XmlError::NamespaceError(e.to_string()))?;
                QualifiedName::new(local_name, Some(ns))
            } else {
                Err(XmlError::NamespaceError(format!(
                    "Undefined namespace prefix: {}",
                    prefix
                )))
            }
        } else if let Some(uri) = ns_map.get("") {
            let ns =
                Namespace::default(uri).map_err(|e| XmlError::NamespaceError(e.to_string()))?;
            QualifiedName::new(qualified_name, Some(ns))
        } else {
            QualifiedName::new(qualified_name, None)
        }
    }
}

impl PartialEq for QualifiedName {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && match (&self.namespace, &other.namespace) {
                (Some(a), Some(b)) => Namespace::is_equal_ns(a, b),
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
            Ordering::Equal => self.name.cmp(&other.name),
            ord => ord,
        }
    }
}

impl std::hash::Hash for QualifiedName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
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
    use std::collections::BTreeSet;
    use std::hash::{Hash, Hasher};

    #[test]
    fn test_creation_and_error() {
        let ns = Namespace::default("http://example.com").unwrap();
        let qn = QualifiedName::with_namespace("foo", &ns);
        assert_eq!(qn.name(), "foo");
        assert_eq!(qn.namespace().unwrap().uri(), "http://example.com");
        assert!(QualifiedName::new("foo:bar", None).is_err());
    }

    #[test]
    fn test_equality_and_ordering() {
        let ns1 = Namespace::default("http://example.com").unwrap();
        let ns2 = Namespace::default("http://other.com").unwrap();
        let a = QualifiedName::with_namespace("foo", &ns1);
        let b = QualifiedName::with_namespace("foo", &ns1);
        let c = QualifiedName::with_namespace("foo", &ns2);
        let d = QualifiedName::with_namespace("bar", &ns1);
        let e = QualifiedName::without_namespace("foo");
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
    }

    #[test]
    fn test_hashing_semantic_equality() {
        let ns1 = Namespace::prefixed("http://example.com", "ex").unwrap();
        let ns2 = Namespace::default("http://example.com").unwrap();
        let a = QualifiedName::with_namespace("foo", &ns1);
        let b = QualifiedName::with_namespace("foo", &ns2);
        let mut hasher_a = std::collections::hash_map::DefaultHasher::new();
        let mut hasher_b = std::collections::hash_map::DefaultHasher::new();
        a.hash(&mut hasher_a);
        b.hash(&mut hasher_b);
        assert_eq!(hasher_a.finish(), hasher_b.finish());
    }

    #[test]
    fn test_resolve_no_prefix() {
        let doc = Document::new();
        let el = doc.create_element(QualifiedName::without_namespace("foo"));
        el.declare_default_namespace("http://default.com".to_string());
        let qn = QualifiedName::resolve(&el, "bar").unwrap();
        assert_eq!(qn.name(), "bar");
        assert_eq!(qn.namespace().unwrap().uri(), "http://default.com");
    }

    #[test]
    fn test_resolve_with_prefix() {
        let doc = Document::new();
        let el = doc.create_element(QualifiedName::without_namespace("foo"));
        el.declare_namespace("ex".to_string(), "http://example.com".to_string());
        let qn = QualifiedName::resolve(&el, "ex:bar").unwrap();
        assert_eq!(qn.name(), "bar");
        assert_eq!(qn.namespace().as_ref().unwrap().uri(), "http://example.com");
        assert_eq!(qn.namespace().as_ref().unwrap().prefix(), Some("ex"));
    }

    #[test]
    fn test_resolve_with_extra_ns() {
        let doc = Document::new();
        let el = doc.create_element(QualifiedName::without_namespace("foo"));
        el.declare_namespace("ex".to_string(), "http://extra.com".to_string());
        let qn = QualifiedName::resolve(&el, "ex:bar").unwrap();
        assert_eq!(qn.name(), "bar");
        assert_eq!(qn.namespace().as_ref().unwrap().uri(), "http://extra.com");
    }

    #[test]
    fn test_resolve_undefined_prefix() {
        let doc = Document::new();
        let el = doc.create_element(QualifiedName::without_namespace("foo"));
        let err = QualifiedName::resolve(&el, "ex:bar").unwrap_err();
        match err {
            XmlError::NamespaceError(_) => {}
            _ => panic!("Expected NamespaceError"),
        }
    }
}
