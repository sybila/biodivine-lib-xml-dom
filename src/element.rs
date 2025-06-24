use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use crate::document::Document;
use crate::error::{XmlError, XmlResult};
use crate::namespace::{Attribute, Namespace};

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
    /// Element name (local name)
    pub name: String,
    /// Element namespace
    pub namespace: Option<Namespace>,
    /// Element attributes
    pub attributes: Vec<Attribute>,
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
    /// Create a new element in the given document
    pub(crate) fn new(document: Document, name: String) -> Self {
        Self(Arc::new(RwLock::new(InternalElement {
            document,
            name,
            namespace: None,
            attributes: Vec::new(),
            children: Vec::new(),
            parent: None,
            namespace_declarations: HashMap::new(),
        })))
    }

    /// Create a new namespaced element
    pub(crate) fn with_namespace(document: Document, name: String, namespace: Namespace) -> Self {
        Self(Arc::new(RwLock::new(InternalElement {
            document,
            name,
            namespace: Some(namespace),
            attributes: Vec::new(),
            children: Vec::new(),
            parent: None,
            namespace_declarations: HashMap::new(),
        })))
    }

    pub fn name(&self) -> String {
        self.0.read().name.clone()
    }

    pub fn namespace(&self) -> Option<Namespace> {
        self.0.read().namespace.clone()
    }

    pub fn qualified_name(&self) -> String {
        let inner = self.0.read();
        if let Some(ref ns) = inner.namespace {
            if let Some(ref prefix) = ns.prefix {
                format!("{}:{}", prefix, inner.name)
            } else {
                inner.name.clone()
            }
        } else {
            inner.name.clone()
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
                let namespace = Namespace::prefixed(uri, prefix.to_string());
                Ok((local_name.to_string(), Some(namespace)))
            } else {
                Err(XmlError::NamespaceError(format!(
                    "Undefined namespace prefix: {}",
                    prefix
                )))
            }
        } else if let Some(uri) = self.get_namespace_uri("") {
            let namespace = Namespace::default(uri);
            Ok((qualified_name.to_string(), Some(namespace)))
        } else {
            Ok((qualified_name.to_string(), None))
        }
    }

    pub fn namespace_declarations(&self) -> HashMap<String, String> {
        self.0.read().namespace_declarations.clone()
    }

    pub fn add_attribute(&self, attribute: Attribute) {
        self.0.write().attributes.push(attribute);
    }

    pub(crate) fn set_attributes(&self, attrs: Vec<Attribute>) {
        self.0.write().attributes = attrs;
    }

    pub fn attributes(&self) -> Vec<Attribute> {
        self.0.read().attributes.clone()
    }

    pub fn get_attribute(&self, name: &str) -> Option<Attribute> {
        self.0
            .read()
            .attributes
            .iter()
            .find(|attr| attr.name == name)
            .cloned()
    }

    pub fn get_attribute_by_qualified_name(&self, qualified_name: &str) -> Option<Attribute> {
        self.0
            .read()
            .attributes
            .iter()
            .find(|attr| {
                if let Some(ref ns) = attr.namespace {
                    if let Some(ref prefix) = ns.prefix {
                        format!("{}:{}", prefix, attr.name) == qualified_name
                    } else {
                        attr.name == qualified_name
                    }
                } else {
                    attr.name == qualified_name
                }
            })
            .cloned()
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
