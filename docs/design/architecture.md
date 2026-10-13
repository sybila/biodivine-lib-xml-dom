# Architecture

How this library is built and why it is built that way. It is written for someone who is about to
change the code: every section states the decision, the reason for it and what it costs. The API
itself is documented in the rustdoc of the `biodivine-lib-xml-dom` crate, which is the place to look
for *what* a method does; this document is about the shape underneath.

## 1. Scope and non-goals

XML 1.0 (fifth edition) and Namespaces in XML 1.0 (third edition), UTF-8 only, with whole-document
manipulation as the goal:

* the tree is in memory, mutable, and shared between threads;
* names are *expanded names* — a local name plus an optional namespace with both the URI and the
  prefix it was written with;
* locally decidable properties are enforced while a document is built, and the remaining
  document-wide properties are reported by one validation pass;
* serialization is lossless for everything the data model can represent.

Explicitly out of scope, because they would change the shape of the library rather than extend it:
streaming or pull parsing, `serde` integration, XPath or XQuery, DTD processing (so no entity
declarations, attribute types or content models), non-UTF-8 encodings, and performance work beyond
"no accidental quadratic behaviour".

## 2. The model: document, arena, handles

A `Document` is a small handle onto shared state. That state owns one arena — a vector of node slots
— behind one reader-writer lock. Nodes refer to each other by index, never by pointer or reference,
and the public handles (`Node`, `Element`) are a document handle, an index and a counted reference
to that slot.

The model diagram shows this relationship; it is the same picture in the crate documentation and in
the book (the drawing is kept in `docs/assets/model.svg` and copied into both by
`scripts/gen_diagrams.py`):

