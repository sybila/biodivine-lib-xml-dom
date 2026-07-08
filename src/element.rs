use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::QualifiedName;
use crate::document::Document;
use crate::error::XmlResult;
use crate::namespace::Namespace;
use crate::xml_spec::NCName;

/// A child node of an [`Element`].
///
/// XML elements can contain other elements, text, comments, CDATA sections,
/// and processing instructions. This enum represents all possible child node types.
#[derive(Debug, Clone)]
pub enum XmlNode {
    /// A child element node.
    Element(Element),
    /// A text node.
    Text(String),
    /// A comment node.
    Comment(String),
    /// A CDATA section node.
    CData(String),
    /// A processing instruction node (target, data).
    ProcessingInstruction(String, String),
}

/// Internal representation of an XML element node
#[derive(Debug)]
pub(crate) struct ElementData {
    /// Reference to the document this element belongs to
    pub document: Document,
    /// Element qualified name (local name + namespace)
    pub qualified_name: QualifiedName,
    /// Element attributes
    pub attributes: BTreeMap<QualifiedName, String>,
    /// Child nodes
    pub children: Vec<XmlNode>,
    /// Parent element (None if root or detached)
    pub parent: Option<Element>,
    /// Namespace declarations on this element (prefix -> Option<Namespace>).
    /// The key is `None` for the default namespace, `Some(prefix)` for a prefixed namespace.
    /// The value is `None` for empty declarations (xmlns="" or xmlns:prefix="").
    pub namespace_declarations: BTreeMap<Option<NCName>, Option<Namespace>>,
}

/// Represents an XML element node within a [`Document`].
///
/// An element carries a qualified name, attributes, child nodes (elements, text,
/// comments, CDATA, and processing instructions), and namespace declarations.
/// Elements are reference-counted and internally synchronized, allowing safe
/// shared ownership across threads.
///
/// Use [`Element::qualified_name`] to access the element's name and namespace.
/// For example, `element.qualified_name().local_name()` returns the local name,
/// and `element.qualified_name().namespace()` returns the optional namespace.
#[derive(Debug, Clone)]
pub struct Element(Arc<RwLock<ElementData>>);

impl Element {
    /// Create a new element in the given document with a qualified name
    pub(crate) fn new(document: Document, qualified_name: QualifiedName) -> Self {
        Self(Arc::new(RwLock::new(ElementData {
            document,
            qualified_name,
            attributes: BTreeMap::new(),
            children: Vec::new(),
            parent: None,
            namespace_declarations: BTreeMap::new(),
        })))
    }

    /// Get the element's qualified name.
    ///
    /// The returned [`QualifiedName`] is cheap to clone (internally `Arc`-backed).
    /// Use its methods to access the local name or namespace, e.g.,
    /// `element.qualified_name().local_name()` or `element.qualified_name().namespace()`.
    pub fn qualified_name(&self) -> QualifiedName {
        self.0.read().qualified_name.clone()
    }

    /// Declare a namespace on this element using the prefix carried by the [`Namespace`].
    ///
    /// If the namespace has no prefix, this declares the default namespace.
    /// For empty default declarations (xmlns=""), use [`Element::undeclare_default_namespace`].
    pub fn declare_namespace(&self, namespace: Namespace) {
        self.0
            .write()
            .namespace_declarations
            .insert(namespace.prefix().cloned(), Some(namespace));
    }

    /// Undeclare the default namespace on this element (xmlns="").
    /// This removes the default namespace within its scope per XML Namespaces spec §6.2.
    pub fn undeclare_default_namespace(&self) {
        self.0.write().namespace_declarations.insert(None, None);
    }

    /// Resolve the namespace for a given prefix by walking up the parent chain.
    ///
    /// Returns `None` if the prefix is not declared on this element or any ancestor.
    pub fn get_namespace(&self, prefix: Option<&NCName>) -> Option<Namespace> {
        let (local_result, parent) = {
            let inner = self.0.read();
            let local_result = inner.namespace_declarations.get(&prefix.cloned()).cloned();
            let parent = inner.parent.clone();
            (local_result, parent)
        };
        match local_result {
            Some(Some(ns)) => return Some(ns),
            Some(None) => return None,
            _ => {}
        }
        if let Some(parent) = parent {
            return parent.get_namespace(prefix);
        }
        None
    }

