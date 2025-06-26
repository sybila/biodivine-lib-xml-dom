use parking_lot::RwLock;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use crate::document::Document;
use crate::error::{XmlError, XmlResult};
use crate::namespace::Namespace;
use crate::QualifiedName;

#[derive(Debug, Clone)]
pub enum XmlNode {
    Element(Element),
    Text(String),
}

/// Internal representation of an XML element node
#[derive(Debug)]
pub(crate) struct InternalElement {
    /// The ID of the internal document this element belongs to
    pub document: Document,
    /// Element qualified name (local name + namespace)
    pub qualified_name: QualifiedName,
    /// Element attributes
    pub attributes: BTreeMap<QualifiedName, String>,
    /// Child elements
    pub children: Vec<XmlNode>,
    /// Parent element (None if root or detached)
    pub parent: Option<Element>,
    /// Namespace declarations on this element (prefix -> URI)
    pub namespace_declarations: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Element(Arc<RwLock<InternalElement>>);

impl Element {
    /// Create a new element in the given document with a qualified name
    pub(crate) fn new(document: Document, qualified_name: QualifiedName) -> Self {
        Self(Arc::new(RwLock::new(InternalElement {
            document,
            qualified_name,
            attributes: BTreeMap::new(),
            children: Vec::new(),
            parent: None,
            namespace_declarations: HashMap::new(),
        })))
    }

    pub fn name(&self) -> String {
        self.0.read().qualified_name.name.clone()
    }

    pub fn namespace(&self) -> Option<Namespace> {
        self.0.read().qualified_name.namespace.clone()
    }

    pub fn qualified_name(&self) -> String {
        let inner = self.0.read();
        if let Some(ref ns) = inner.qualified_name.namespace {
            if let Some(prefix) = ns.prefix() {
                format!("{}:{}", prefix, inner.qualified_name.name)
            } else {
                inner.qualified_name.name.clone()
            }
        } else {
            inner.qualified_name.name.clone()
        }
    }

    pub fn declare_namespace(&self, prefix: String, uri: String) {
        self.0.write().namespace_declarations.insert(prefix, uri);
    }

    pub fn declare_default_namespace(&self, uri: String) {
        self.0
            .write()
            .namespace_declarations
            .insert("".to_string(), uri);
    }

    pub fn get_namespace_uri(&self, prefix: &str) -> Option<String> {
        let inner = self.0.read();
        if let Some(uri) = inner.namespace_declarations.get(prefix) {
            return Some(uri.clone());
        }
        if let Some(parent) = &inner.parent {
            parent.get_namespace_uri(prefix)
        } else {
            None
        }
    }

    pub fn resolve_qualified_name(
        &self,
        qualified_name: &str,
    ) -> XmlResult<(String, Option<Namespace>)> {
        if let Some(colon_pos) = qualified_name.find(':') {
            let prefix = &qualified_name[..colon_pos];
            let local_name = &qualified_name[colon_pos + 1..];

            if let Some(uri) = self.get_namespace_uri(prefix) {
                let namespace = Namespace::prefixed(uri, prefix).unwrap();
                Ok((local_name.to_string(), Some(namespace)))
            } else {
                Err(XmlError::NamespaceError(format!(
                    "Undefined namespace prefix: {}",
                    prefix
                )))
            }
        } else if let Some(uri) = self.get_namespace_uri("") {
            let namespace = Namespace::default(uri).unwrap();
            Ok((qualified_name.to_string(), Some(namespace)))
        } else {
            Ok((qualified_name.to_string(), None))
        }
    }

    pub fn namespace_declarations(&self) -> HashMap<String, String> {
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
            if let Some(ns) = &qname.namespace {
                if let Some(prefix) = ns.prefix() {
                    if format!("{}:{}", prefix, qname.name) == qualified_name {
                        return Some((qname.clone(), value.clone()));
                    }
                } else if qname.name == qualified_name {
                    return Some((qname.clone(), value.clone()));
                }
            } else if qname.name == qualified_name {
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
