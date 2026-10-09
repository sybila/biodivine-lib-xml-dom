# Review: `biodivine-lib-xml-dom` at commit 76beb74

Audit of the current implementation against the eight task requirements, written before any
rewrite work. Every claim is traceable to a file, a command, or a file under
`docs/design/evidence/`.

* Baseline verification: `docs/design/evidence/baseline.md`
* Reproducible probes and raw transcripts: `docs/design/evidence/probes/RESULTS.md`
* Rule-by-rule scope decision: `docs/design/evidence/rule-inventory.md`
* Target design: `docs/design/PLAN.md`

Toolchain used throughout: `rustc 1.99.0 (b940084d7 2026-09-28)` / `cargo 1.99.0 (5f94df478 2026-08-27)`.
CI pins `rust-version: 1.95.0` / `min-rust-version: 1.88.0` (`.github/workflows/build.yml`);
the baseline passes unchanged on both compilers, so nothing below is a compiler artefact.

## 0. What the current implementation is

4 250 lines of Rust across nine modules, ~650 of them tests:

| module | lines | role |
| --- | --- | --- |
| `src/xml_spec.rs` | 1 043 | validated newtypes (`NCName`, `Text`, `Comment`, `CData`, `PiTarget`, `PiData`), name/namespace predicates, rule-file-anchored tests |
| `src/io.rs` | 1 147 | `quick-xml` based parser and serializer |
| `src/element.rs` | 656 | `Element = Arc<RwLock<ElementData>>`, `XmlNode` enum |
| `src/qualified_name.rs` | 656 | `QualifiedName = Arc<QualifiedNameData>` with namespace resolution |
| `src/namespace.rs` | 243 | `Namespace = Arc<NamespaceData>` |
| `src/lib.rs` | 209 | crate docs, re-exports, integration-ish unit tests |
| `src/main.rs` | 184 | demo binary (not part of the library API) |
| `src/document.rs` | 91 | `Document = Arc<InternalDocument>` |
| `src/error.rs` | 21 | `XmlError` (stringly typed) |

Baseline gates: `cargo build`, `cargo test` (81 unit + 25 doctests), `cargo clippy --all-targets`
(10 warnings, all `uninlined_format_args` in test code), `cargo fmt --check` (clean),
`cargo doc --no-deps` (clean). See `baseline.md`.

## 1. Requirement-by-requirement status

