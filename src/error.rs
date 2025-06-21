use thiserror::Error;

/// Error types for XML DOM operations
#[derive(Error, Debug)]
pub enum XmlError {
    #[error("Invalid XML: {0}")]
    InvalidXml(String),
    #[error("Namespace error: {0}")]
    NamespaceError(String),
    #[error("Element not found")]
    ElementNotFound,
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Quick XML error: {0}")]
    QuickXmlError(#[from] quick_xml::Error),
}

/// Result type for XML DOM operations
pub type XmlResult<T> = Result<T, XmlError>; 