    /// Resolve a qualified name string for an element in the context of this element's
    /// namespace declarations.
    ///
    /// Delegates to [`QualifiedName::resolve_element`].
    pub fn resolve_qualified_name(&self, qualified_name: &str) -> XmlResult<QualifiedName> {
        QualifiedName::resolve_element(self, qualified_name)
    }

    /// Get a clone of this element's namespace declarations.
    pub fn namespace_declarations(&self) -> BTreeMap<Option<NCName>, Option<Namespace>> {
        self.0.read().namespace_declarations.clone()
    }

    /// Add an attribute to this element.
    pub fn add_attribute(&self, name: QualifiedName, value: String) {
        self.0.write().attributes.insert(name, value);
    }

    /// Replace all attributes on this element.
    pub(crate) fn set_attributes(&self, attrs: BTreeMap<QualifiedName, String>) {
        self.0.write().attributes = attrs;
    }

    /// Get a clone of this element's attributes.
    pub fn attributes(&self) -> BTreeMap<QualifiedName, String> {
        self.0.read().attributes.clone()
    }

    /// Get the value of an attribute by its qualified name.
    pub fn get_attribute(&self, name: &QualifiedName) -> Option<String> {
        self.0.read().attributes.get(name).cloned()
    }

    /// Add a child element to this element.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::XmlError::InvalidOperation`] if the child belongs to a
    /// different document, if the child is the same element as `self`, if the child
    /// already has a parent, or if `self` would appear in the subtree of `child` (which
    /// would create a cycle).
    pub fn add_child_element(&self, child: Element) -> XmlResult<()> {
        if Arc::ptr_eq(&self.0, &child.0) {
            return Err(crate::error::XmlError::InvalidOperation(
                "Cannot add an element as its own child".to_string(),
            ));
        }
        if !Arc::ptr_eq(&self.document().internal, &child.document().internal) {
            return Err(crate::error::XmlError::InvalidOperation(
                "Element belongs to a different document".to_string(),
            ));
        }
        if child.0.read().parent.is_some() {
            return Err(crate::error::XmlError::InvalidOperation(
                "Child element already has a parent".to_string(),
            ));
        }
        if child.is_ancestor(self) {
            return Err(crate::error::XmlError::InvalidOperation(
                "Cannot add an ancestor as a child (would create a cycle)".to_string(),
            ));
        }
        child.0.write().parent = Some(self.clone());
        self.0.write().children.push(XmlNode::Element(child));
        Ok(())
    }

    /// Check whether `self` appears in the parent chain of `other`.
    pub fn is_ancestor(&self, other: &Element) -> bool {
        let mut current = other.clone();
        loop {
            let parent = {
                let inner = current.0.read();
                inner.parent.clone()
            };
            if let Some(parent) = parent {
                if Arc::ptr_eq(&self.0, &parent.0) {
                    return true;
                }
                current = parent;
            } else {
                break;
            }
        }
        false
    }

    /// Add a text node as a child of this element.
    pub fn add_text(&self, text: String) {
        self.0.write().children.push(XmlNode::Text(text));
    }

    /// Add a comment node as a child of this element.
    pub fn add_comment(&self, comment: String) {
        self.0.write().children.push(XmlNode::Comment(comment));
    }

    /// Add a CDATA section as a child of this element.
    pub fn add_cdata(&self, cdata: String) {
        self.0.write().children.push(XmlNode::CData(cdata));
    }

    /// Add a processing instruction as a child of this element.
    pub fn add_processing_instruction(&self, target: String, data: String) {
        self.0
            .write()
            .children
            .push(XmlNode::ProcessingInstruction(target, data));
    }

    /// Get all child nodes of this element.
    pub fn children(&self) -> Vec<XmlNode> {
        self.0.read().children.clone()
    }

