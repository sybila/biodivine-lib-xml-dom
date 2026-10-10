# Design notes

This chapter is a pointer rather than a tutorial: the design decisions, the alternatives that were
rejected and the reasoning live in the repository's design documents, which are kept up to date as
the implementation changes.

| document | what is in it |
| --- | --- |
| [`docs/design/architecture.md`](../design/architecture.md) | the model and its trade-offs, the concurrency and panic-safety arguments, the error model, the namespace contract, module layout and the testing strategy |
| [`docs/design/bindings.md`](../design/bindings.md) | the item-by-item mapping of the Python bindings: what is mirrored, what is deliberately not and why, the error mapping and the GIL policy |
| [`docs/design/rule-coverage.md`](../design/rule-coverage.md) | every XML rule, the layer that acts on it, and what is deliberately out of scope |
| [`docs/design/build.md`](../design/build.md) | how to set the project up, run the checks and build the documentation |
| `specification/` | the XML 1.0 and Namespaces 1.0 specifications, and one file per rule in `specification/rules/` |

Two decisions are worth summarising here because they shape everything a user sees:

**One lock per document, and an arena of nodes.** Nodes live in a vector and refer to each other by
index; a handle is a document reference, an index and a counted reference that keeps the node alive.
That makes an edit a single critical section (so a cycle cannot be raced into existence), removes
reference cycles from the design entirely, and makes reads cheap. The cost is that a node's slot is
held until a collection reclaims it — a documented trade-off, not an oversight.

**"Parse, don't validate" at the boundaries, one validation pass for the whole document.** Names,
text and namespace URIs cannot be constructed invalid, so the type system carries the local
guarantees; the whole-document properties (namespace scope, unique ids, structural invariants) are
collected by `validate` in one pass, reporting every problem at once. Edits stay silent so they stay
fast and predictable.

The XML-specification logic itself lives in the `xml_spec` module, one function per rule, annotated
with the rule it implements: the annotation names the rule summary, the documentation links it, and
the tests assert the summary exists.