| # | requirement | status | evidence / pointers |
| --- | --- | --- | --- |
| (1) | thread-safe sharing of one document; **arena + one document-level lock**; `Node`/`Element` as IDs; deep/shallow/handle clone; keep `Arc` dedup | **missing (architecture wrong)** | `src/element.rs:59` is `Arc<RwLock<ElementData>>`, i.e. one lock *per node*; there is no arena and no `Node` type (`cf_missing_api`). `Clone` currently copies the handle (correct), but neither `deep_clone` nor `shallow_clone` exists. `Arc` sharing for `Namespace`/`QualifiedName` exists and is good (`src/namespace.rs`, `src/qualified_name.rs`), but there is no per-document interning. Concurrency is unsafe in a way `Send`/`Sync` cannot catch: `it_cycle_race` creates cycles. |
| (2) | multiple documents; node↔document binding; attach/detach; copy between documents | **partial** | `Document` is `Arc`-shared and `Element` carries its `Document` (`src/element.rs:33`), and cross-document `add_child_element` is rejected (`src/element.rs`). But there is no detach/remove/insert/replace API and no way to copy a subtree between documents (`cf_missing_api`: `remove`, `detach`, `replace_with`, `insert_child`, `deep_clone_into` all missing). |
| (3) | safe, ergonomic namespace support; no "magic"; breakage detected by validation | **partial** | `QualifiedName`/`Namespace` design is right (expanded names carry prefix + URI, `Arc`-shared, value equality on URI only). But serialization drops element prefixes (`it_prefix_loss`) and never emits missing declarations (`it_ns_silent_loss`), and there is no `validate()` to catch the breakage. Namespace *queries* exist (`get_namespace`, `resolve_qualified_name`) but there is no way to enumerate the in-scope bindings or to remove a declaration. |
| (4)(1) | low-level integrity enforced by construction / at parse time | **partial, mostly good** | `NCName`/`Text`/`Comment`/`CData`/`PiTarget`/`PiData` are validated at construction with `TryFrom` (`src/xml_spec.rs`), attributes are keyed by expanded name so duplicates are unrepresentable, and the parser rejects duplicate expanded attribute names and `xmlns:p=""`. Gaps: `declare_namespace` *errors* on re-declaration where requirement (4)(1) asks for a documented overwrite; `XmlError`/`XmlResult` use ad-hoc `String` payloads; entity handling is a panic rather than an error (`it_entity_panic`). |
| (4)(2) | high-level integrity, whole-document, **all issues at once** | **missing** | no `Document::validate`, no validation error type, no multi-error accumulator anywhere in the tree. |
| (5) | XML-spec compliance, rule annotations, `xml_spec` module convention | **partial** | the `xml_spec` convention is followed and 30-odd rules are unit-tested with a "rule file must exist" guard (`src/xml_spec.rs:718-732`); the two big specification HTML files and all 194 rule summaries are present. But annotations only cover the rules already implemented, and a large fraction of locally decidable rules (attribute-value normalisation, `<` in attribute values, single root, end-tag matching, predefined entities, `xml:lang`/`xml:space`) are neither annotated nor enforced. See `rule-inventory.md`. |
| (6) | PyO3 bindings, three layers (core / `-py-sys` / pure Python) | **missing** | `sh_python_bindings`: no `pyo3` reference, one workspace member, no Python sources, no packaging. |
| (7) | rustdoc + Sphinx + docs book with Rust/Python switchable examples | **partial** | per-item rustdoc is genuinely decent (615 `///` lines) and doctests run; but `missing_docs` is not enabled, `# Errors`/`# Panics` sections are inconsistent, there is no docs book and no Sphinx configuration (`sh_docs_and_repo_state`). |
| (8) | versioning, clippy, formatting, small commits on a `rewrite` branch | **partial** | `cargo fmt`/`clippy` are clean, `[lints.clippy] uninlined_format_args = "warn"` is set, and CI exists; but there is no `rewrite` branch, no `rust-toolchain.toml`, no `rust-version` in `Cargo.toml`, a stray 0-byte `output.xml`, and a 184-line demo binary inside the library crate. |

## 2. Defects, ranked by severity

Each defect below has a reproducible probe; raw transcripts are in
`docs/design/evidence/probes/raw/`.

### D1 — CRITICAL: the parser aborts the process on the predefined entities

`src/io.rs:127`:

```rust
Ok(Event::GeneralRef(_)) => {
    unimplemented!("Custom entities are currently not supported.")
}
```

`&amp;`, `&lt;`, `&gt;`, `&apos;`, `&quot;` are *predefined* entities that every XML processor
must expand (XML 1.0 §4.6, `specification/rules/rule.entities.predefined-entities-recognized.md`);
they are not "custom entities". `parse_string("<a>AT&amp;T</a>")` therefore panics, as does
`<a>&undefined;</a>`. Probe: `it_entity_panic` (both tests panic at `src/io.rs:127`).
Impact: `unimplemented!` cannot be caught by the PyO3 layer — in a Python process this would
abort the interpreter.

### D2 — CRITICAL: the serializer silently corrupts documents

`src/io.rs::write_element()` (line 271 ff.) builds the start tag and end tag from
`qname.local_name()` alone, ignoring `qname.namespace().prefix()`:

* `<html:html xmlns:html="…"><html:body>hi</html:body></html:html>` round-trips to
  `<html xmlns:html="…"><body>hi</body></html>` — prefixes are lost (`it_prefix_loss`);
* namespace declarations are only written if they are stored on the node, so an element whose
  name carries a namespace with no declaration in scope is written without any indication of
  that namespace, and re-parsing produces a different document (`it_ns_silent_loss`).

Note the same function *does* prepend prefixes for attribute names, so the element/attribute
behaviour is inconsistent. This is the single most damaging defect: any `parse → edit → write`
pipeline that uses namespaces loses data silently, and the output is not even equivalent XML.

