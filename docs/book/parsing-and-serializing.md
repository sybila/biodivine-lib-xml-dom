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
:start-after: ANCHOR: roundtrip
:end-before: ANCHOR_END: roundtrip
:dedent:
```
:::
:::{tab-item} Python
:sync: python
```{literalinclude} examples/python/parsing.py
:language: python
:start-after: ANCHOR: roundtrip
:end-before: ANCHOR_END: roundtrip
:dedent:
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
:start-after: ANCHOR: options
:end-before: ANCHOR_END: options
:dedent:
```
:::
:::{tab-item} Python
:sync: python
```{literalinclude} examples/python/parsing.py
:language: python
:start-after: ANCHOR: options
:end-before: ANCHOR_END: options
:dedent:
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
:start-after: ANCHOR: errors
:end-before: ANCHOR_END: errors
:dedent:
```
:::
:::{tab-item} Python
:sync: python
```{literalinclude} examples/python/parsing.py
:language: python
:start-after: ANCHOR: errors
:end-before: ANCHOR_END: errors
:dedent:
```
:::
::::

Note the two that are *not* errors: a comment or a processing instruction outside the root element
is accepted and discarded (the model has one root and nothing else at the top level), and a
document whose names are fine but whose *declarations* are missing parses happily — that is what
[validation](validation.md) is for.

## Entities

The five predefined entities and character references are expanded while parsing; any other
entity reference is a typed error, because this library does not process `DOCTYPE` declarations.

::::{tab-set}
:::{tab-item} Rust
:sync: rust
```{literalinclude} ../../examples/book_parsing.rs
:language: rust
:start-after: ANCHOR: entities
:end-before: ANCHOR_END: entities
:dedent:
```
:::
:::{tab-item} Python
:sync: python
```{literalinclude} examples/python/parsing.py
:language: python
:start-after: ANCHOR: entities
:end-before: ANCHOR_END: entities
:dedent:
```
:::
::::

`DOCTYPE` is read and ignored: this library does not process DTDs. The five predefined entities
(`&amp;`, `&lt;`, `&gt;`, `&apos;`, `&quot;`) and character references (`&#65;`, `&#x41;`) are
expanded; any other entity reference is an error, because without a DTD it can never be declared.
