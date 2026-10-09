//! In-process tests of the binding layer.
//!
//! These run inside `cargo test` (the crate's dev-dependency on `pyo3` uses `auto-initialize`), so
//! the mirroring layer is exercised without building a wheel. They live inside the crate because
//! `#[pymethods]` bodies are crate-private: their only public surface is the Python module.
//!
//! What is checked here, and why it is not in `tests-python/`:
//!
//! * every `#[pyclass]` is `Send + Sync`, so a handle can be used from any Python thread — the
//!   `unsafe`-free thread-safety argument of the Rust core carries over to Python;
//! * operations that need no interpreter token (creating, attaching, reading, cloning) really can
//!   be called from a plain Rust thread, which is what "one document, several threads" means;
//! * a round trip through the bindings goes through every layer (document → element → attribute →
//!   serialize), and a validation failure carries every problem.

use biodivine_lib_xml_dom::{Document, Element, Node, NodeId, NodeKind};
use pyo3::prelude::*;

use crate::{
    PyDocument, PyElement, PyNamespace, PyNode, PyNodeArg, PyNodeId, PyNodeKind, PyQualifiedName,
    PyWriteOptions, parse_string, write_string,
};

#[test]
fn handles_are_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    // The Python-facing classes ...
    assert_send_sync::<PyDocument>();
    assert_send_sync::<PyElement>();
    assert_send_sync::<PyNode>();
    assert_send_sync::<PyNamespace>();
    assert_send_sync::<PyQualifiedName>();
    assert_send_sync::<PyNodeId>();
    assert_send_sync::<PyNodeKind>();
    assert_send_sync::<PyWriteOptions>();
    // ... the Rust types behind them ...
    assert_send_sync::<Document>();
    assert_send_sync::<Node>();
    assert_send_sync::<Element>();
    assert_send_sync::<NodeId>();
    assert_send_sync::<NodeKind>();
    // ... and the pointer a Python program actually holds, which is what has to cross threads.
    assert_send_sync::<Py<PyDocument>>();
    assert_send_sync::<Py<PyNode>>();
    assert_send_sync::<Py<PyElement>>();
}

#[test]
fn a_document_round_trips_through_the_bindings() {
    Python::attach(|py| {
        let document = PyDocument::new();
        let namespace = PyNamespace::prefixed("http://example.com", "ex").unwrap();
        let name = PyQualifiedName::with_namespace("item", &namespace).unwrap();

        let root = document.create_element(&name);
        document.set_root(&root).unwrap();
        root.declare_namespace(&namespace);
        let text = document.create_text("hello").unwrap();
        root.node().append_child(PyNodeArg::Node(text)).unwrap();
        root.set_attribute(&PyQualifiedName::without_namespace("a").unwrap(), "v")
            .unwrap();

        assert!(document.is_valid(py));
        assert!(document.validation_errors(py).is_empty());
        assert_eq!(document.node_count(), 2);
        assert_eq!(
            write_string(py, &document).unwrap(),
            r#"<ex:item xmlns:ex="http://example.com" a="v">hello</ex:item>"#
        );

        let root_again = document.root().unwrap();
        assert!(root.ptr_eq(&root_again));
        assert_eq!(root.local_name(), "item");
        assert_eq!(root.node().kind(), PyNodeKind::Element);
        assert_eq!(root.attributes().len(), 1);
        assert_eq!(root.children().len(), 1);
        assert_eq!(root.children()[0].text().as_deref(), Some("hello"));
        assert_eq!(root.node().as_element().unwrap().local_name(), "item");
    });
}

#[test]
fn a_parsed_document_is_the_same_tree_as_the_serialized_one() {
    Python::attach(|py| {
        let source = r#"<ex:root xmlns:ex="http://example.com"><ex:b a="1">t</ex:b></ex:root>"#;
        let document = parse_string(py, source).unwrap();
        assert!(document.is_valid(py));
        assert_eq!(write_string(py, &document).unwrap(), source);

        let root = document.root().unwrap();
        assert_eq!(root.qualified_name().__str__(), "ex:root");
        let child = root.children()[0].as_element().unwrap();
        assert_eq!(child.local_name(), "b");
        assert_eq!(
            child
                .attribute(&PyQualifiedName::without_namespace("a").unwrap())
                .as_deref(),
            Some("1")
        );
    });
}

#[test]
fn a_validation_failure_carries_every_problem() {
    Python::attach(|py| {
        let document = PyDocument::new();
        let missing = PyNamespace::prefixed("http://x", "missing").unwrap();
        let root =
            document.create_element(&PyQualifiedName::with_namespace("root", &missing).unwrap());
        document.set_root(&root).unwrap();
        let xml_namespace =
            PyNamespace::prefixed("http://www.w3.org/XML/1998/namespace", "xml").unwrap();
        root.set_attribute(
            &PyQualifiedName::with_namespace("space", &xml_namespace).unwrap(),
            "nope",
        )
        .unwrap();

        let problems = document.validation_errors(py);
        assert_eq!(problems.len(), 2);
        assert_eq!(
            problems
                .iter()
                .map(|problem| problem.kind())
                .collect::<Vec<_>>(),
            vec!["undeclared_prefix", "invalid_xml_space"]
        );
        assert_eq!(
            problems[0].rule(),
            "rule.namespace-usage.prefix-declared.md"
        );
        assert!(document.validate(py).is_err());
        assert!(!document.is_valid(py));
    });
}

#[test]
fn handles_can_be_moved_into_a_plain_rust_thread() {
    // No interpreter token is needed for any of this, which is the point: the handle is a
    // self-contained, thread-safe view of the document.
    let document = PyDocument::new();
    let root = document.create_element(&PyQualifiedName::without_namespace("root").unwrap());
    document.set_root(&root).unwrap();

    let handle = root.node();
    let worker = std::thread::spawn(move || {
        let child = handle
            .document()
            .create_element(&PyQualifiedName::without_namespace("child").unwrap());
        handle
            .append_child(PyNodeArg::Element(child.clone()))
            .unwrap();
        child.local_name()
    });
    assert_eq!(worker.join().unwrap(), "child");
    assert_eq!(root.children().len(), 1);
}

#[test]
fn a_document_can_be_shared_by_several_threads() {
    let document = PyDocument::new();
    let root = document.create_element(&PyQualifiedName::without_namespace("root").unwrap());
    document.set_root(&root).unwrap();

    let workers: Vec<_> = (0..4)
        .map(|index| {
            let handle = root.node();
            let document = document.clone();
            std::thread::spawn(move || {
                for _ in 0..100 {
                    let node = document
                        .create_element(&PyQualifiedName::without_namespace("item").unwrap())
                        .node();
                    handle.append_child(PyNodeArg::Node(node.clone())).unwrap();
                    node.detach();
                }
                index
            })
        })
        .collect();
    for (index, worker) in workers.into_iter().enumerate() {
        assert_eq!(worker.join().unwrap(), index);
    }
    assert!(root.children().is_empty());
    assert_eq!(document.node_count(), 1 + 4 * 100);
}
