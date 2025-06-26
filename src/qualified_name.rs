use crate::element::Element;
use crate::error::{XmlError, XmlResult};
use crate::namespace::Namespace;
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct QualifiedName {
    pub name: String,
    pub namespace: Option<Namespace>,
}

impl QualifiedName {
    /// Create a new QualifiedName. Name must not contain ':'.
    pub fn new(name: String, namespace: Option<Namespace>) -> XmlResult<Self> {
        if name.contains(':') {
            return Err(XmlError::NamespaceError(
                "Attribute name must not contain ':'; use resolved name and namespace".to_string(),
            ));
        }
        Ok(Self { name, namespace })
    }

    /// Resolve a qualified name in the context of an element and optional extra namespace declarations.
    pub fn resolve(
        element: &Element,
        qualified_name: &str,
        extra_ns: Option<&HashMap<String, String>>,
    ) -> XmlResult<Self> {
        if let Some(colon_pos) = qualified_name.find(':') {
            let prefix = &qualified_name[..colon_pos];
            let local_name = &qualified_name[colon_pos + 1..];
            let uri = if let Some(extra) = extra_ns {
                extra
                    .get(prefix)
                    .cloned()
                    .or_else(|| element.get_namespace_uri(prefix))
            } else {
                element.get_namespace_uri(prefix)
            };
            if let Some(uri) = uri {
                let ns = Namespace::prefixed(uri, prefix)
                    .map_err(|e| XmlError::NamespaceError(e.to_string()))?;
                QualifiedName::new(local_name.to_string(), Some(ns))
            } else {
                Err(XmlError::NamespaceError(format!(
                    "Undefined namespace prefix: {}",
                    prefix
                )))
            }
        } else {
            let uri = if let Some(extra) = extra_ns {
                extra
                    .get("")
                    .cloned()
                    .or_else(|| element.get_namespace_uri(""))
            } else {
                element.get_namespace_uri("")
            };
            if let Some(uri) = uri {
                let ns =
                    Namespace::default(uri).map_err(|e| XmlError::NamespaceError(e.to_string()))?;
                QualifiedName::new(qualified_name.to_string(), Some(ns))
            } else {
                QualifiedName::new(qualified_name.to_string(), None)
            }
        }
    }

    pub fn resolve_with_map(
        qualified_name: &str,
        ns_map: &std::collections::HashMap<String, String>,
    ) -> XmlResult<Self> {
        if let Some(colon_pos) = qualified_name.find(':') {
            let prefix = &qualified_name[..colon_pos];
            let local_name = &qualified_name[colon_pos + 1..];
            if let Some(uri) = ns_map.get(prefix) {
                let ns = Namespace::prefixed(uri, prefix)
                    .map_err(|e| XmlError::NamespaceError(e.to_string()))?;
                QualifiedName::new(local_name.to_string(), Some(ns))
            } else {
                Err(XmlError::NamespaceError(format!(
                    "Undefined namespace prefix: {}",
                    prefix
                )))
            }
        } else if let Some(uri) = ns_map.get("") {
            let ns =
                Namespace::default(uri).map_err(|e| XmlError::NamespaceError(e.to_string()))?;
            QualifiedName::new(qualified_name.to_string(), Some(ns))
        } else {
            QualifiedName::new(qualified_name.to_string(), None)
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
    use std::collections::{BTreeSet, HashMap};
    use std::hash::{Hash, Hasher};

    #[test]
    fn test_creation_and_error() {
        let ns = Namespace::default("http://example.com").unwrap();
        let qn = QualifiedName::new("foo".to_string(), Some(ns.clone())).unwrap();
        assert_eq!(qn.name, "foo");
        assert_eq!(qn.namespace, Some(ns));
        assert!(QualifiedName::new("foo:bar".to_string(), None).is_err());
    }

    #[test]
    fn test_equality_and_ordering() {
        let ns1 = Namespace::default("http://example.com").unwrap();
        let ns2 = Namespace::default("http://other.com").unwrap();
        let a = QualifiedName::new("foo".to_string(), Some(ns1.clone())).unwrap();
        let b = QualifiedName::new("foo".to_string(), Some(ns1.clone())).unwrap();
        let c = QualifiedName::new("foo".to_string(), Some(ns2.clone())).unwrap();
        let d = QualifiedName::new("bar".to_string(), Some(ns1.clone())).unwrap();
        let e = QualifiedName::new("foo".to_string(), None).unwrap();
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
        let a = QualifiedName::new("foo".to_string(), Some(ns1)).unwrap();
        let b = QualifiedName::new("foo".to_string(), Some(ns2)).unwrap();
        let mut hasher_a = std::collections::hash_map::DefaultHasher::new();
        let mut hasher_b = std::collections::hash_map::DefaultHasher::new();
        a.hash(&mut hasher_a);
        b.hash(&mut hasher_b);
        assert_eq!(hasher_a.finish(), hasher_b.finish());
    }

    #[test]
    fn test_resolve_no_prefix() {
        let doc = Document::new();
        let el = doc.create_element("foo".to_string());
        el.declare_default_namespace("http://default.com".to_string());
        let qn = QualifiedName::resolve(&el, "bar", None).unwrap();
        assert_eq!(qn.name, "bar");
        assert_eq!(qn.namespace.unwrap().uri(), "http://default.com");
    }

    #[test]
    fn test_resolve_with_prefix() {
        let doc = Document::new();
        let el = doc.create_element("foo".to_string());
        el.declare_namespace("ex".to_string(), "http://example.com".to_string());
        let qn = QualifiedName::resolve(&el, "ex:bar", None).unwrap();
        assert_eq!(qn.name, "bar");
        assert_eq!(qn.namespace.as_ref().unwrap().uri(), "http://example.com");
        assert_eq!(qn.namespace.as_ref().unwrap().prefix(), Some("ex"));
    }

    #[test]
    fn test_resolve_with_extra_ns() {
        let doc = Document::new();
        let el = doc.create_element("foo".to_string());
        let mut extra = HashMap::new();
        extra.insert("ex".to_string(), "http://extra.com".to_string());
        let qn = QualifiedName::resolve(&el, "ex:bar", Some(&extra)).unwrap();
        assert_eq!(qn.name, "bar");
        assert_eq!(qn.namespace.as_ref().unwrap().uri(), "http://extra.com");
    }

    #[test]
    fn test_resolve_undefined_prefix() {
        let doc = Document::new();
        let el = doc.create_element("foo".to_string());
        let err = QualifiedName::resolve(&el, "ex:bar", None).unwrap_err();
        match err {
            XmlError::NamespaceError(_) => {}
            _ => panic!("Expected NamespaceError"),
        }
    }
}
