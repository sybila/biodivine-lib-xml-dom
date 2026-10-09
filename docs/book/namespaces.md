# Namespaces

## Names are expanded, prefixes are decoration

Every element and attribute name in the tree is an *expanded* name: a local name plus, optionally, a
namespace that carries both a URI and the prefix it was written with. Two names are the same if
their local names and their *URIs* match; the prefix is presentation. That is what makes
`<p:x xmlns:p="u"/>` and `<q:x xmlns:q="u"/>` the same element — which is exactly what the XML
Namespaces specification means by an expanded name.

## Scope

A declaration applies from the start tag that carries it to the end of the corresponding end tag, so
it applies to the element itself and to everything below it. An inner declaration shadows an outer
one, an unprefixed element name takes whatever the default namespace is in scope, and an unprefixed
*attribute* never does.

::::{tab-set}
:::{tab-item} Rust
:sync: rust
```{literalinclude} ../../examples/book_namespaces.rs
:language: rust
:lines: 8-33
```
:::
:::{tab-item} Python
:sync: python
```{literalinclude} examples/python/namespaces.py
:language: python
:lines: 4-20
```
:::
::::

The `xml` prefix is special: it is bound to `http://www.w3.org/XML/1998/namespace` without a
declaration, so `xml:lang`, `xml:space` and `xml:id` always resolve.

## `xmlns=""`

An empty default declaration removes the default namespace from the element it is declared on. The
tree records the declaration explicitly, so it survives a round trip:

::::{tab-set}
:::{tab-item} Rust
:sync: rust
```{literalinclude} ../../examples/book_namespaces.rs
:language: rust
:lines: 35-44
```
:::
:::{tab-item} Python
:sync: python
```{literalinclude} examples/python/namespaces.py
:language: python
:lines: 22-27
```
:::
::::

## No magic

The editing API never touches declarations. Removing a declaration that a subtree relies on
succeeds, moving an element into a scope where its prefix means something else succeeds, and the
serializer writes exactly the declarations that are in the tree — it never invents one to "fix" the
output. The contract is: **edits are silent, `validate` reports**.

::::{tab-set}
:::{tab-item} Rust
:sync: rust
```{literalinclude} ../../examples/book_namespaces.rs
:language: rust
:lines: 46-56
```
:::
:::{tab-item} Python
:sync: python
```{literalinclude} examples/python/namespaces.py
:language: python
:lines: 29-37
```
:::
::::

See [Validating a document](validation.md) for what is reported and
[Known limitations and gotchas](limitations.md) for what the library deliberately does not do.
