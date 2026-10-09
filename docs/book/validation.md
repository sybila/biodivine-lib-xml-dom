# Validating a document

## Two kinds of integrity

**Local** properties are enforced by the types and checked while you construct: a name that is not a
valid `NCName`, text containing a control character, a comment containing `--`, an attribute value
that is not legal XML — none of them can enter a document. Nothing to remember, nothing to validate.

**Whole-document** properties need a view of the tree, so they are collected by one call that
reports *every* problem it can find, with the node it belongs to and the rule it comes from. That
way a whole class of issues can be fixed in one sweep instead of one failed operation at a time.

## What is checked

| group | examples |
| --- | --- |
| namespace scope | a prefix that is not declared where it is used; a prefix bound to a different URI; an element name in a default namespace that is not declared; an unprefixed element name inside a default-namespace scope (writing it out would move it); an attribute carrying a namespace without a prefix |
| structure | a missing root; and the tree invariants (parent links agreeing, no cycles) as defensive self-checks |
| values | `xml:id` values must be valid names and unique; `xml:lang` must be a language tag or empty; `xml:space` must be `default` or `preserve` |

::::{tab-set}
:::{tab-item} Rust
:sync: rust
```{literalinclude} ../../examples/book_validation.rs
:language: rust
:lines: 8-31
```
:::
:::{tab-item} Python
:sync: python
```{literalinclude} examples/python/validation.py
:language: python
:lines: 5-28
```
:::
::::

Each issue also knows which rule file in `specification/rules/` it enforces, so the message points
at the part of the XML specification it comes from.

## Why `validate` and not the edits

Making every edit check namespace integrity would mean a whole-document scan per operation, and it
would still not help: moving an element into a strange scope is only a *problem* if the surrounding
document is genuinely inconsistent, which is a property of the whole tree. So the division is
deliberate — see [Namespaces](namespaces.md#no-magic):

* edits never check namespace or structural integrity, so they are fast and predictable;
* `validate` is where you ask, and it tells you everything at once.

::::{tab-set}
:::{tab-item} Rust
:sync: rust
```{literalinclude} ../../examples/book_validation.rs
:language: rust
:lines: 33-37
```
:::
:::{tab-item} Python
:sync: python
```{literalinclude} examples/python/validation.py
:language: python
:lines: 30-37
```
:::
::::

A document that came from `parse` is validated for free if you like: a well-formed document whose
names resolve is exactly what validation checks, and the round trip preserves that.