    /// Get only the element children of this element.
    pub fn element_children(&self) -> Vec<Element> {
        self.0
            .read()
            .children
            .iter()
            .filter_map(|n| {
                if let XmlNode::Element(e) = n {
                    Some(e.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get only the text children of this element.
    pub fn text_children(&self) -> Vec<String> {
        self.0
            .read()
            .children
            .iter()
            .filter_map(|n| {
                if let XmlNode::Text(t) = n {
                    Some(t.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get only the comment children of this element.
    pub fn comment_children(&self) -> Vec<String> {
        self.0
            .read()
            .children
            .iter()
            .filter_map(|n| {
                if let XmlNode::Comment(c) = n {
                    Some(c.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get only the CDATA children of this element.
    pub fn cdata_children(&self) -> Vec<String> {
        self.0
            .read()
            .children
            .iter()
            .filter_map(|n| {
                if let XmlNode::CData(c) = n {
                    Some(c.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get only the processing instruction children of this element.
    pub fn processing_instruction_children(&self) -> Vec<(String, String)> {
        self.0
            .read()
            .children
            .iter()
            .filter_map(|n| {
                if let XmlNode::ProcessingInstruction(target, data) = n {
                    Some((target.clone(), data.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get the parent element, if any.
    pub fn parent(&self) -> Option<Element> {
        self.0.read().parent.clone()
    }

    /// Check if this element is attached to a document (reachable from the document root).
    pub fn is_attached(&self) -> bool {
        let mut current = self.clone();
        loop {
            let (parent, document) = {
                let inner = current.0.read();
                (inner.parent.clone(), inner.document.clone())
            };
            if parent.is_none() {
                let root = document.internal.root();
                return root
                    .as_ref()
                    .map(|r| Arc::ptr_eq(&r.0, &current.0))
                    .unwrap_or(false);
            }
            current = parent.unwrap();
        }
    }

    /// Get the document this element belongs to.
    pub fn document(&self) -> Document {
        self.0.read().document.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Namespace;

    #[test]
    fn test_empty_default_namespace_stops_inheritance() {
        // An empty default namespace declaration (xmlns="") should block
        // inheritance of the default namespace from ancestors.
        let doc = Document::empty();
        let parent = doc.create_element(QualifiedName::without_namespace("parent").unwrap());
        parent.declare_namespace(Namespace::without_prefix("http://default.com").unwrap());

        let child = doc.create_element(QualifiedName::without_namespace("child").unwrap());
        child.undeclare_default_namespace();
        parent.add_child_element(child.clone()).unwrap();

        // Child should not inherit parent's default namespace
        assert!(
            child.get_namespace(None).is_none(),
            "Empty default namespace declaration must stop inheritance"
        );

        // A grandchild without its own declaration also should NOT see the ancestor's default ns
        let grandchild =
            doc.create_element(QualifiedName::without_namespace("grandchild").unwrap());
        child.add_child_element(grandchild.clone()).unwrap();
        assert!(
            grandchild.get_namespace(None).is_none(),
            "Grandchild should not see ancestor's default namespace past empty declaration"
        );
    }

    #[test]
    fn test_get_namespace_prefixed_still_inherits() {
        // Prefixed namespace resolution should still walk up the parent chain.
        let doc = Document::empty();
        let parent = doc.create_element(QualifiedName::without_namespace("parent").unwrap());
        parent.declare_namespace(Namespace::prefixed("http://example.com", "ex").unwrap());

        let child = doc.create_element(QualifiedName::without_namespace("child").unwrap());
        parent.add_child_element(child.clone()).unwrap();

        assert_eq!(
            child.get_namespace(Some(&crate::xml_spec::nc_name("ex"))),
            Some(Namespace::prefixed("http://example.com", "ex").unwrap())
        );
    }

    #[test]
    fn test_undeclare_does_not_affect_prefixed_ns() {
        // Undeclaring the default namespace should not interfere with prefixed namespace resolution.
        let doc = Document::empty();
        let parent = doc.create_element(QualifiedName::without_namespace("parent").unwrap());
        parent.declare_namespace(Namespace::prefixed("http://example.com", "ex").unwrap());

        let child = doc.create_element(QualifiedName::without_namespace("child").unwrap());
        child.undeclare_default_namespace();
        parent.add_child_element(child.clone()).unwrap();

        // Prefixed namespace is still visible through the child
        let ex = crate::xml_spec::nc_name("ex");
        assert_eq!(
            child.get_namespace(Some(&ex)),
            Some(Namespace::prefixed("http://example.com", "ex").unwrap())
        );
    }

    #[test]
    fn test_undeclare_records_declaration() {
        // The undeclaration should be recorded in namespace_declarations.
        let doc = Document::empty();
        let el = doc.create_element(QualifiedName::without_namespace("el").unwrap());
        el.undeclare_default_namespace();

        let decls = el.namespace_declarations();
        assert!(
            decls.contains_key(&None),
            "Default namespace key should be present"
        );
        assert_eq!(
            decls.get(&None),
            Some(&None),
            "Value should be None (undeclaration)"
        );
    }

    #[test]
    fn test_undeclare_re_enables_default_ns() {
        // Undeclaring then re-declaring the default namespace on the same element works.
        let doc = Document::empty();
        let root = doc.create_element(QualifiedName::without_namespace("root").unwrap());
        root.declare_namespace(Namespace::without_prefix("http://first.com").unwrap());

        let child = doc.create_element(QualifiedName::without_namespace("child").unwrap());
        child.undeclare_default_namespace();
        child.declare_namespace(Namespace::without_prefix("http://second.com").unwrap());
        root.add_child_element(child.clone()).unwrap();

        // Child sees its own re-declared default namespace, not parent's.
        assert_eq!(
            child.get_namespace(None),
            Some(Namespace::without_prefix("http://second.com").unwrap())
        );
    }

    #[test]
    fn test_add_child_prevents_cycle() {
        // Adding a descendant as a child would create a cycle and must be rejected.
        let doc = Document::empty();
        let a = doc.create_element(QualifiedName::without_namespace("a").unwrap());
        let b = doc.create_element(QualifiedName::without_namespace("b").unwrap());
        let c = doc.create_element(QualifiedName::without_namespace("c").unwrap());

        a.add_child_element(b.clone()).unwrap();
        b.add_child_element(c.clone()).unwrap();

        // Trying to add `a` as child of `c` would create a -> b -> c -> a cycle
        let result = c.add_child_element(a.clone());
        assert!(
            result.is_err(),
            "Adding an ancestor as a child must be rejected"
        );
    }

    #[test]
    fn test_is_ancestor_direct_parent() {
        let doc = Document::empty();
        let parent = doc.create_element(QualifiedName::without_namespace("parent").unwrap());
        let child = doc.create_element(QualifiedName::without_namespace("child").unwrap());

        parent.add_child_element(child.clone()).unwrap();

        assert!(parent.is_ancestor(&child));
    }

    #[test]
    fn test_is_ancestor_deep_tree() {
        let doc = Document::empty();
        let a = doc.create_element(QualifiedName::without_namespace("a").unwrap());
        let b = doc.create_element(QualifiedName::without_namespace("b").unwrap());
        let c = doc.create_element(QualifiedName::without_namespace("c").unwrap());

        a.add_child_element(b.clone()).unwrap();
        b.add_child_element(c.clone()).unwrap();

        assert!(a.is_ancestor(&c));
        assert!(a.is_ancestor(&b));
    }

    #[test]
    fn test_is_ancestor_not_related() {
        let doc = Document::empty();
        let a = doc.create_element(QualifiedName::without_namespace("a").unwrap());
        let b = doc.create_element(QualifiedName::without_namespace("b").unwrap());

        assert!(!a.is_ancestor(&b));
    }

    #[test]
    fn test_is_ancestor_same_element() {
        let doc = Document::empty();
        let a = doc.create_element(QualifiedName::without_namespace("a").unwrap());

        assert!(!a.is_ancestor(&a));
    }
}