### D3 — CRITICAL: concurrent edits can create cycles (the per-node locking design is unsound)

`Element::add_child_element` performs four steps with per-node locks acquired and released in
between, so the operation is not atomic:

1. `child.0.read().parent.is_some()` — "is it attached?"
2. `child.is_ancestor(self)` — "would this create a cycle?"
3. `child.0.write().parent = Some(self.clone())`
4. `self.0.write().children.push(XmlNode::Element(child))`

Two threads editing two *different* children (`a.add_child(b)` and `b.add_child(a)`) each pass
their own step-1 check because they read different fields. Probe `it_cycle_race`: 37 170 cycles
out of 128 000 attempted pairs (~29 %). Afterwards `is_ancestor`/`is_attached` walk the cycle
forever, so the document becomes unusable. Commit 6bf19e3 fixed the *single-threaded* case; the
concurrent case cannot be fixed by adding more per-node locks without acquiring them in a global
order, which is exactly the complexity the task asks to avoid. This is the strongest argument
for the requested single document-level lock.

### D4 — HIGH: `Arc` reference cycles leak every document

`ElementData` holds `parent: Option<Element>` (`src/element.rs:41`) while the parent holds
`children: Vec<XmlNode>` containing `XmlNode::Element(Element)` (`src/element.rs:39`), and
`Element = Arc<RwLock<ElementData>>` (`src/element.rs:59`). Both directions are strong
references. Probe `unit_element_arc_cycle_leak`: after dropping the `Document` and the user's
handle, `Arc::strong_count(child) == 2`. A program that builds and drops thousands of documents
never frees them. Additionally `Document::empty()`'s `InternalDocument` keeps the root alive via
`RwLock<Option<Element>>` (`src/document.rs:12`) — that part is fine, but the tree below it is not.
An arena of `NodeId` indices removes the entire problem class.

### D5 — HIGH: no whole-document validation (requirement (4)(2) is entirely absent)

There is no `Document::validate`, no validation error type and no way to collect multiple issues.
Consequences visible today: a document that uses an undeclared prefix (or a namespace with no
declaration) is serialized as if the namespace did not exist (D2), and nothing tells the user.
`specification/rules/` already contains the rules that need this treatment — the inventory
classifies 9 of them as requiring a whole-document view (`xml:id` uniqueness, `xml:lang`
declaration/inheritance, `xml:space` declaration, `xmlns` as an element prefix, prefix scope,
default-namespace scope, `prefix-declared`).

### D6 — HIGH: no Python bindings at all

Requirement (6) is unimplemented: no `pyo3` dependency anywhere, a single-package workspace, no
`biodivine-lib-xml-dom-py-sys` crate, no pure-Python wrapper package, no `pyproject.toml`, no
`.py`/`.pyi` files (`sh_python_bindings`).

### D7 — MEDIUM: the node API is missing the structural half

`cf_missing_api` compiles to 12 errors: no `Node` type, no `deep_clone`/`shallow_clone`/
`deep_clone_into`, no `remove`/`detach`/`replace_with`/`insert_child`, no `namespaces_in_scope`/
`remove_namespace_declaration`, no `validate`, no `belongs_to`. In practice an element can only be
appended — never removed, moved, reordered, or copied into another document. Requirements (1) and
(2) explicitly need all of these. Related smaller gaps: no `Display`/`to_string` for a document,
no typed creation of text/comment/CDATA/PI nodes (only `add_*` on an existing element), no
attribute removal, no `Document::set_root` clearing, and no way to build a node before knowing
its final name/namespace.

### D8 — MEDIUM: namespace-relevant local rules are not enforced

The parser/serializer do not implement a set of rules that are locally decidable and cheap:

