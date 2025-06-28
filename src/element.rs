use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::document::Document;
use crate::error::XmlResult;
use crate::namespace::Namespace;
use crate::QualifiedName;

#[derive(Debug, Clone)]
pub enum XmlNode {
    Element(Element),
    Text(String),
    Comment(String),
    CData(String),
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
    /// Child elements
    pub children: Vec<XmlNode>,
    /// Parent element (None if root or detached)
    pub parent: Option<Element>,
    /// Namespace declarations on this element (prefix -> Namespace)
    pub namespace_declarations: BTreeMap<String, Namespace>,
}

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

    pub fn name(&self) -> String {
        self.0.read().qualified_name.name().to_string()
    }

    pub fn namespace(&self) -> Option<Namespace> {
        self.0.read().qualified_name.namespace().cloned()
    }

    pub fn qualified_name(&self) -> QualifiedName {
        self.0.read().qualified_name.clone()
    }

    pub fn declare_namespace(&self, prefix: String, namespace: Namespace) {
        self.0
            .write()
            .namespace_declarations
            .insert(prefix, namespace);
    }

    pub fn declare_default_namespace(&self, namespace: Namespace) {
        self.0
            .write()
            .namespace_declarations
            .insert("".to_string(), namespace);
    }

    pub fn get_namespace(&self, prefix: &str) -> Option<Namespace> {
        let inner = self.0.read();
        if let Some(ns) = inner.namespace_declarations.get(prefix) {
            return Some(ns.clone());
        }
        if let Some(parent) = &inner.parent {
            parent.get_namespace(prefix)
        } else {
            None
        }
    }

    pub fn resolve_qualified_name(
        &self,
        qualified_name: &str,
    ) -> XmlResult<(String, Option<Namespace>)> {
        match QualifiedName::resolve(self, qualified_name) {
            Ok(qname) => Ok((qname.name().to_string(), qname.namespace().cloned())),
            Err(e) => Err(e),
        }
    }

    pub fn namespace_declarations(&self) -> BTreeMap<String, Namespace> {
        self.0.read().namespace_declarations.clone()
    }

    pub fn add_attribute(&self, name: QualifiedName, value: String) {
        self.0.write().attributes.insert(name, value);
    }

    pub(crate) fn set_attributes(&self, attrs: BTreeMap<QualifiedName, String>) {
        self.0.write().attributes = attrs;
    }

    pub fn attributes(&self) -> BTreeMap<QualifiedName, String> {
        self.0.read().attributes.clone()
    }

    pub fn get_attribute(&self, name: &QualifiedName) -> Option<String> {
        self.0.read().attributes.get(name).cloned()
    }

    pub fn get_attribute_by_qualified_name(
        &self,
        qualified_name: &str,
    ) -> Option<(QualifiedName, String)> {
        let inner = self.0.read();
        for (qname, value) in &inner.attributes {
            if let Some(ns) = qname.namespace() {
                if let Some(prefix) = ns.prefix() {
                    if format!("{}:{}", prefix, qname.name()) == qualified_name {
                        return Some((qname.clone(), value.clone()));
                    }
                } else if qname.name() == qualified_name {
                    return Some((qname.clone(), value.clone()));
                }
            } else if qname.name() == qualified_name {
                return Some((qname.clone(), value.clone()));
            }
        }
        None
    }

    pub fn add_child_element(&self, child: Element) -> crate::error::XmlResult<()> {
        if !Arc::ptr_eq(&self.document().internal, &child.document().internal) {
            return Err(crate::error::XmlError::InvalidOperation(
                "Element belongs to a different document".to_string(),
            ));
        }
        child.0.write().parent = Some(self.clone());
        self.0.write().children.push(XmlNode::Element(child));
        Ok(())
    }

    pub fn add_text(&self, text: String) {
        self.0.write().children.push(XmlNode::Text(text));
    }

    pub fn add_comment(&self, comment: String) {
        self.0.write().children.push(XmlNode::Comment(comment));
    }

    pub fn add_cdata(&self, cdata: String) {
        self.0.write().children.push(XmlNode::CData(cdata));
    }

    pub fn children(&self) -> Vec<XmlNode> {
        self.0.read().children.clone()
    }

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

    pub fn parent(&self) -> Option<Element> {
        self.0.read().parent.clone()
    }

    pub fn is_attached(&self) -> bool {
        self.0.read().parent.is_some()
    }

    pub fn document(&self) -> Document {
        self.0.read().document.clone()
    }
}
