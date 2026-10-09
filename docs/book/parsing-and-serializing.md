# Parsing and serializing

## A faithful round trip

`parse` and `write` are inverses for anything the data model can represent: prefixes, namespace
declarations, the order of children, escaping and the XML declaration all survive. Output is not
reformatted — text is preserved verbatim, so mixed content is never disturbed.

::::{tab-set}
:::{tab-item} Rust
:sync: rust
```{literalinclude} ../../examples/book_parsing.rs
:language: rust
:lines: 8-12
```
:::
:::{tab-item} Python
:sync: python
```{literalinclude} examples/python/parsing.py
:language: python
:lines: 6-12
```
:::
::::

## Output options

`WriteOptions` chooses whether the `<?xml …?>` declaration is written (`Never`, `IfPresent` — the
default — or `Always`) and whether an element without children is written as `<a/>` or `<a></a>`.
The defaults reproduce the input of a parsed document as closely as the model allows.

::::{tab-set}
:::{tab-item} Rust
:sync: rust
```{literalinclude} ../../examples/book_parsing.rs
:language: rust
:lines: 14-23
```
:::
:::{tab-item} Python
:sync: python
```{literalinclude} examples/python/parsing.py
:language: python
:lines: 14-20
```
:::
::::

## What parsing rejects

Everything that is decidable while looking at one element is enforced *while parsing*, with a typed
error rather than a panic — including input that makes many parsers unhappy:

::::{tab-set}
:::{tab-item} Rust
:sync: rust
```{literalinclude} ../../examples/book_parsing.rs
:language: rust
:lines: 25-45
```
:::
:::{tab-item} Python
:sync: python
```{literalinclude} examples/python/parsing.py
:language: python
:lines: 22-42
```
:::
::::

Note the two that are *not* errors: a comment or a processing instruction outside the root element
is accepted and discarded (the model has one root and nothing else at the top level), and a
document whose names are fine but whose *declarations* are missing parses happily — that is what
[validation](validation.md) is for.

## Entities

`DOCTYPE` is read and ignored: this library does not process DTDs. The five predefined entities
(`&amp;`, `&lt;`, `&gt;`, `&apos;`, `&quot;`) and character references (`&#65;`, `&#x41;`) are
expanded; any other entity reference is an error, because without a DTD it can never be declared.
