# API reference

Every public module of the `biodivine_lib_xml_dom` package. The docstrings come from the Python
wrappers and, where a method delegates, from the native bindings — which is why the Rust-level
documentation of the same behaviour is in `docs/design/BINDINGS.md` and the rustdoc of
`biodivine-lib-xml-dom`.

## Package

```{eval-rst}
.. automodule:: biodivine_lib_xml_dom
   :members:
   :exclude-members: Document, Element, Namespace, Node, NodeId, NodeKind, QualifiedName, ValidationError, ValidationErrors, WriteOptions, DeclarationStyle, EmptyElementStyle, XmlDeclaration, XmlError, XmlSyntaxError, XmlNamespaceError, XmlDocumentError, XmlIoError, XmlValidationError, parse, parse_file, write, write_file
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

## Parsing, writing and options

```{eval-rst}
.. automodule:: biodivine_lib_xml_dom.parsing
   :members:
```

```{eval-rst}
.. automodule:: biodivine_lib_xml_dom.write_options
   :members:
```