| rule | current behaviour |
| --- | --- |
| `rule.well-formedness.single-root-element` | a second root produces a confusing `InvalidOperation("Element belongs to a different document")` or an `add_child_element` failure rather than a validation statement; text outside the root is silently dropped |
| `rule.elements-and-tags.end-tag-must-match-start-tag`, `rule.well-formedness.elements-nest-properly` | delegated entirely to `quick-xml`; on mismatch the whole parse fails with a generic message |
| `rule.elements-and-tags.start-tag-syntax` / `rule.attributes.no-lt-in-values` | `<` inside an attribute value is not explicitly checked |
| `rule.document-structure.processor-must-normalize-line-breaks` | not implemented (only `quick-xml`'s built-in `xml10_content`/`normalized_value` behaviour applies) |
| `rule.document-structure.xml-lang-must-be-bcp47-or-empty`, `…xml-space-must-be-enumerated-default-preserve` | not checked anywhere |
| `rule.attributes.id-must-be-unique`, `rule.attributes.id-must-be-name` | not checked anywhere (no `xml:id` handling) |
| `rule.entities.default-encoding-utf8`, `…encoding-must-match-declaration`, `…utf8-bom-optional` | the `<?xml …?>` declaration is discarded without inspection (`Event::Decl(_) => {}`), so a declared non-UTF-8 encoding is silently mis-read |

`rule-inventory.md` shows the full classification: of 194 rule files, 30 are local
(type/construction), 55 are locally decidable during parsing, 9 need a whole-document view, 2 are
serialization concerns, 5 are not applicable, and 93 are deliberately out of scope (DTD/validity
and non-UTF-8 encodings).

### D9 — MEDIUM: serialization is not configurable and loses structure

* no XML declaration is ever written, so a parsed `<?xml version="1.0" encoding="UTF-8"?>` is
  lost on output (the `output.xml` fixture and `tests/assets/example.xml` both contain one);
* every element is written as `<a></a>`; there is no empty-element style and no way to choose;
* attributes are emitted in `BTreeMap` order (document order is not preserved). XML does not
  attribute meaning to attribute order, so this is a documentation issue rather than a bug, but
  it should be stated explicitly;
* text nodes are split exactly as they were parsed (adjacent `Event::Text` chunks are not merged),
  which makes `parse → write` output depend on parser chunking.

### D10 — MEDIUM: the error model is stringly typed

`XmlError` (`src/error.rs`) has `InvalidXml(String)`, `NamespaceError(String)`,
`InvalidOperation(String)` and an unused-looking `ElementNotFound`. Callers cannot match on the
failure reason, and the three string variants routinely carry conditions that belong to
well-formedness, spec violations and API misuse respectively. Requirement (4) needs a structured
error enum for validation results (one variant per issue kind) and for the `_checked` API.

### D11 — LOW: housekeeping and repository hygiene

* `output.xml` — a 0-byte file committed in the crate root (probably an artefact of the demo binary);
* `src/main.rs` — 184-line demo binary inside a library crate; should be `examples/*.rs`;
* no `rust-toolchain.toml` and no `rust-version` in `Cargo.toml` although CI pins 1.95.0/1.88.0;
* `[lints.clippy]` sets one lint; `missing_docs`, `unsafe_code`, rustdoc-warning denial, and a
  workspace-level lint configuration are absent;
* `.gitignore` covers `/target` and `.idea` but not `__pycache__`, `.venv`, Sphinx `_build`, etc.
  (needed once G5/G6 land);
* `.coderabbit.yaml` filters only `specification/**`; once `docs/design/evidence/` contains large
  transcripts it should be excluded too;
* no `rewrite` branch, and the ten `clippy` warnings are all in test code
  (`uninlined_format_args`) — trivially fixable, and `-D warnings` is not enforced anywhere.

## 3. What works and must be preserved

The rewrite replaces the *structure* (node storage, locking, handle types, I/O), but several
decisions are good and should survive:

1. **`xml_spec` as the home of spec logic.** Validated newtypes with `TryFrom` plus rule-file-anchored
   tests (`verify_rule_exists`) is exactly the convention AGENTS.md asks for. Keep the module, keep
   the type names, keep the "every test cites a rule id" rule, and extend it (see `rule-inventory.md`).
2. **Validated newtypes for content.** `NCName`, `Text`, `Comment`, `CData`, `PiTarget`, `PiData`
   already make a large class of malformed documents unrepresentable, which is requirement (4)(1)
   in its purest form. They should also back the arena's storage.
3. **Expanded names as map keys.** `BTreeMap<QualifiedName, String>` for attributes makes duplicate
   attributes unrepresentable *by construction*, and `QualifiedName`'s equality (local name + URI,
   ignoring the prefix) is the semantically correct notion for both attributes and elements.
