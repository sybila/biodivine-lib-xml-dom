# API reference

Every public module of the `biodivine_lib_xml_dom` package. The docstrings come from the Python
wrappers and, where a method delegates, from the native bindings; the Rust-level documentation of
the same behaviour, including how this package maps onto the Rust API, is in the rustdoc of
`biodivine-lib-xml-dom` (its `documentation` module).

## Package

```{eval-rst}
.. automodule:: biodivine_lib_xml_dom
   :members:
   :exclude-members: Document, Element, Namespace, Node, NodeId, NodeKind, QualifiedName, ValidationError, ValidationErrors, WriteOptions, DeclarationStyle, EmptyElementStyle, XmlDeclaration, XmlError, XmlSyntaxError, XmlNamespaceError, XmlDocumentError, XmlIoError, XmlValidationError, GcStats
```

## Documents

```{eval-rst}
.. automodule:: biodivine_lib_xml_dom.document
   :members:
```

## Nodes and elements

```{eval-rst}
.. automodule:: biodivine_lib_xml_dom.node
   :members:
```

```{eval-rst}
.. automodule:: biodivine_lib_xml_dom.node_id
   :members:
```

```{eval-rst}
.. automodule:: biodivine_lib_xml_dom.element
   :members:
```

## Names and namespaces

```{eval-rst}
.. automodule:: biodivine_lib_xml_dom.namespace
   :members:
```

```{eval-rst}
.. automodule:: biodivine_lib_xml_dom.name
   :members:
```

## Validation

```{eval-rst}
.. automodule:: biodivine_lib_xml_dom.validation
   :members:
```

## Errors

```{eval-rst}
.. automodule:: biodivine_lib_xml_dom.errors
   :members:
```

## Serialization options

```{eval-rst}
.. automodule:: biodivine_lib_xml_dom.write_options
   :members:
```

## Garbage collection

```{eval-rst}
.. automodule:: biodivine_lib_xml_dom.gc
   :members:
```