<!-- diagram: model.svg -->
<svg xmlns="http://www.w3.org/2000/svg" width="720" height="300" viewBox="0 0 720 300" role="img" aria-label="A document handle owns a reference to the shared document state; node and element handles are a document handle plus a counted reference to one arena slot.">
  <defs>
    <marker id="arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
      <path d="M 0 0 L 10 5 L 0 10 z" fill="#4b5563"/>
    </marker>
  </defs>
  <style>
    .box { fill: #f8fafc; stroke: #475569; stroke-width: 1.2; }
    .inner { fill: #eef2f7; stroke: #94a3b8; stroke-width: 1; }
    .label { font-family: system-ui, -apple-system, "Segoe UI", sans-serif; font-size: 13px; fill: #0f172a; }
    .small { font-family: system-ui, -apple-system, "Segoe UI", sans-serif; font-size: 11px; fill: #475569; }
    .title { font-family: system-ui, -apple-system, "Segoe UI", sans-serif; font-size: 14px; font-weight: 600; fill: #0f172a; }
    .edge { stroke: #4b5563; stroke-width: 1.4; fill: none; marker-end: url(#arrow); }
  </style>
  <text class="title" x="24" y="26">Your program</text>
  <rect class="box" x="24" y="40" width="196" height="88" rx="8"/>
  <text class="label" x="40" y="66">Document</text>
  <text class="small" x="40" y="86">a cheap, shareable handle</text>
  <line x1="40" y1="96" x2="204" y2="96" stroke="#cbd5e1"/>
  <text class="label" x="40" y="116">Node, Element</text>
  <rect class="box" x="24" y="150" width="196" height="60" rx="8"/>
  <text class="label" x="40" y="174">one node reference</text>
  <text class="small" x="40" y="194">counted; dropped when unused</text>
  <rect class="box" x="300" y="40" width="396" height="170" rx="8"/>
  <text class="title" x="320" y="66">Document state</text>
  <text class="small" x="320" y="86">one reader-writer lock per document</text>
  <rect class="inner" x="320" y="98" width="356" height="94" rx="6"/>
  <text class="label" x="336" y="122">arena: one slot per node</text>
  <text class="small" x="336" y="142">payload, parent index, external count</text>
  <text class="small" x="336" y="160">free slots are reused after a collection</text>
  <text class="small" x="336" y="180">the root slot is kept alive by the document</text>
  <path class="edge" d="M 220 84 C 260 84, 262 84, 298 84"/>
  <text class="small" x="228" y="76">owns a reference</text>
  <path class="edge" d="M 220 180 C 260 180, 262 140, 298 130"/>
  <text class="small" x="228" y="200">keeps its slot alive</text>
  <text class="small" x="24" y="248">Internal links between nodes are slot indices: they cost nothing and cannot form a cycle.</text>
  <text class="small" x="24" y="268">Reachable nodes are those reachable from the document root or from a node reference.</text>
  <text class="small" x="24" y="288">Everything else is reclaimed by the collector, and its slot is handed out again.</text>
</svg>
<!-- /diagram -->

Three properties follow, and they are the reason for the design:

* **No reference cycles.** A parent stores the indices of its children and a child stores the index
  of its parent; neither keeps the other alive. Dropping every handle frees the whole tree.
* **One lock, therefore no lock ordering.** Every public operation acquires the document's lock
  once, at the outermost frame, and never re-acquires it. Internal code takes `&Arena`/`&mut Arena`
  and cannot lock at all, so a check-then-act sequence cannot be interleaved by another thread. In
  debug builds a re-entrancy guard turns a violation into a panic instead of a hang.
* **Atomic edits.** Because all structural edits are validated and applied inside one exclusive
  borrow, an operation such as "append this child if it would not create a cycle"
  either happens completely or not at all.

What it costs: node payloads live in a vector, so an index is not a stable pointer across a
re-allocation (the vector itself is only ever grown, never re-ordered, so *indices* are stable),
and a detached node still occupies its slot until the collector reclaims it.

## 3. Lifetime of a node, and the collector

A node lives while something outside the document refers to it. There are exactly two kinds of
external reference:

1. **Handles.** `Node` and `Element` own a counted reference — an atomic count per slot. Cloning a
   handle increments it, dropping one decrements it, and neither operation touches the arena lock,
   which is what makes handles cheap enough to pass around freely.
2. **The document root.** The arena itself keeps the root alive, because a document that forgot its
   own content the moment the caller stopped holding a handle would be useless.

Everything else — parent links, child lists, a subtree that a copy operation has just built — is an
internal reference and does not count. A node that nothing refers to is reclaimed by a mark-and-sweep
collection:

* the mark phase starts from every slot with a positive count and from the root, and follows both
  parent and child links, so a handle to a deep node also keeps its ancestors reachable;
* the sweep phase drops the payload of every unmarked slot and pushes its index onto a free list.

Correctness rests on one invariant: **a slot can only become externally referenced while the arena
lock is held**, and the only way to create a handle is an arena method called inside that critical
section. The collector holds the write lock, so it cannot miss a reference created before it started;
a reference *dropped* while it runs may go unnoticed, in which case the node survives one extra
cycle. Cloning a handle only raises an already-positive count, and every access to node payload goes
through the lock, so the counter needs no ordering guarantees of its own beyond atomicity.

The capacity policy is deliberately simple and configurable through `GcConfig`:

| setting | default | why |
| --- | --- | --- |
| initial capacity | 1024 slots | a document that fits needs no growth at all |
| growth factor | 2 | amortised doubling keeps the number of grow steps logarithmic |
| minimum free fraction | 10% | a document that is permanently almost full would otherwise collect before every allocation |

Collection is automatic (an allocation that finds the arena full collects, and grows if collection
did not free enough) and manual (`Document::collect_garbage`). Two things are permanent: the arena
never shrinks or compacts, and the name interner only ever grows. Both are accepted in exchange for
stable indices and cheap interning; a document that repeatedly builds and discards huge subtrees
should be recycled as a whole rather than expected to shrink.

## 4. Concurrency

The design goal is that a document can be shared between threads *without* the caller having to think
about it, and that no sequence of calls can deadlock. That is achieved structurally rather than by
convention:

* one lock per document, taken at the outermost frame of every public method;
* no operation holds two document locks — cross-document copies snapshot the source under a read
  lock, release it, and only then take the write lock of the target;
* the arena's own invariants are maintained inside the exclusive borrow, so a partially linked state
  is unrepresentable outside it;
* failure paths return before anything is changed, so a rejected operation leaves the document
  exactly as it was.

Handles are `Send + Sync`, and so is `Document`; the counted reference is atomic. The Python
bindings release the interpreter lock around whole-document work (parsing, serializing, validating,
copying, collecting), so a long operation in one thread does not freeze others.

### 4.1 Cancellation

The operations that take time proportional to the document (parsing, serialization, validation,
deep copies and explicit garbage collection) are *cooperatively cancellable* through the
[`cancel_this`](https://crates.io/crates/cancel-this) crate. A trigger is registered on the current
thread (`cancel_this::on_trigger` and friends), the operation consults it periodically, and a
cancelled operation returns `XmlError::Cancelled` rather than running to completion. There is no
cancellation token in any signature: the trigger lives in a thread-local scope. Consulting it costs
a thread-local borrow and a dynamic call, so the walks do it once every 1024 steps rather than on
every node (`src/cancellation.rs`).

A cancelled operation leaves the document as it was, which constrains where the checks may go:

* parsing and validation build their result privately, so stopping them discards nothing visible;
* a deep copy first reads the source into a snapshot and then builds the copy from it; both phases
  are cancellable, and a copy cancelled while it is being built frees the nodes it allocated, so a
  cancelled copy creates nothing;
* the collector's mark phase is read-only and cancellable, while the sweep is not: stopping the sweep
  part-way would free half of an unreachable subtree and leave the other half linked to freed slots;
* serialization streams its output, so a cancelled write leaves the output written so far — the one
  observable effect, and it is documented on the write methods.

The collector invoked *implicitly* by the allocator is not cancellable, because allocation is
infallible. Cheap operations (single-node edits, shallow copies) and the `Display` implementations,
which cannot report an error, never consult the triggers.

The Python bindings build on this without running Python code under a document lock; see
`bindings.md`.

## 5. Errors

Two error styles coexist on purpose:

* **`XmlError` and `XmlResult`** for everything that can fail for a *logical* reason: malformed
  input, a name that is not an NCName, a cycle, attaching a node to a foreign document.
* **Panicking twins** for the ergonomic spelling of the same operations (`set_root` next to
  `set_root_checked`). They are marked `#[track_caller]`, so the panic points at the call site, and
  the documentation states the condition. Locally decidable properties are usually not errors at
  all: `NCName`, `Text`, `Comment`, `CData`, `PiTarget` and `PiData` can only be constructed from
  valid input, so a violated rule is unrepresentable rather than reportable.

## 6. Namespaces

Names are expanded names: the local name plus the namespace (URI *and* prefix). Consequences that
the whole API follows:

* two spellings of the same expanded name compare equal, so prefix juggling disappears;
* attributes are keyed by expanded name, which makes duplicate attributes unrepresentable;
* editing never adds, removes or rewrites a declaration. Moving an element into a scope where its
  prefix means something else succeeds silently; `Document::validate` reports the resulting
  inconsistency, listing every problem at once.

This is a deliberate split: editing stays predictable and fast, validation is where correctness is
judged. The alternative — repairing the tree behind the caller's back — hides the change and makes
edits expensive.

## 7. Integrity: local versus whole-document

| kind of property | when it is checked | how it fails |
| --- | --- | --- |
| locally decidable (name, text, comment, CDATA, PI shape, attribute duplication) | at construction | typed error, or a type that cannot represent the invalid value |
| document-wide (namespace scope, `xml:id` uniqueness, `xml:lang`/`xml:space` values) | `Document::validate` | a collection of problems, one per rule |

Detached nodes are a normal state, not an error: node-local rules are checked for them too, so a
subtree can be prepared and validated before it is attached, while document-wide rules are judged
over the tree below the root.

## 8. Module layout

The crate keeps a flat public surface with one canonical location per item, and a private tree that
is organized by feature so that the file tree tells you where something lives:

| path | contents |
| --- | --- |
| `src/lib.rs` | crate documentation, module declarations, the re-export list |
| `src/document.rs`, `src/document/io.rs` | `Document` — identity, lock, GC and capacity API; parsing and serialization entry points |
| `src/arena.rs`, `src/arena/{slot,snapshot,gc,config}.rs` | the arena, its slots, the collector and the capacity policy |
| `src/ref_count.rs` | the counted external reference |
| `src/node_id.rs` | the arena slot id (`NodeId`) and the node-count bound (`MAX_NODES`) |
| `src/node.rs`, `src/element.rs` | the two public handles |
| `src/namespace.rs`, `src/qualified_name.rs` | the two name types: a namespace with its prefix, and an expanded name |
| `src/interner.rs` | the per-document name interner |
| `src/error.rs` | the error type (`XmlError`) and the `XmlResult` alias |
| `src/cancellation.rs` | the periodic cancellation check used by the long operations |
| `src/io.rs`, `src/io/{parse,write}.rs` | the parser and the serializer implementations |
| `src/validation.rs` | whole-document validation |
| `src/xml_spec.rs`, `src/xml_spec/*` | specification types, predicates, validation rules and the rule set |
| `src/documentation.rs` | this document, the bindings map and the rule coverage |

Only the crate root, `xml_spec` and `documentation` are public; everything else is an internal
module, which is why the documentation never names one.

## 9. Testing

* **Unit tests** next to the code that they pin, including the arena's structural invariants and the
  collector's reference-count bookkeeping (debug builds also check those invariants after every
  collection).
* **Integration tests** (`tests/`) for the public behaviour: structure and editing, namespaces,
  parsing and serialization round trips, validation, cloning, concurrency, garbage collection.
* **Property tests** (`proptest`) for the invariants that should hold for *every* input: serializing
  and re-parsing, editing sequences, and round trips through mutated input.
* **Examples** in `examples/` are real programs. `cargo test` compiles them, `make examples` and
  `scripts/verify.sh` run them, and the Python examples of the book are run by the Python test
  suite, so no chapter can show code that no longer compiles or runs.
* **Documentation tests**: every rustdoc example is compiled and executed by `cargo test`.

`scripts/verify.sh` runs all of it, plus formatting, clippy in both profiles, the declared minimum
Rust version, the documentation build and the repository checkers.

## 10. Documentation strategy

| artifact | audience | source of truth |
| --- | --- | --- |
| rustdoc (crate, `xml_spec`, `documentation`) | Rust users and contributors | the code |
| `xml_spec::rule_set` | anyone checking a rule against the specification | `specification/rules/*.md` |
| rule coverage (section 10.1) | anyone asking what the library deliberately does not do | `scripts/gen_rule_coverage.py` |
| the documentation book (`docs/book/`) | people learning the library, in Rust or Python | its own chapters plus the executed examples |
| the Python API reference | Python users | the docstrings in the bindings and the Python package |
| `docs/design/` | contributors | this file, `build.md`, `bindings.md`, `rule-coverage.md` |

### 10.1 Rule summaries and coverage

`specification/rules/` holds one markdown summary per specification rule. Those files are the single
source of truth: `xml_spec::rule_set` reproduces each of them as a module (generated by
`scripts/gen_rule_set.py`, whose `--check` mode fails when the module list drifts), and code that
enforces a rule links to that module, so a reader can jump from a method to the specification text.

The summaries describe the specification, not the implementation. Which rules this library actually
acts on — and where — is the generated rule coverage table, whose classification is maintained by
hand in `scripts/gen_rule_coverage.py`; the script fails if a rule has no classification or if a rule
that the library is expected to implement has no verdict.
