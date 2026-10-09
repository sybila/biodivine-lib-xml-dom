# Design notes

This chapter is a pointer rather than a tutorial: the design decisions, the alternatives that were
rejected and the reasoning live in the repository's design documents, which are kept up to date as
the implementation changes.

| document | what is in it |
| --- | --- |
| `docs/design/REVIEW.md` | the audit of the 0.1 implementation: per-requirement status, eleven ranked defects with reproductions, and what was kept |
| `docs/design/PLAN.md` | the target architecture, the deadlock-freedom and panic-safety arguments, the `_checked`/panic convention, the API sketch, the test seams, the goal ordering and the risks — plus §15, which records every place the implementation deviated and why |
| `docs/design/BINDINGS.md` | the item-by-item audit of the Python bindings: what is mirrored, what is deliberately not and why, the error mapping, the GIL policy, the `Send`/`Sync` justifications |
| `docs/design/evidence/` | the reproducible evidence: the baseline gates, the defect probes with raw transcripts, the rule inventory and the rule-enforcement verdicts, and the docs/bindings build transcripts |
| `specification/` | the XML 1.0 and Namespaces 1.0 specifications, and one file per rule in `specification/rules/` |

Two decisions are worth summarising here because they shape everything a user sees:

**One lock per document, and an arena of nodes.** Nodes live in a vector and refer to each other by
index; a handle is a document reference plus an index. That makes an edit a single critical section
(so a cycle cannot be raced into existence), removes the reference cycles that leaked memory in 0.1,
and makes reads cheap. The cost is that arena slots are never reused — a documented trade-off, not an
oversight.

**"Parse, don't validate" at the boundaries, one validation pass for the whole document.** Names,
text and namespace URIs cannot be constructed invalid, so the type system carries the local
guarantees; the whole-document properties (namespace scope, unique ids, structural invariants) are
collected by `validate` in one pass, reporting every problem at once. Edits stay silent so they stay
fast and predictable.

The XML-specification logic itself lives in the `xml_spec` module, one function per rule, annotated
with the rule file it implements and tested against it — the convention the project started with and
that this rewrite kept.
