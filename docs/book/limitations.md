# Known limitations and gotchas

Everything here is a deliberate decision that is documented somewhere in the repository; this
chapter collects them so the book does not over-promise. Nothing on this page is a bug report —
each entry says where the reasoning lives.

## The data model

**One root element, nothing else at the top level.** A comment or processing instruction outside the
root element is *discarded* when parsing (there is nowhere in the model to keep it), whitespace
outside the root is ignored, non-whitespace text outside the root is an error, and a second root
element is an error. If you need to preserve a leading comment, keep it inside the root.

**Empty text nodes have no representation.** `create_text("")` is accepted, but XML cannot express
an empty text node, so it is omitted when writing and does not come back when parsing. Empty
comments (`<!---->`) and empty CDATA sections (`<![CDATA[]]>`) *can* be expressed and are preserved.

**Comments, CDATA and processing-instruction content are line-end normalised at construction.**
Their content cannot contain an escape sequence, so a literal carriage return there could not
survive a round trip; it is stored as a line feed instead (`xml_spec::normalize_line_ends`). Text
nodes and attribute values are *not* normalised — their bytes are preserved and escaped on output.

**Attribute order is not preserved.** Attributes are stored in a map keyed by expanded name, so they
are written in a deterministic but not necessarily input order. XML attaches no meaning to attribute
order, so this does not change the document's meaning, but the bytes differ.

**No output formatting.** Text is written verbatim, so the serializer never re-indents or wraps a
document, and there is no pretty-printer. Mixed content is therefore never disturbed.

## Standards coverage

**No DTD processing.** `DOCTYPE` declarations are read and ignored: no entities defined by a DTD, no
attribute types, no content models, no validity checking. Consequences worth knowing:

* only the five predefined entities and character references are expanded; any other entity
  reference is a typed error, because it can never be declared;
* `xml:id` is the only attribute whose type this library can know, so "unique IDs" is checked for
  `xml:id` only;
* the two `xml:lang`/`xml:space` "must be declared" rules are validity constraints and are not
  checked — their *value* shapes are (see `docs/design/evidence/rule-enforcement.md`).

**UTF-8 only.** Other encodings are rejected with a typed error rather than silently mis-decoded.
`docs/design/evidence/rule-inventory.md` lists every rule file and whether this library enforces it,
and `rule-enforcement.md` gives the verdict for each rule in the parser, validation and serializer
layers — 54 enforced, 6 partial (the DTD half), 4 deferred (UTF-16, DTD validity) and 2 not
applicable.

## Intended behaviours that surprise people

**Editing does not check namespaces.** Removing a declaration that a subtree relies on succeeds,
moving an element into a scope where its prefix means something else succeeds, and the serializer
never adds a declaration to make the output valid. `validate()` reports all of it; that is the
contract, and the reason edits stay fast and predictable.

**`Document::validate` is the only reporter.** A parsed document is well-formed by construction, but
a document assembled through the API can be inconsistent, and until you validate it nothing will tell
you. (`it_ns_silent_loss` in `docs/design/evidence/probes/` is the probe that pins this.)

**Arena slots are never reclaimed.** `NodeId`s stay valid forever, which is why handles never dangle
and slots are never reused — but it also means `node_count()` only grows, even for detached nodes.
Long-running edit loops on one document should be avoided or split across documents.

**Deep nesting is safe but not free.** Parsing, serialization, `Display` and the clone operations are
all iterative, so no depth can crash the process; a very deep document still costs memory
proportional to its depth.

## Python-specific

**The native module is not the documented surface.** `biodivine_lib_xml_dom._sys` is importable, but
the supported API is the Python package; a test enumerates exactly what is exported from it.

**`NodeContent` is not mirrored.** Rust's content enum has no Python counterpart: `node.kind` plus the
typed accessors (`text()`, `comment()`, …) carry the same information without a discriminator dance.

**No `abi3` wheels yet.** The extension is built for the interpreter maturin finds, so a wheel is
specific to one CPython version (`docs/design/BINDINGS.md` §8).

**Compound edits are not transactional.** Each operation is atomic, but a sequence of them is not;
see [Thread safety](thread-safety.md#what-thread-safe-does-not-mean).