4. **`Arc`-shared immutable `Namespace`/`QualifiedName`.** Cheap to clone, `Send + Sync`, and the
   value-equality/hash/ordering implementations are already carefully aligned
   (`test_hashing_semantic_equality`, `test_ord_consistent_with_partial_eq`).
5. **Reserved-prefix handling.** `validate_namespace` / `validate_resolved_prefix` /
   `validate_xml_prefix_binding` cover the `xml`/`xmlns` rules correctly and are well tested.
6. **`xmlns=""` semantics.** `test_empty_default_namespace_removes_scope` and
   `rule.namespace-usage.empty-default-namespace` are handled correctly, including the subtle
   "the empty declaration applies to the element that bears it" case. The arena design must keep
   this behaviour and its test.
7. **Attribute resolution ignoring the default namespace** (`resolve_attribute`), including the
   automatic `xml:` binding (`resolve_xml_prefix`).
8. **The public doc-comment conventions** from AGENTS.md (intra-doc links resolve, `# Errors`
   and `# Panics` sections, no "`[Type]`s" plurals, `unsafe` justification comments).
9. **CI, `.coderabbit.yaml`, `Cargo.lock`, `specification/`** — all kept as-is, extended where needed.

## 4. Scope decisions taken by this review

### 4.1 Entity handling (relates to D1 and requirement (5))

Per AGENTS.md the library targets XML 1.0 with UTF-8 and performs **no DOCTYPE/DTD processing**.
That fixes the entity policy, and it is recorded here and in `PLAN.md` so that G3's
"every enforced rule carries its rule id" cannot silently grow into DTD support:

* **In scope:** the five predefined entities `amp`, `lt`, `gt`, `apos`, `quot` are expanded in
  content and in attribute values, at parse time and on serialization
  (`rule.entities.predefined-entities-recognized`, `rule.attributes.entity-refs-expanded`,
  `rule.well-formedness.escape-ampersand-and-lt`, `rule.attributes.char-refs-expanded`).
* **In scope:** character references (`&#…;`, `&#x…;`) are expanded by the parser and validated to
  denote legal characters (`rule.entities.charref-legal-character`,
  `rule.attributes.char-ref-legal-char`, `rule.well-formedness.char-ref-legal-char`).
* **In scope:** any other general entity reference (e.g. `&undefined;`, `&custom;`) is a **typed
  error**, never a panic — `rule.attributes.no-undeclared-entity-refs`,
  `rule.attributes.entity-declared-wfc`, `rule.entities.entity-declared-wfc`,
  `rule.attributes.no-external-entity-refs`, `rule.entities.unparsed-ref-forbidden`. (Without DTD
  processing there is no way for such a reference to be declared, so "not predefined ⇒ error" is
  the correct reading of these rules for this library.)
* **Out of scope:** every rule about the internal/external DTD subset, entity *declarations*,
  parameter entities, notations, unparsed entities, and validity — 93 of the 194 rule files, listed
  individually in `rule-inventory.md` with layer `X`.
* **Out of scope:** non-UTF-8 encodings (UTF-16, BOM-based detection beyond UTF-8, encoding-name
  conventions for other encodings). A declared encoding other than UTF-8 is rejected with a typed
  error rather than silently mis-decoded; that is stricter than "ignore the declaration" and is
  recorded as a documented limitation.

### 4.2 Locking and the `_checked` API split (relates to D3 and requirement (1))

Requirement (1) is read literally: `_checked` variants exist for **logical** failures (cycle
creation, cross-document attach, missing parent/root), while lock acquisition itself cannot fail
because there is exactly one lock and no re-entrancy. See `PLAN.md` §3 for the full argument.

### 4.3 Breaking changes

AGENTS.md states the project is experimental and unreleased, so the rewrite may change the public
API freely. The plan nevertheless keeps the well-tested names (`Document`, `Element`, `Namespace`,
`QualifiedName`, the `xml_spec` types, `parse_*`/`write_*`) so that the diff stays reviewable, and
G6 documents the migration in the docs book.
