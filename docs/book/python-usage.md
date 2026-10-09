# Using the Python package

## Three layers, one behaviour

`biodivine_lib_xml_dom` is a thin Pythonic layer over the native `biodivine_lib_xml_dom._sys`
extension, which mirrors the Rust API. There is no XML logic in Python, so the behaviour is the
Rust behaviour:

* names are accepted as a `str` (no namespace), a `(local_name, namespace_or_uri)` tuple, or a
  `QualifiedName`;
* a `Namespace` may be given anywhere a namespace is expected, or a URI string;
* any `PathLike` is accepted for file paths;
* a failure raises one of the `XmlError` subclasses rather than returning an error value, because
  Python has no `Result` type.

## What is Pythonic rather than mirrored

::::{tab-set}
:::{tab-item} Rust
:sync: rust
```{literalinclude} ../../examples/book_pythonic.rs
:language: rust
:lines: 9-30
```
:::
:::{tab-item} Python
:sync: python
```{literalinclude} examples/python/pythonic.py
:language: python
:lines: 5-31
```
:::
::::

The extras are conveniences, not a second API: `node.attributes` is a `dict` instead of a list of
pairs, `Element.get(name)` reads an attribute by local name, `validation_errors()` returns a `list`
so `if document.validation_errors():` works, and `Document.create_element_tree(name, *children)`
builds a parent and its children in one call.

## Errors

```python
try:
    document.validate()
except xml.XmlValidationError as error:
    summary, problems = error.args
    for problem in problems:
        print(problem.kind, problem.node, problem.message, problem.rule)
```

All exceptions derive from `XmlError`; the subclasses are `XmlSyntaxError` (the input is not
well-formed), `XmlNamespaceError`, `XmlDocumentError` (an operation on the tree itself: foreign
document, cycle, bad index) and `XmlIoError`.

## Identity and equality

Handles are value objects: `==` compares the *node* (document identity plus arena index) and `hash`
follows, so handles work as dictionary keys; `is` compares Python objects, which are created on
demand. Use `a.ptr_eq(b)` when you want to be explicit, and `a is b` never.

## A note on types

The package ships a `py.typed` marker and full annotations, so type checkers see
`Document`, `Element`, `Node`, `Namespace`, `QualifiedName`, `ValidationErrors`, the option types and
the exception hierarchy. The native `_sys` module is importable but is not the documented surface —
`tests-python/test_public_surface.py` exists precisely to keep it that way.
