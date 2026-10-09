# Long-term implementation plan

Companion to `REVIEW.md` (what is wrong) — this document is *how it will be fixed*. It is the
reference for G2–G7 in `GOALS.md` and may be revised as implementation reveals new constraints;
revisions are recorded in `GOALS.md` via `submit_plan` with a `rationale`.

Contents: [1 Scope](#1-scope) · [2 Target architecture](#2-target-architecture) ·
[3 Concurrency, deadlock and panic safety](#3-concurrency-deadlock-and-panic-safety) ·
[4 Error model and the `_checked` convention](#4-error-model-and-the-checked-convention) ·
[5 Public API sketch](#5-public-api-sketch) · [6 Namespaces](#6-namespaces) ·
[7 Cloning and cross-document copies](#7-cloning-and-cross-document-copies) ·
[8 I/O](#8-io) · [9 Validation](#9-validation) · [10 Workspace and packaging](#10-workspace-and-packaging) ·
[11 Documentation](#11-documentation) · [12 Test strategy and seams](#12-test-strategy-and-seams) ·
[13 Goal ordering](#13-goal-ordering) · [14 Risks and open questions](#14-risks-and-open-questions)

## 1. Scope

* XML 1.0 (fifth edition) + Namespaces in XML 1.0 (third edition), UTF-8 only.
* No DTD processing, no validity checking (AGENTS.md). Entity policy: see §1.1.
* Breaking API changes are allowed (unreleased crate).
* Non-goals: streaming/`serde` integration, XPath, entity/DTD support, non-UTF-8 encodings,
  performance micro-optimisation. The library is for *document manipulation*, not bulk parsing.

### 1.1 Entity scope (decided in REVIEW §4.1)

| input | behaviour |
| --- | --- |
| `&amp; &lt; &gt; &apos; &quot;` | expanded, in content and attribute values |
| `&#NN;` / `&#xHH;` | expanded, and the resulting character must be a legal XML character |
| any other `&name;` | typed error (`XmlError::UndeclaredEntityReference`) — never a panic |
| `<!DOCTYPE …>` | read and ignored (no DTD processing), as today |
| declared encoding ≠ UTF-8 | typed error (`XmlError::UnsupportedEncoding`) instead of silent mis-decoding |

93 of the 194 rule files are out of scope for exactly these reasons; they are listed individually
in `docs/design/evidence/rule-inventory.md` with layer `X`.

## 2. Target architecture

### 2.1 Storage: one arena per document

```text
Document ── Arc<DocumentInner>
                    │
                    └─ RwLock<Arena>                 <-- ONE lock for the whole document
                          ├─ nodes:  Vec<NodeData>   <-- all node payloads
                          ├─ root:   Option<NodeId>
                          └─ names:  Interner        <-- Arc-dedup for names/namespaces

NodeId = u32 (opaque newtype, Copy)   Node { doc: Document, id: NodeId }
```

* `NodeId` is a **plain monotonically increasing index** into `nodes`. Slots are never reused and
  never freed, so an ID can never dangle or alias a different node (no ABA problem, no generation
  counters, no tombstones). The cost is that the arena keeps one slot per node ever created in that
  document; this is documented on `Document` and is the deliberate trade-off for making every
  handle permanently valid. A future `Document::shrink()` could compact the arena, but only by
  invalidating detached handles, so it is explicitly **not** part of this plan.
* `NodeData` is the enum that removes the reference cycle of D4 — parents and children store
  `NodeId`, never a handle:

```rust
enum NodeData {
    Element(ElementData),
    Text(Text),                       // xml_spec::Text  (validated, Arc<str> backed)
    Comment(Comment),                 // xml_spec::Comment
    CData(CData),                     // xml_spec::CData
    ProcessingInstruction { target: PiTarget, data: PiData },
}

struct ElementData {
    name: QualifiedName,                                  // Arc-shared
    attributes: BTreeMap<QualifiedName, Arc<str>>,        // uniqueness by expanded name
    namespace_declarations: BTreeMap<Option<NCName>, Option<Namespace>>,
    parent: Option<NodeId>,
    children: Vec<NodeId>,
}
```

* `Arena` owns every structural invariant. All mutation goes through `Arena` methods that update
  *both* directions (`parent` of the child and `children` of the parent) inside a single `&mut self`
  borrow, so a half-linked state is unrepresentable outside the arena module.
* For O(1) removal, `ElementData` additionally stores the child's index in its parent's `children`
  vector (`ChildLink { parent: NodeId, index: u32 }` semantics) with an internal helper that keeps
  indices consistent on insert/remove; `Vec::remove` keeps child order stable (siblings keep
  document order) and the index bookkeeping is confined to `arena.rs`.

### 2.2 Handles

| type | invariant | notes |
| --- | --- | --- |
| `Node` | any node kind | `Clone` = handle copy; `PartialEq`/`Hash` by `(document identity, NodeId)`; `Send + Sync` |
| `Element` | the target is `NodeData::Element` | newtype over `Node`; only constructible via `Node::as_element()`/`Document::create_element()` |
| `Namespace` | unchanged from today | `Arc<NamespaceData>` |
| `QualifiedName` | unchanged from today | `Arc<QualifiedNameData>` |

`Element`'s invariant is enforced by construction, so `Element` methods never need to handle
"this is actually a text node" (the `rust-api-type-system-design` "parse, don't validate"/newtype
guidance). `Node::as_element()` returns `Option<Element>`; `Node::kind()` returns a `NodeKind`
enum for branching.

### 2.3 Module layout

```text
src/lib.rs            crate docs + re-exports
src/xml_spec.rs       spec types & predicates (+ src/xml_spec/validation.rs, src/xml_spec/rules.rs)
src/error.rs          XmlError, XmlResult
src/arena.rs          Arena, NodeId, NodeData, ElementData, internals (pub(crate))
src/interner.rs       per-document Arc interning for names/namespaces
src/document.rs       Document, DocumentInner, with_read/with_write help
src/node.rs           Node, NodeKind, traversal, structural editing
src/element.rs        Element (element-specialised handle)
src/validation.rs     Document::validate  (orchestration over xml_spec::validation)
src/io/mod.rs         re-exports
src/io/parse.rs       parser
src/io/write.rs       serializer + WriteOptions
examples/*.rs         (moved from src/main.rs)
```

## 3. Concurrency, deadlock and panic safety

### 3.1 The invariant that makes deadlock impossible

> **Locking invariant L.** Exactly one lock exists per document. It is acquired only in the
> outermost frame of a public `Document`/`Node`/`Element` method, is held for the duration of that
> method, and is *never* acquired again while held. Internal helpers accept `&Arena` / `&mut Arena`
> and therefore cannot lock at all.

Consequences:

* There is nothing to order, so there is no lock-ordering deadlock.
* Re-entrancy is impossible *by construction*: a helper that wanted to lock would have to obtain a
  `Document` from a handle and call a public method — the code review checklist plus the
  `pub(crate)` boundary prevents this, and it is additionally checked at runtime in debug builds
  (see §3.3).
* Operations that need two nodes (attach, move, copy, replace) are performed under a single
  `&mut Arena`, so they are atomic — this directly fixes D3: the ancestor walk and the pointer
  update happen inside the same critical section, and no other thread can interleave.
* Readers use `RwLock::read`; writers use `RwLock::write`. `parking_lot::RwLock` is used for
  consistent performance, the absence of poisoning, and `read_recursive` availability if a
  later goal ever needs it — no custom atomics, no `unsafe`.

Implementation shape:

```rust
impl DocumentInner {
    pub(crate) fn read<R>(&self, f: impl FnOnce(&Arena) -> R) -> R { f(&self.arena.read()) }
    pub(crate) fn write<R>(&self, f: impl FnOnce(&mut Arena) -> R) -> R { f(&mut self.arena.write()) }
}
// e.g.
pub fn local_name(&self) -> NCName { self.node.doc.inner.read(|a| a.element(self.node.id).name.local_name().clone()) }
```

Closure-based access (rather than returning guards) makes it impossible to leak a guard into
user-visible types, which is what would otherwise turn a read into a `Send`-unsafety and a
deadlock hazard.

### 3.2 Panic safety

* A mutation is one `with_write` call. Inside it, the `Arena` is available only as `&mut Arena`
  through a single exclusive borrow, so the borrow checker guarantees no aliasing.
* Every arena mutation is written to be **panic-free**: no `unwrap` on user input (all validation
  happens before the mutation), no indexing without a bounds-checked helper, and no allocation that
  can be observed half-done. Ordering rule: validate *everything* first, then perform only
  infallible pointer/index writes. A panic can therefore only occur for a genuine bug, and even
  then `parking_lot` has no poisoning, so a subsequent operation sees the arena in the state left
  by the last completed mutation (all-or-nothing at the operation granularity).
* `unsafe` is not used anywhere in the crate. A crate-level `#![deny(unsafe_code)]` will be added
  in G2, making this machine-checked.

### 3.3 Debug-only re-entrancy detector

In debug builds a `thread_local! { static LOCK_DEPTH: Cell<u32> }` is incremented on entry to
`with_read`/`with_write` and panics with a clear message if it is already non-zero. This turns the
"no re-entrancy" claim into something a test can assert: the whole test suite, plus a dedicated
stress test, runs with the detector enabled. `Document` and all handles are also asserted
`Send + Sync` with a compile-time `fn assert_send_sync<T: Send + Sync>()` test.

### 3.4 What is *not* used, and why

* **No `loom`.** Loom models custom atomics and lock-free protocols. Here there is exactly one
  `parking_lot::RwLock` with no custom synchronisation, so loom would provide no additional signal.
  Instead: a multi-threaded stress test (readers + writers + structural edits) and the debug
  re-entrancy detector. This is stated up front rather than promised and skipped.
* **No `Send`-based `Element` in `py-sys` without an audit.** PyO3 objects will be audited for
  `Send`/`Sync` in G5 (`rust-python-pyo3-maturin`).
* **Miri** is attempted in G7 on a targeted subset; if `parking_lot` is unsupported under Miri the
  result is recorded as "not applicable", not claimed as passing. There is no `unsafe` code for
  Miri to validate beyond dependencies.

## 4. Error model and the `_checked` convention

### 4.1 `XmlError`

Replaces the stringly typed enum of D10 with typed variants, roughly:

```rust
pub enum XmlError {
    // construction / spec violations (locally decidable)
    InvalidName(String), InvalidText(String), InvalidComment(String), InvalidCData(String),
    InvalidPi { .. }, InvalidNamespace { .. }, ReservedPrefix { .. },
    // document-level well-formedness, raised while parsing
    UndeclaredPrefix(String), DuplicateAttribute(QualifiedName), MultipleRootElements,
    ContentOutsideRoot, UnsupportedEncoding(String), UnsupportedXmlVersion(String),
    UndeclaredEntityReference(String), InvalidCharacterReference(String),
    MalformedXml(String), Utf8(String),
    // API misuse
    ForeignDocument, CycleDetected, NodeNotFound(NodeId), NotAnElement(NodeId),
    RootAlreadySet, MissingRoot, CannotAttachRoot,
    // I/O
    Io(std::io::Error),
}
```

`XmlError` keeps `Display`/`Error` via `thiserror`, and gains structured payloads so the PyO3 layer
can map categories onto Python exception classes (`XmlSyntaxError`, `XmlNamespaceError`,
`XmlDocumentError`, `XmlIoError`).

### 4.2 The `_checked` / panicking split — precise interpretation

Requirement (1) motivates `_checked` variants by multi-lock acquisition failure. With exactly one
document-level lock that failure mode **does not exist** (§3.1), so the split is reinterpreted:

* `_checked` returns `XmlResult<()>` and covers **logical** failures only: creating a cycle
  (`XmlError::CycleDetected`), attaching a node from another document (`ForeignDocument`),
  attaching a node that carries a namespace the target document cannot represent (never today —
  namespaces are immutable values, so this variant is not needed), attaching a root twice,
  replacing a node that has no parent, etc.
* The ergonomic variant has the unsuffixed name, calls the `_checked` one, and panics with
  `panic!("<operation> failed: {error}")` when it returns `Err`. Its rustdoc carries a
  `# Panics` section listing the same conditions (AGENTS.md requirement).
* Value construction (names, text, comments, CDATA, PI, namespaces, qualified names) stays
  `Result`-returning, because requirement (4)(1) asks for construction to be the enforcement point
  and a panicking constructor would be a footgun. There is no `_checked` twin for those.

Naming rule applied consistently:

| operation | panicking (ergonomic) | fallible |
| --- | --- | --- |
| append child | `append_child` | `append_child_checked` |
| insert child at index | `insert_child` | `insert_child_checked` |
| insert before/after sibling | `insert_before` / `insert_after` | `…_checked` |
| replace a node | `replace_with` | `replace_with_checked` |
| detach | `detach` | `detach_checked` |
| set the document root | `set_root` | `set_root_checked` |
| copy a subtree into another document | `deep_clone_into` | `deep_clone_into_checked` |

## 5. Public API sketch

Deliberately close to the current naming so the diff stays reviewable, but arena-shaped.
`…` marks an abbreviated body; every item below carries full rustdoc with `# Errors` / `# Panics`
where applicable.

```rust
// ---------- documents -----------------------------------------------------------------
impl Document {
    pub fn empty() -> Self;
    pub fn from_root(root: Element) -> XmlResult<Self>;      // convenience

    pub fn root(&self) -> Option<Element>;
    pub fn set_root(&self, root: Element);                    // panics on failure
    pub fn set_root_checked(&self, root: Element) -> XmlResult<()>;
    pub fn clear_root(&self) -> Option<Element>;

    // node factories (all validate immediately -> requirement (4)(1))
    pub fn create_element(&self, name: QualifiedName) -> Element;
    pub fn create_element_with(&self, name: QualifiedName) -> ElementBuilder; // attrs/ns in one go
    pub fn create_text(&self, text: impl Into<String>) -> XmlResult<Node>;
    pub fn create_comment(&self, text: impl Into<String>) -> XmlResult<Node>;
    pub fn create_cdata(&self, text: impl Into<String>) -> XmlResult<Node>;
    pub fn create_processing_instruction(&self, target: impl Into<String>, data: impl Into<String>) -> XmlResult<Node>;

    pub fn node_count(&self) -> usize;                        // arena slots used
    pub fn validate(&self) -> Result<(), Vec<XmlValidationError>>;
    pub fn is_valid(&self) -> bool;
    pub fn nodes(&self) -> Vec<Node>;                         // all nodes, incl. detached

    pub fn parse_str(&self, xml: &str) -> XmlResult<Node>;    // into an existing document
}

impl PartialEq for Document { /* Arc::ptr_eq */ }
impl fmt::Display for Document { /* serialize */ }

// ---------- nodes ---------------------------------------------------------------------
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Node { doc: Document, id: NodeId }   // Clone = handle copy

impl Node {
    pub fn document(&self) -> Document;
    pub fn kind(&self) -> NodeKind;                          // Element | Text | Comment | CData | Pi
    pub fn as_element(&self) -> Option<Element>;
    pub fn is_attached(&self) -> bool;                       // reachable from the document root
    pub fn parent(&self) -> Option<Node>;
    pub fn children(&self) -> Vec<Node>;
    pub fn child_elements(&self) -> Vec<Element>;
    pub fn first_child(&self) -> Option<Node>;
    pub fn last_child(&self) -> Option<Node>;
    pub fn next_sibling(&self) -> Option<Node>;
    pub fn previous_sibling(&self) -> Option<Node>;
    pub fn index_in_parent(&self) -> Option<usize>;
    pub fn descendants(&self) -> Vec<Node>;                  // pre-order snapshot
    pub fn element_children(&self) -> Vec<Element>;

    // typed content accessors (None when the kind does not match)
    pub fn text(&self) -> Option<Text>;                      // xml_spec::Text
    pub fn comment(&self) -> Option<Comment>;
    pub fn cdata(&self) -> Option<CData>;
    pub fn processing_instruction(&self) -> Option<(PiTarget, PiData)>;

    // structural editing -- atomic under the single document lock
    pub fn detach(&self);                                    // panics on failure
    pub fn detach_checked(&self) -> XmlResult<()>;
    pub fn append_child(&self, child: impl Into<Node>);
    pub fn append_child_checked(&self, child: impl Into<Node>) -> XmlResult<()>;
    pub fn insert_child(&self, index: usize, child: impl Into<Node>);            // panics
    pub fn insert_child_checked(&self, index: usize, child: impl Into<Node>) -> XmlResult<()>;
    pub fn insert_before(&self, sibling: impl Into<Node>);
    pub fn insert_before_checked(&self, sibling: impl Into<Node>) -> XmlResult<()>;
    pub fn insert_after(&self, sibling: impl Into<Node>);
    pub fn insert_after_checked(&self, sibling: impl Into<Node>) -> XmlResult<()>;
    pub fn replace_with(&self, replacement: impl Into<Node>) -> Node;            // panics
    pub fn replace_with_checked(&self, replacement: impl Into<Node>) -> XmlResult<Node>;
    pub fn append_text(&self, text: impl Into<String>) -> XmlResult<()>;         // validate+append
    pub fn append_comment(&self, text: impl Into<String>) -> XmlResult<()>;

    pub fn belongs_to(&self, doc: &Document) -> bool;
    pub fn ptr_eq(&self, other: &Node) -> bool;

    // clones
    pub fn deep_clone(&self) -> Node;                        // same document, detached copy
    pub fn deep_clone_into(&self, target: &Document) -> XmlResult<Node>;
    pub fn deep_clone_into_checked(&self, target: &Document) -> XmlResult<Node>;
    pub fn shallow_clone(&self) -> Node;
    pub fn shallow_clone_into(&self, target: &Document) -> XmlResult<Node>;
}

impl fmt::Debug for Node { /* <html:body> @3 in Document#1 */ }
impl fmt::Display for Node { /* serialize this subtree (no namespace magic) */ }

// ---------- elements ------------------------------------------------------------------
pub struct Element(Node);   // newtype: the target IS an element

impl Element {
    pub fn qualified_name(&self) -> QualifiedName;
    pub fn local_name(&self) -> NCName;
    pub fn namespace(&self) -> Option<Namespace>;
    pub fn set_qualified_name(&self, name: QualifiedName);

    // attributes: BTreeMap<QualifiedName, _> -> duplicates unrepresentable
    pub fn attributes(&self) -> BTreeMap<QualifiedName, Arc<str>>;
    pub fn attribute(&self, name: &QualifiedName) -> Option<Arc<str>>;
    pub fn set_attribute(&self, name: QualifiedName, value: impl Into<String>);   // overwrites
    pub fn remove_attribute(&self, name: &QualifiedName) -> Option<Arc<str>>;
    pub fn clear_attributes(&self);
    pub fn has_attribute(&self, name: &QualifiedName) -> bool;
    pub fn attribute_local(&self, local: &NCName) -> Option<Arc<str>>;   // no-namespace attribute

    // convenience views over attributes
    pub fn xml_lang(&self, inherit: bool) -> Option<...>;
    pub fn xml_id(&self) -> Option<NCName>;

    pub fn add_child_element(&self, child: Element);          // backwards-compatible alias
    pub fn element_children(&self) -> Vec<Element>;

    pub fn node(&self) -> Node;
    pub fn into_node(self) -> Node;
}
impl Deref for Element { type Target = Node; }   // all Node methods available on Element
```

`impl Into<Node>` for `Node`/`Element` (and `&Node`/`&Element`) means `append_child` accepts
either handle type or a reference to one — the ergonomics requirement (3) asks for.

## 6. Namespaces

### 6.1 The contract (requirement (3): "no magic")

* Element/attribute names are always expanded names (`QualifiedName` = local NCName + optional
  `Namespace{uri, prefix}`). Prefixes are *stored*, not inferred.
* Edits never touch namespace declarations: `append_child`, `detach`, `replace_with`,
  `set_qualified_name` and friends do not add, remove or rewrite `xmlns:*`. Removing a declaration
  that children rely on is allowed and silent.
* Serialization writes exactly the declarations present on the nodes and the prefixes present in
  the names. It never invents a declaration.
* `Document::validate()` is the only place that reports the resulting inconsistencies
  (§9), as instructed by the task.

### 6.2 Namespace API

```rust
impl Element {
    /// Declarations written on *this* element only (no inheritance).
    pub fn namespace_declarations(&self) -> BTreeMap<Option<NCName>, Option<Namespace>>;
    /// All bindings visible to this element, innermost first (the "in scope" set).
    pub fn namespaces_in_scope(&self) -> Vec<(Option<NCName>, Option<Namespace>)>;

    /// Declare/overwrite a binding (prefix carried by the `Namespace`; `None` = default ns).
    /// Overwrites any previous binding for that prefix -- documented default behaviour,
    /// per requirement (4)(1). Use `declare_namespace_checked` to get an error instead.
    pub fn declare_namespace(&self, ns: Namespace);
    pub fn declare_namespace_checked(&self, ns: Namespace) -> XmlResult<()>;
    pub fn undeclare_default_namespace(&self);                     // xmlns=""
    pub fn remove_namespace_declaration(&self, prefix: Option<&NCName>) -> Option<...>;

    /// Resolve a prefix (None = default namespace) using the in-scope bindings.
    pub fn resolve_prefix(&self, prefix: Option<&NCName>) -> Option<Namespace>;
    pub fn get_namespace(&self, prefix: Option<&NCName>) -> Option<Namespace>;   // kept (recursive today -> iterative)

    /// QName string -> expanded name, using this element's scope.
    pub fn resolve_qualified_name(&self, qname: &str) -> XmlResult<QualifiedName>;   // element rules
    pub fn resolve_attribute_name(&self, qname: &str) -> XmlResult<QualifiedName>;   // attrs ignore default ns
}
```

Behaviour preserved from today (see REVIEW §3): `xml` is pre-bound to
`http://www.w3.org/XML/1998/namespace` without a declaration, `xmlns` may never be used as a
prefix, an empty default declaration stops inheritance, and unprefixed attributes never take the
default namespace. `get_namespace` becomes an iterative loop over the arena instead of recursion
over `Arc` handles (same semantics, no stack depth limit).

The one deliberate behaviour change: same-prefix re-declaration on the same element **overwrites**
(documented) instead of erroring, because requirement (4)(1) prescribes "documented default
behaviour (e.g. new attribute declaration overwrites old attribute declaration)". The erroring
behaviour remains available as `declare_namespace_checked`. Parsing still rejects duplicate
`xmlns:*` on one element, because that is a Namespaces "Attributes Unique" violation, i.e. a
locally decidable well-formedness error (`rule.namespace-usage.attributes-unique-expanded-name`).

### 6.3 Validation-time namespace rules

`Document::validate()` reports (as one issue per occurrence, not fail-fast):

* a prefixed element/attribute name whose prefix is not bound in scope (`prefix-declared`,
  `prefix-declaration-scope`);
* an element name with a namespace whose *prefix* does not match the binding in scope — i.e. the
  name says `ex:foo` while `ex` is bound to a different URI, or the name has a default namespace
  while the in-scope default declaration differs (`default-namespace-scope`);
* an unprefixed element name whose stored namespace is non-empty while no default declaration
  provides it;
* an attribute whose expanded name has a namespace but no prefix (attributes must be prefixed);
* declarations that are not spec-legal (`xml`/`xmlns` reserved bindings).

## 7. Cloning and cross-document copies

| operation | semantics |
| --- | --- |
| `clone()` (Rust `Clone`) | copies the **handle** — the result points at the same arena node; cheapest possible |
| `shallow_clone()` | new detached node in the same document: same name, attributes, declarations (text/comment/CDATA/PI: same content); **no** children |
| `deep_clone()` | new detached subtree in the same document, children cloned recursively |
| `deep_clone_into(&target)` | new detached subtree in `target`, recursively, with all names/namespaces **re-interned into the target document** |
| `shallow_clone_into(&target)` | as `shallow_clone`, but in `target` |

Cross-document rules:

* attaching a node whose `Document` differs from the parent's is an error
  (`XmlError::ForeignDocument`) and never silently retargets;
* `deep_clone_into` is the sanctioned way to move a tree between documents;
* the copy is *structural*: no namespace declarations are added or removed, because that is the
  target document's business and validation will report any inconsistency (§6.1).

`Arc` dedup / interning in the target document: `DocumentInner` owns an
`Interner { namespaces: HashMap<Namespace, Namespace>, names: HashMap<QualifiedName, QualifiedName> }`
keyed by value (not by pointer). Both `Namespace` and `QualifiedName` implement `Hash`/`Eq` on
value, so:

* `Interner::intern_namespace(ns) -> Namespace` returns the already-present value-equal `Arc`
  instead of inserting a duplicate;
* `Interner::intern_name(qn) -> QualifiedName` likewise for qualified names (keys derived from the
  interned namespaces so a document converges on one `Arc` per distinct expanded name);
* the same interner is used by `create_element`, `set_attribute`, `declare_namespace` and by both
  `*_clone_into` variants, so a subtree copied from document A into document B shares nothing with
  A but is fully deduplicated *within* B;
* interning is a pure optimisation: equality is value-based, so a name from a foreign document
  compares equal to an interned one and behaves identically — nothing in the API depends on
  pointer identity except `Document`/`Node` equality (which is intentional).

## 8. I/O

### 8.1 Parser (`src/io/parse.rs`), replacing D1/D8/D9

Built on `quick-xml` event streaming, but the library owns every decision:

1. **Never panic.** No `unimplemented!`/`unwrap` on input. `Event::GeneralRef` → expand if
   predefined, otherwise `XmlError::UndeclaredEntityReference`. All quick-xml errors are mapped to
   typed `XmlError`s with position information where available.
2. **Well-formedness enforced locally while parsing:** tag nesting/matching
   (`elements-nest-properly`, `end-tag-must-match-start-tag`, `every-start-tag-must-have-end-tag`),
   exactly one root element (`single-root-element`), no non-whitespace content outside the root
   (`document-production`), duplicate expanded attribute names (`unique-name-in-tag`,
   `attributes-unique-expanded-name`), `xmlns:p=""` rejected (`no-prefix-undeclaring`),
   undeclared prefixes rejected (`prefix-declared`), `<` in attribute values rejected
   (`no-lt-in-values`), illegal characters rejected (`legal-characters`, `charref-legal-character`),
   invalid UTF-8 rejected (`illegal-byte-sequence-fatal`).
3. **Attribute value normalisation** per XML 1.0 §3.3.3
   (`values-must-be-normalized`, `linebreaks-normalized-to-lf`, `whitespace-normalized-to-space`
   for non-CDATA types where applicable), and document-level line-end normalisation
   (`processor-must-normalize-line-breaks`) implemented explicitly rather than relying on
   defaults.
4. **Encoding policy:** the XML declaration is parsed (rather than discarded as today);
   `version="1.0"` required, `encoding` must be UTF-8/utf-8 (typed error otherwise), an optional
   UTF-8 BOM is accepted, and the `standalone` pseudo-attribute is accepted and ignored
   (no DTD). The parsed declaration is exposed on the document as
   `Document::xml_declaration() -> Option<XmlDeclaration>` so serialization can reproduce it.
5. **Public entry points** keep today's names: `parse_string`, `parse_reader`, `parse_file`, plus
   `parse_bytes`, and `*_into(doc)` variants so a document created by the user can be filled.

### 8.2 Serializer (`src/io/write.rs`), replacing D2/D9

* **Element names are written with their prefix** (`<html:body>`), fixing D2; end tags match start
  tags exactly.
* Namespace declarations are written from the node's own declarations; nothing is inferred or
  added. Attribute names are written with their prefix, matching today's behaviour.
* Text/attribute values are escaped once (`escape-ampersand-and-lt`, and `>` where required);
  comments, CDATA and PI content are written verbatim, with CDATA content reproduced exactly
  (the value is validated to be free of `]]>` at construction).
* `WriteOptions { declaration: Option<XmlDeclaration>, expand_empty_elements: bool,
  indent: Option<Indent> }`, defaulting to "write the declaration if the document has one" and
  `<a/>` for empty elements. Document order of children is preserved; attribute order follows
  `BTreeMap` order and that is documented.
* Adjacent text nodes are merged on output unless the caller asks for verbatim output, so
  `parse → write` is stable regardless of how `quick-xml` chunked the input.
* `write_string`, `write_file`, `write_writer` keep their signatures and gain
  `write_string_with`/`write_file_with`/`write_writer_with` taking `&WriteOptions`.

## 9. Validation

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlValidationError {
    pub kind: ValidationErrorKind,
    pub node: NodeId,               // where it was found
    pub message: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationErrorKind {
    UndeclaredPrefix { prefix: NCName },
    UndeclaredDefaultNamespace,
    NamespacePrefixMismatch { prefix: NCName, declared: String, used: String },
    IllegalNamespaceDeclaration { prefix: Option<NCName>, uri: String },
    DuplicateXmlId { id: NCName }, InvalidXmlId { value: String },
    InvalidXmlLang { value: String }, InvalidXmlSpace { value: String },
    MissingRoot, MultipleRoots, RootHasParent, OrphanedNode, CyclicStructure,
    ContentOutsideRoot, ParentChildMismatch { .. },
}
```

`Document::validate()` walks the arena **once** and collects *all* issues (requirement (4)(2)),
never failing fast; `is_valid()` is the convenience wrapper. The walk is a single pass from the
root for structural checks plus a linear scan of all arena slots for node-local checks, and a
second pass only for the namespace-scope stack (a push/pop of in-scope bindings, so it stays
linear). Detached nodes are validated for node-local rules (illegal declarations, `xml:id`
syntax) but do not produce structural errors merely for being detached — that is a legal state
per requirement (2). The plan for `xml:id` is the conservative reading available without DTD:
`xml:id` attributes are the only ID-typed attributes, must be valid NCNames, and must be unique
document-wide.

Rule IDs referenced above are the ones from `specification/rules/` and each check is annotated
with its rule file, keeping the AGENTS.md convention.

## 10. Workspace and packaging

```text
Cargo.toml                             # [workspace] members = [".", "biodivine-lib-xml-dom-py-sys"]
                                       # root package stays the core crate -> src/ paths unchanged
src/…                                  # core crate (NO pyo3 anywhere)
biodivine-lib-xml-dom-py-sys/
  Cargo.toml                           # pyo3, name = "biodivine_lib_xml_dom_sys"
  src/lib.rs, src/{document,node,element,namespace,…}.rs
  pyproject.toml                       # maturin, abi3 where possible
python/
  biodivine_lib_xml_dom/               # pure-Python wrapper (idiomatic layer)
    __init__.py, document.py, node.py, element.py, namespace.py, errors.py, py.typed
  tests/                               # pytest
  docs/                                # Sphinx (autodoc over the wrapper, not autodoc-pyo3)
docs/book/                             # the "book" (Sphinx + sphinx-design)
docs/design/                           # REVIEW.md, PLAN.md, VERIFICATION.md, evidence/
```

Keeping the core crate as the workspace root package avoids moving `src/` and (with it) the whole
existing test/doc surface; the `-py-sys` crate is a sibling member. The core crate keeps zero PyO3
references, which is asserted in CI by a grep over `src/` and by `cargo tree -p biodivine-lib-xml-dom`.

`Cargo.toml` additions: `rust-version = "1.88"`, workspace lint configuration
(`[workspace.lints.rust] missing_docs = "warn"`, `unsafe_code = "forbid"`), and
`[lints.clippy]` groups. A `rust-toolchain.toml` pins the CI toolchain (1.95.0) while keeping the
declared MSRV honest (verified with `cargo +1.95.0` at minimum; MSRV 1.88 verification attempted if
a toolchain can be installed).

## 11. Documentation

* **Rustdoc (authoritative for the Rust API).** Every public item documented; `missing_docs` warning
  enabled; `# Errors`/`# Panics` sections wherever applicable (AGENTS.md); intra-doc links that
  resolve; `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` clean. Doctests double as examples.
* **Sphinx (authoritative for the Python API).** `sphinx.ext.autodoc` over the pure-Python package
  (docstrings in `py-sys` are surfaced through the wrapper), `napoleon`, `intersphinx`.
* **The book** (requirement (7)): Sphinx + `sphinx-design` tabs, so every example is written twice —
  a Rust tab and a Python tab — and a single click switches languages (`content.tabs.link`).
  Chapters: Getting started · Building documents · Traversing and editing · Namespaces · Parsing
  and serializing · Validation and error handling · Thread-safety and sharing · Python usage ·
  Migration from 0.1 · Design notes (arena + single lock).
  The Python snippets are executed by the test suite so the book cannot drift; Rust snippets are
  doctests or `examples/` targets.

## 12. Test strategy and seams

Per the `tdd` skill, work proceeds in vertical slices at pre-agreed **seams**; those seams are
recorded here instead of being confirmed interactively (approved condition), and each goal's plan
repeats the seams it touches.

| seam | what it catches | what it misses |
| --- | --- | --- |
| `xml_spec` pure predicates/constructors (unit tests) | name/content/namespace validation, rule-by-rule | anything requiring a document |
| `Document`/`Node`/`Element` public API (integration tests in `tests/`) | tree structure, attach/detach/clone semantics, cross-document rules, errors | parser/serializer specifics |
| `parse_*` / `write_*` (integration + property tests) | well-formedness enforcement, round-trip fidelity, no-panic guarantee | hand-built documents that never round-trip through text |
| `Document::validate` (integration tests) | whole-document invariants, multi-error collection | — |
| concurrency stress (integration test) | cycles, torn edits, deadlock (with the debug re-entrancy detector) | scheduling-dependent starvation |
| Python: pytest against the built extension | the whole Python stack | Rust-only concerns |

Applied `rust-testing-verification` guidance and how it shaped this review and the plan: the risk
classes it names are exactly the ones present here — **parser/untrusted input** (hence the
"parse must never panic" probe and the arbitrary-input property test), **concurrency/lock
protocol** (hence a multi-threaded race probe with a concrete cycle counter, and a stress test plus
debug re-entrancy detector rather than `loom`, which models atomics we do not have), and the
**FFI/PyO3 boundary** (hence boundary tests from Python in G5, and a `Send`/`Sync` audit of the
pyclass wrappers). Its "cheapest test that can fail for the real risk" rule is why the probes in
`docs/design/evidence/probes/` are minimal, single-purpose, and runnable in under a second each.

Verification gates for every goal: `cargo fmt --check`, `cargo clippy --all-targets --all-features`
eventually with `-D warnings`, `cargo test --all-features`, `RUSTDOCFLAGS=-D warnings cargo doc`.

## 13. Goal ordering

```text
G1 review + plan  ✔ (this document)
        │
        ▼
G2 core rewrite ──────────────────────────► G3 spec layer + I/O ──► G4 validation
   arena, single lock, handles,                     │                     │
   structural edits, clones, interner               │                     │
        │                                           ▼                     ▼
        └─────────────────────────────────► G5 Python bindings ◄───────────┘
                                                    │
                                                    ▼
                                          G6 docs (rustdoc, Sphinx, book)
                                                    │
                                                    ▼
                                          G7 verification + hand-off
```

Rationale for the order: G2 fixes the architecture that every other goal builds on (D1–D7 all
touch it); G3 depends on the handle/arena API and is where the remaining locally decidable spec
rules land; G4 needs both the arena and the resolved-name machinery from G3; G5 needs a stable
public API (G2–G4) and must not force API changes, so it comes after; G6 documents the finished
API and the Python layer; G7 verifies everything. G2's subgoals (arena/lock → handles → structural
editing → clones/interner) are ordered so each is independently testable and reviewable.

## 14. Risks and open questions

| # | risk / question | impact | mitigation |
| --- | --- | --- | --- |
| R1 | Multi-threaded correctness cannot be *proved* by tests. | A latent race could survive G2. | The single-lock invariant (§3.1) makes the class of bug structurally impossible rather than merely untested; a debug re-entrancy detector + stress test + the ported `it_cycle_race` scenario (expected to now yield 0 cycles) provide evidence. Loom is deliberately not used (§3.4). |
| R2 | MSRV: local toolchain is 1.99.0, CI pins 1.95.0 and declares 1.88.0. | Code could use APIs newer than the MSRV. | `rust-toolchain.toml` + `rust-version` in `Cargo.toml`; 1.95.0 is already installed and will be part of the G7 gate. Full 1.88 verification only if that toolchain can be installed, otherwise the declared MSRV is raised to what is actually verified — no unverified claims. |
| R3 | Arena never reuses slots ⇒ memory grows with the number of nodes ever created. | Long-running edit loops on one document leak slots. | Documented limitation on `Document`; `node_count()` exposes it so users can detect the situation. Slot reuse was rejected because it invalidates outstanding handles (ABA). Revisit only with a generational design and an explicit `gc()` contract. |
| R4 | `parking_lot` under Miri. | Cannot claim Miri evidence. | Attempt in G7; if unsupported, record "not applicable" (no `unsafe` in this crate). |
| R5 | Cross-document copy must re-create names/namespaces in the target; a naive implementation would leak document-specific assumptions. | Silent aliasing between documents. | The interner is per-document and `deep_clone_into` interns through the target (§7); tests assert that the copy's document identity is the target and that no `Arc` is shared with the source for names/namespaces. |
| R6 | Requirement (4)(2) mentions "all IDs are unique" without DTDs. | Over- or under-enforcement. | Conservative reading: only `xml:id` is ID-typed, must be a valid NCName and unique document-wide. Recorded here; revisit if the advisor prefers `id`-by-name heuristics. |
| R7 | Serializer "no magic" vs. producing valid XML. | Writing an unvalidated document can emit undeclared prefixes. | Explicitly intended by requirement (3). The book documents `validate()` as the check to run before serialization; `write_*` stays magic-free, and `WriteOptions` gains nothing that invents declarations. |
| R8 | Replacing the public API breaks downstream users (Biodivine tooling). | Churn for dependents. | Names kept where possible (`Document`, `Element`, `Namespace`, `QualifiedName`, `parse_*`, `write_*`, `xml_spec` types); the book has a "Migration from 0.1" chapter; version bumped to 0.2.0 on the `rewrite` branch (never published, never pushed). |
| R9 | Sphinx/sphinx-design availability in this sandbox. | G6 could be blocked. | Fallback documented in G6's acceptance criteria: `mkdocs-material` content tabs, which offer the same language-switching behaviour. |
| R10 | Toolchain/build reality: `maturin` + a C toolchain + Python headers are needed for G5. | G5 could be blocked. | `build-essential` is already installed; Python 3.11 + pip are present; PyO3 builds against the system CPython. Verified at the start of G5 before promising a wheel. |

## 15. Implementation notes and deviations

Recorded as the plan is executed; each entry says what changed and why.

### 15.1 G2 — arena and handles

* **`deep_clone_into` is infallible, so it has no `_checked` twin.** §4.2 listed one, but with the
  snapshot protocol (§7, §3.1) the operation cannot fail: the source snapshot is taken under a
  read lock and released, then the copy is inserted under the target's write lock. There is no
  contention failure and no logical failure, so a `_checked` twin would be a pure middle man.
  The `_checked`/panicking split therefore applies to the *editing* operations
  (`append_child`, `insert_child`, `insert_before`, `insert_after`, `replace_with`, `set_root`,
  `set_attribute`), and `detach`/`remove` are infallible for the same reason (`detach` on a
  detached node is a documented no-op).
* **Immutable payloads are shared *across* documents (risk R5 revised).** §7 said a cross-document
  copy must "share nothing with the source". That is stricter than necessary and would defeat the
  `Arc` dedup scheme requirement (1) asks to keep: `QualifiedName` and `Namespace` are immutable
  values holding a local name / URI / prefix and no document identity, so a shared allocation can
  neither expose nor retain the source document's state. What is guaranteed instead, and tested,
  is that (a) the copy's document identity is the target, (b) the target document interns the
  copy's names and namespaces into its own canonical allocation, and (c) the copy stays fully
  usable after the source document has been dropped — i.e. nothing keeps the source arena alive.
* **`NodeId` capacity.** Slots are never reclaimed (risk R3) and ids are `u32` indices, so a
  document holds at most `u32::MAX` nodes over its lifetime. The bound is checked explicitly
  (`arena::ensure_capacity`) and documented on `Document`/`NodeId`, and the guard itself is unit
  tested, rather than letting the index wrap.
* **The `xml_spec` `Arc<str>` refactor landed as its own commit** (`refactor(xml_spec): back
  content newtypes with Arc<str>`) and is proven semantics-preserving: the `#[cfg(test)] mod tests`
  block of `xml_spec.rs` is byte-identical before and after, the set of doctests is unchanged
  (including `NCName::as_str`), and the rule-file assertions are untouched.
* **The single-lock invariant is machine-checked in debug builds** by a re-entrancy guard keyed by
  document identity. It panics on re-entering the *same* document's lock (the operation that would
  hang) and deliberately tolerates nesting across different documents, because that is a lock
  *ordering* concern which this crate avoids by construction (no operation holds two locks) rather
  than a re-entrancy concern. Both behaviours are tested.

### 15.2 G2 — post-review round

* **`replace_with` on the node itself is a no-op, not an error.** The first implementation returned
  `XmlError::NodeHasNoParent` for `x.replace_with_checked(x)`, which was both inconsistent with
  `Arena::replace`'s own documentation and with the sibling operations (`insert_before`/
  `insert_after` are documented no-ops when sibling and child coincide), and plainly misleading:
  the node *does* have a parent, the request is simply already satisfied. It now reports success
  and leaves the document untouched, for attached and detached nodes alike, and both the checked
  and the panicking variant behave that way. (`Arena::replace` returns `XmlResult<()>` now: the
  parent it used to return was never used, and a no-op has no parent to report.)
* **Namespaces: `resolve_prefix` vs `get_namespace`.** §6.2 lists a `resolve_prefix` in the
  namespace API; the public accessor that was implemented is `get_namespace` (the name the previous
  version already used). `Arena::resolve_prefix` exists internally as the primitive that walks the
  ancestor chain. No alias is added: `get_namespace` is the equivalent accessor and a second name
  for the same thing would only be a second spelling to keep in sync.
* **Every `# Errors` section was audited against the implementation** rather than assumed, and the
  audit is now checked by `tests/errors.rs`, which triggers each documented condition and asserts
  the documented variant comes back (13 tests, covering `Namespace::{new,without_prefix,prefixed}`,
  `QualifiedName::{without_namespace,with_namespace,resolve_*}`, `Document::{set_root_checked,
  create_*}`, `Element::{set_attribute_checked,declare_namespace_checked,resolve_*}`,
  `Node::{append_child_checked,insert_child_checked,insert_{before,after}_checked,
  replace_with_checked}`, `parse_*`/`write_file`). Gaps found and fixed while auditing:
  `QualifiedName::resolve_*` and `Element::resolve_*` can return `XmlError::ReservedPrefix` (for the
  `xmlns` prefix) but their `# Errors` sections did not say so; `Namespace::{new,prefixed,
  without_prefix}` documented "an error" without naming the variants; and `parse_reader`'s list was
  incomplete (it omitted `InvalidName`, `ReservedPrefix` and `InvalidNamespace`, all of which it
  can report, as the new tests confirm).
* **Two further D8 gaps found by that audit** (both pinned by the characterisation test
  `unclosed_elements_are_currently_accepted` in `tests/errors.rs` and added to REVIEW D8, so G3 has
  to fix them deliberately): `parse_string("<a>")` accepts an unclosed start tag
  (`rule.elements-and-tags.every-start-tag-must-have-end-tag`), and
  `parse_string("<a><?xml target?></a>")` accepts an illegal processing-instruction target
  (`rule.well-formedness.pi-target-not-xml`) while silently dropping the node.

### 15.3 G3 — parser and serializer

* **Escapable vs. non-escapable content (advisor condition C1).** Two different treatments, chosen
  by whether an escape sequence exists:
  * *Text nodes* keep the caller's bytes verbatim and the serializer writes a literal carriage
    return as `&#xD;`, because the processor's mandatory line-end normalisation
    (`rule.document-structure.processor-must-normalize-line-breaks`) would otherwise turn it into a
    line feed on the next parse. Tab and line feed need no escaping in content, and `>` is escaped
    so that a `]]>` cannot form (the `CharData` production excludes it).
  * *Attribute values* also keep the caller's bytes, and the serializer additionally writes tab,
    line feed and carriage return as character references, because attribute-value normalisation
    turns a *literal* whitespace character into a space (XML 1.0 §3.3.3,
    `rule.attributes.values-must-be-normalized`). This is the asymmetry the condition asked to pin:
    `a="x\ty"` parses to `x y`, while `a="x&#x9;y"` parses to `x\ty` and round-trips byte-exactly.
    Both directions are asserted in `tests/io.rs::attribute_value_normalisation_in_both_directions`.
  * *Comments, CDATA sections and processing-instruction content* cannot contain an escape sequence
    at all, so a literal `\r` there is **normalised at construction** to `\n` (documented default
    behaviour, requirement (4)(1)). Normalising rather than rejecting is deliberate: a `\r` in a
    comment is perfectly valid XML, and refusing it would be worse than recording it the way a
    re-parse will read it. The consequence is that these three payload types always hold exactly
    what a parse would produce (`xml_spec::normalize_line_ends`, tested in
    `xml_spec::g3_tests`).
  * The property generators inject `<`, `>`, `&`, quotes, tabs, line feeds and carriage returns
    into text, attribute values, comments, CDATA and PI content, so these cases are exercised
    rather than assumed.
* **Depth (advisor condition C2).** Serialization is iterative (an explicit work stack in
  `io::write`), and so are `Display` for `Node`/`Element` (they use the same writer) and the arena's
  `Snapshot` — the last one was made flat and index-based for exactly this reason, because taking,
  rebuilding *and dropping* a recursive snapshot all recurse. `tests/io.rs::a_deeply_nested_document_round_trips`
  round-trips a 5 000-level document and also deep-clones it; no bound is imposed anywhere, so no
  depth can turn into an abort.
* **Top-level content policy (advisor condition C3).** The data model is one root element and
  nothing else at the top level, so the parser's behaviour is: whitespace outside the root is
  ignored; a comment or processing instruction outside the root is **discarded** (the model has
  nowhere to represent it — silently losing it is the documented default behaviour, and it keeps
  parsing real-world documents that start with a comment or an `<?xml-stylesheet?>`); non-whitespace
  text outside the root is `ContentOutsideRoot`; a second element is `MultipleRootElements`; a
  document with no element (including an empty input or a bare declaration) is `MissingRoot`. All
  six cases are pinned in `tests/io.rs::top_level_content_policy`, documented in the `io::parse`
  module documentation, and the round-trip generator only produces representable documents.
  Representing top-level misc nodes for real (a `prolog`/`epilog` pair of vectors in the arena, plus
  the API and validation work that implies) is the obvious future extension if the loss turns out
  to matter; it is not needed by any requirement.
* **Rule annotations vs. the inventory (advisor condition C4).** `docs/design/evidence/rule-enforcement.md`
  is generated by `make_rule_enforcement.py`, which reads the *set* of layer-B/D rules out of
  `rule-inventory.md` (so a rule cannot be dropped from the table silently — the script fails if any
  layer-B/D rule lacks a verdict) and pairs each with an explicit verdict and a pointer to the code
  path and test. Current result: 57 rules, **49 enforced, 6 partial, 2 deferred**. The six partial
  ones are the DTD-dependent halves of rules whose non-DTD half *is* implemented
  (`entities.document-entity-well-formed`, `entities.only-referenced-entities-well-formed`,
  `namespace-usage.declarations-direct-or-internal-dtd`, `well-formedness.entities-must-be-well-formed`,
  `well-formedness.no-peref-in-comments`, `well-formedness.no-peref-in-pis`); the two deferred ones
  are the UTF-16 halves of `utf-utf16-support` / `utf-8-utf-16-support`, which AGENTS.md puts out of
  scope. `tests/io.rs::rules_referenced_by_the_parser_and_serializer_exist` checks that every rule
  named in the parser/serializer documentation still has its summary file.
* **The round-trip signature compares the full expanded name.** The canonical signature spells an
  element name as `prefix|uri|local` (the same shape the attribute signature uses) and the document
  generator produces varied local names, so the property is sensitive to element-name fidelity and
  not just to namespaces. Verified by deliberate sabotage, twice: making the serializer append
  `_corrupted` to a tag name makes the signature report `||a_corrupted` against `||a`, and
  reproducing the pre-G3 behaviour of writing the *local name only* (REVIEW D2) makes it report
  `||a` against `p0|http://example.com/ns0|a`. Both sabotages were reverted immediately; the
  property passes again on the unmodified code.
* **Deviations from §8.1.** `parse_*_into` variants were dropped: cross-document import already has a
  sanctioned, tested path (`deep_clone_into`), and an extra constructor that fills an existing
  document would either duplicate that logic or leave the target partially modified on error.
  `parse_bytes` exists as planned. Indentation on output is still omitted (it would inject
  whitespace into mixed content and is not needed for fidelity).

### 15.4 G4 — whole-document validation

* **The error type is a newtype around the list.** The criterion said
  `Result<(), Vec<XmlValidationError>>` "or equivalent"; the implementation returns
  `Result<(), XmlValidationErrors>`, a newtype that derefs to a slice, iterates, implements
  `std::error::Error` and renders *all* issues in one `Display` (so a caller can `?` it and a
  human can read the whole list). The list is deterministic: nodes in arena order, and within a
  node the checks run in a fixed order, so two validation runs are `==`.
* **`XmlValidationError::node` is `Option<NodeId>`.** Every issue points at the node it belongs to
  except `MissingRoot`, where there is no node to point at. The alternative — a synthetic node id
  or a separate error type for that one case — would be worse than an `Option` that is documented.
* **Structural checks are self-checks.** The arena maintains parent/child agreement, acyclicity and
  "the root has no parent" by construction, so those variants (`ParentChildMismatch`,
  `CyclicStructure`, `RootHasParent`, `RootIsNotAnElement`) should never fire. They are implemented
  anyway, because the guarantee then does not depend on the maintaining code being correct, and
  because the *rule* (`rule.well-formedness.elements-nest-properly`,
  `single-root-element`) is a document-level statement that deserves an explicit check.
* **`xml:id` uniqueness is checked over the attached tree only.** A detached `deep_clone` of a
  subtree legitimately carries the same `xml:id` values as its original, so comparing all arena
  slots would produce a false positive for a completely normal workflow. Node-local rules
  (`xml:lang`/`xml:space` values, `xml:id` syntax, name resolution against the node's own scope)
  *are* checked for detached nodes, so a subtree can be prepared and validated before it is
  attached. Both halves are tested
  (`a_detached_copy_does_not_conflict_with_its_original`,
  `detached_subtrees_are_validated_against_their_own_scope`).
* **The two "must be declared" rules are DTD validity and stay out of scope.** The inventory
  assigned `rule.document-structure.xml-lang-must-be-declared` and
  `xml-space-must-be-declared` to layer C, but their content is "this attribute MUST be declared
  [in an `ATTLIST`]": without DTD processing there is no declaration to check. The *checkable* half
  of those two rules — the value shape — is layer A and is applied by the validation pass through
  `is_valid_language_tag`/`is_valid_xml_space_value`. The two `xml-lang-must-inherit-*` rules
  describe how a processor resolves the language of an element; this crate exposes no API that
  consumes language information, so they are recorded as *not applicable* rather than silently
  dropped. All three verdicts are in `docs/design/evidence/rule-enforcement.md`, which now covers
  layers B, C and D (66 rules: 54 enforced, 6 partial, 4 deferred, 2 n/a).
* **The `xml:id` policy is pinned in both directions** (advisor condition C1), in
  `tests/validation.rs::the_xml_id_policy_is_pinned_in_both_directions` and in the rustdoc of
  `Document::validate`:
  (a) a detached `deep_clone` that repeats its original's `xml:id` produces **no** issue;
  (b) once *both* are attached the duplicate *is* reported — exactly one issue, on the later node,
  with the earlier one named in the message — and a third element sharing the value adds exactly
  one more issue, so each offending node is reported once;
  (c) node-local rules (`xml:id` syntax, `xml:lang`, `xml:space`) *are* reported for detached
  nodes, and fixing them there removes the issues without attaching anything;
  (d) the attached-tree-only rule for uniqueness is stated in the `validate()` rustdoc and here.
* **The no-false-positive direction is a property, not a fixture list** (advisor condition C2):
  `tests/properties.rs::generated_documents_validate_clean` reuses the round-trip generator and
  asserts that every generated document validates with zero issues, that `is_valid()` agrees, and
  that a `write`/`parse` round trip does not introduce a problem either (256 cases). This is what
  catches an over-eager rule, which is the main risk of adding validation at all.
* **The structural checks are labelled honestly** (advisor condition C3). `MultipleRootElements`
  and `ContentOutsideRoot` are *parser* errors (`XmlError`) — a second root or text outside the root
  cannot even be loaded — and of the validation kinds only `MissingRoot` is reachable through the
  public API: the arena writes parent and child links together, `set_root` replaces the root rather
  than appending a second one, and `Arena::attach` rejects cycles. The other four structural
  variants are therefore documented as *defensive self-checks* on the enum, at `check_structure`,
  and in the `validate()` rustdoc, and
  `tests/validation.rs::consistent_documents_produce_no_structural_issues` asserts the complementary
  direction over a representative set (no root, root only, detached subtree present, re-attached
  subtree, a 2000-level tree, mixed content of every kind, cleared root) so the checks cannot
  silently start producing false positives.
* **Validation is read-only and takes the lock once.** `Document::validate` goes through
  `read_arena`, so it cannot re-enter the lock, and a test asserts that the serialized document is
  unchanged by a validation run.

### 15.5 G5 — Python bindings

* **The workspace keeps the core crate as the root package** and adds `biodivine-lib-xml-dom-py-sys`
  as a member, so `src/` and every existing test stayed where they were. The native module is a
  *private* submodule of the pure-Python package (`module-name = "biodivine_lib_xml_dom._sys"` with
  `python-source = "python"`), which is what makes the three-layer split visible in the import path.
* **`extension-module` is enabled by maturin, not by default** (`[tool.maturin] features`), with a
  `dev-dependency` on `pyo3` using `auto-initialize`: that is the arrangement in which both
  `cargo test` (which links libpython into the test binary) and `maturin develop` (whose wheel must
  *not* link it) work from one manifest. Verified by building and importing a scratch extension
  before writing any binding.
* **The full item-by-item mirroring audit is `docs/design/BINDINGS.md`** (advisor condition C1): one
  row per public Rust item group with its `_sys` binding, its Python name and a verdict, so the
  "does not make sense to mirror" set is explicit. Its notable entries: the panicking twins of the
  `_checked` operations are dropped (Python has no panics, so the checked behaviour is the only
  one); `NodeContent` is not mirrored because `Node.kind()` plus the typed accessors express the
  same thing without a discriminator dance; and the `xml_spec` newtypes are not mirrored because in
  Python their role — validity enforced by construction — is played by the raising constructors.
  Rust's `Deref<Target = Node>` for `Element` cannot be expressed for a native Python type, so
  `_sys` offers `Element.node()` (the Rust method) and the pure-Python `Element` subclasses the
  pure-Python `Node` so users get one object with both APIs.
* **The GIL policy is stated and implemented** (advisor condition C2): parsing, serializing,
  validation and the cross-document copies run under `Python::detach` with `Send`-only captures,
  everything else holds the GIL. The policy table is in both the crate docs and
  `BINDINGS.md` §3. No binding acquires two document locks, so the Rust deadlock-freedom argument
  carries over unchanged; `tests-python` shares one document between four Python threads with a
  30-second timeout so a hang fails the test.
* **Boundary hygiene is checked rather than asserted** (advisor condition C3): no
  `unwrap`/`expect`/`panic!` outside the test module of the py-sys crate; no profile sets
  `panic = "abort"`, so a bug would surface as PyO3's `PanicException` instead of killing the
  interpreter; each `#[pyclass]` has a `Send`/`Sync` justification in `BINDINGS.md` §5 backed by a
  compile-time assertion in `src/tests.rs`, and `unsendable` is needed nowhere because every wrapped
  Rust type is `Send + Sync`.
* **The mirror is usable from Rust as well**: the `#[pymethods]` bodies are `pub`, so the crate's own
  tests drive the same code paths Python does without building a wheel.
* **Pins and recipe** (advisor condition C4): `pyo3 0.29.3`, `maturin 1.15.0`, `pytest 9.1.1`,
  `python3-dev` for the CPython 3.11 headers. The recipe is in `BINDINGS.md` §7 and the transcripts
  are in the goal result. `cargo tree -p biodivine-lib-xml-dom --edges normal` still lists only
  `parking_lot`, `quick-xml` and `thiserror`.
* **Deliberately not done here**: `abi3` (a packaging decision, recorded in `BINDINGS.md` §8) and the
  Sphinx site / tutorial book (goal G6, which generates them from the docstrings added here).
