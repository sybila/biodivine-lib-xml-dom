# Probe results — pre-rewrite `master`

Every probe below is runnable by a third party with a single command and reproduces against
commit `76beb74` ("Merge pull request #1 from sybila/feat/daemontus/initial-implementation").

```
docs/design/evidence/probes/run_probe.sh docs/design/evidence/probes/<probe-file>
```

The runner prints the commit, `rustc --version`, `cargo --version` and a UTC timestamp in its
header, then runs the probe and prints `PROBE_EXIT=<code>`. Raw, unedited transcripts are kept
next to this file in `probes/raw/<probe>.txt`.

**Toolchain used for the recorded transcripts**

```
$ rustc --version
rustc 1.99.0 (b940084d7 2026-09-28)
$ cargo --version
cargo 1.99.0 (5f94df478 2026-08-27)
```

The repository's own CI pins `rust-version: 1.95.0` / `min-rust-version: 1.88.0`
(`.github/workflows/build.yml`); the baseline also passes unchanged on `cargo +1.95.0 test`
(see `baseline.md`), so nothing recorded here is an artefact of a newer compiler. Version 1.95.0
was installed with `rustup toolchain install 1.95.0 --profile minimal -c clippy,rustfmt` and is
kept for the MSRV gate in G7.

**Probe files are not part of the crate's own test suite.** They reproduce defects of the
*pre-rewrite* implementation; the runner copies them into `tests/__probe.rs` (or, for probes that
need private items, into `src/<module>/__probe.rs`) and always restores the tree afterwards — it
never uses `git checkout`, so uncommitted work is safe. All probes that demonstrate a defect are
*expected* to exit non-zero; that failure is the evidence.

## Summary

| probe | kind | hypothesis | observed | verdict |
| --- | --- | --- | --- | --- |
| `it_prefix_loss` | integration test | the serializer preserves element prefixes | `<html:body>` is written as `<body>` | **defect confirmed** |
| `it_entity_panic` | integration test | the parser never panics on input | panic at `src/io.rs:127` for `&amp;` **and** for `&undefined;` | **defect confirmed** |
| `unit_element_arc_cycle_leak` | crate-internal test | dropping a document frees its nodes | `strong_count(child) == 2` after drop | **defect confirmed** |
| `it_cycle_race` | integration test | concurrent edits cannot create a cycle | 37170 cycles / 128000 attempted pairs | **defect confirmed** |
| `it_ns_silent_loss` | integration test | namespace information survives a round trip | `<r ex:a="v"></r>`, `<r></r>` — declarations and prefixes dropped | **defect confirmed** |
| `cf_missing_api` | compile-fail | the required API surface exists | 12 × `E0599`/`E0425` "no method named …" | **gap confirmed** |
| `sh_python_bindings` | shell | PyO3 / `-py-sys` / Python package exist | no `pyo3` reference, single workspace member, no Python files | **gap confirmed** |
| `sh_docs_and_repo_state` | shell | docs book, MSRV pin, clean repo exist | no book, no `rust-version`, no `rust-toolchain.toml`, no `rewrite` branch, 0-byte `output.xml`, 184-line `src/main.rs` | **gap confirmed** |

---

## 1. `it_prefix_loss` — serializer drops element prefixes

Command: `docs/design/evidence/probes/run_probe.sh docs/design/evidence/probes/it_prefix_loss.rs`
Raw output: `probes/raw/it_prefix_loss.txt`, exit `101`.

```
output: <html xmlns:html="http://www.w3.org/1999/xhtml"><body>hi</body></html>
thread 'serializer_preserves_element_prefix' panicked at tests/__probe.rs:25:5:
error: test failed
```

* Expected: `<html:body>` remains `<html:body>` after a `parse → write` round trip.
* Observed: the element prefix is gone; only the (now useless) namespace declaration survives.
* Cause: `src/io.rs::write_element()` derives both the start tag and the end tag from
  `qname.local_name()` only and never consults `qname.namespace()`, while the very same function
  *does* prepend the prefix when writing attribute names.

## 2. `it_entity_panic` — parser aborts on entity references

Command: `docs/design/evidence/probes/run_probe.sh docs/design/evidence/probes/it_entity_panic.rs`
Raw output: `probes/raw/it_entity_panic.txt`, exit `101`.

```
test parser_does_not_panic_on_predefined_entities ... !!! panic intercepted: panicked at src/io.rs:127:17
thread 'parser_does_not_panic_on_predefined_entities' panicked at tests/__probe.rs:39:19:
parser panicked instead of returning Ok(_) or Err(_)
test undeclared_entity_is_a_typed_error ... !!! panic intercepted: panicked at src/io.rs:127:17
thread 'undeclared_entity_is_a_typed_error' panicked at tests/__probe.rs:57:19:
parser panicked instead of returning a typed error
```

* Expected: `<a>AT&amp;T</a>` parses (XML 1.0 §4.6 — the five predefined entities must be
  recognised whether declared or not); an undeclared reference such as `&undefined;` returns a
  typed error.
* Observed: both panic on `src/io.rs:127`, which is
  `Event::GeneralRef(_) => unimplemented!("Custom entities are currently not supported.")`.
* Impact: the process dies. In the PyO3 layer this would abort the Python interpreter.

## 3. `unit_element_arc_cycle_leak` — parent↔child `Arc` reference cycle

Command: `docs/design/evidence/probes/run_probe.sh docs/design/evidence/probes/unit_element_arc_cycle_leak.rs`
Raw output: `probes/raw/unit_element_arc_cycle_leak.txt`, exit `101`.

```
before drop: strong_count(child) = 2, strong_count(parent) = 3
after dropping the document and the parent handle:
  strong_count(child) = 2
  expected            1 (only the `child` handle above)
```

* Expected: after the `Document` and the user's `Element` handle are dropped, only the local
  `child` handle keeps the child alive (`strong_count == 1`).
* Observed: `strong_count == 2` — the parent's `children` vector still strongly owns the child,
  and the child's `parent` field still strongly owns the parent. The whole subtree is leaked,
  including for any program that builds a document and drops it.
* Cause: strong references in both directions between `Arc<RwLock<ElementData>>` values.

## 4. `it_cycle_race` — concurrent edits create cycles

Command: `docs/design/evidence/probes/run_probe.sh docs/design/evidence/probes/it_cycle_race.rs`
Raw output: `probes/raw/it_cycle_race.txt`, exit `101`.

```
loops created: 37170 out of 128000 attempted parent/child pairs
thread 'concurrent_edits_must_not_create_cycles' panicked:
assertion `left == right` failed: concurrent edits created parent/child cycles
```

* Expected: zero cycles. `add_child_element` already returns an error when the operation would
  create one (commit 6bf19e3 fixed that for the single-threaded case).
* Observed: ~29 % of attempts succeed in creating `a → b → a`.
* Cause: `src/element.rs::add_child_element` is a check-then-act sequence:
  `child.0.read().parent.is_some()` → `child.is_ancestor(self)` → `child.0.write().parent = …`
  → `self.0.write().children.push(…)`. Per-node locks are released between the steps, and two
  threads working on two *different* children (`a.add_child(b)` vs. `b.add_child(a)`) each pass
  their own "parent is none" check on a different field.
* Secondary impact: once a cycle exists, `is_ancestor` (and therefore `add_child_element` and
  `is_attached`) walks it forever.
* Why the fix is architectural and not a patch: making this atomic with per-node locks requires
  acquiring every lock along the ancestor chain in a globally consistent order, which is exactly
  the granular-locking complexity the task description calls out. A single document-level lock
  makes the entire sequence one critical section.

## 5. `it_ns_silent_loss` — namespaces are silently dropped on serialization

Command: `docs/design/evidence/probes/run_probe.sh docs/design/evidence/probes/it_ns_silent_loss.rs`
Raw output: `probes/raw/it_ns_silent_loss.txt`, exit `101`.

```
thread 'default_namespace_without_declaration_survives_round_trip' panicked:
namespace not declared in the output: <r></r>
thread 'prefixed_namespace_without_declaration_survives_round_trip' panicked:
prefix `ex` not declared in the output: <r ex:a="v"></r>
```

* Expected: either the namespace is declared in the output, or the document is reported as
  invalid *before* serialization; requirement (3) explicitly delegates this to validation.
* Observed: the output is written as if the namespaces did not exist, and re-parsing yields a
  different document. There is no `Document::validate()`, and the serializer performs no checks.
* This is the concrete failure mode behind requirement (3)'s "moving elements breaks namespace
  declarations, detect it during document validation".

## 6. `cf_missing_api` — API surface that does not exist

Command: `docs/design/evidence/probes/run_probe.sh docs/design/evidence/probes/cf_missing_api.rs`
Raw output: `probes/raw/cf_missing_api.txt`, exit `101` (compile failure, as intended).

```
error[E0425]: cannot find type `Node` in crate `biodivine_lib_xml_dom`
error[E0599]: no method named `deep_clone` found for reference `&Element`
error[E0599]: no method named `shallow_clone` found for reference `&Element`
error[E0599]: no method named `deep_clone_into` found for reference `&Element`
error[E0599]: no method named `remove` found for reference `&Element`
error[E0599]: no method named `detach` found for reference `&Element`
error[E0599]: no method named `replace_with` found for reference `&Element`
error[E0599]: no method named `insert_child` found for reference `&Element`
error[E0599]: no method named `namespaces_in_scope` found for reference `&Element`
error[E0599]: no method named `remove_namespace_declaration` found for reference `&Element`
error[E0599]: no method named `validate` found for reference `&Document`
error[E0599]: no method named `belongs_to` found for reference `&Element`
error: could not compile `biodivine-lib-xml-dom` (test "__probe") due to 12 previous errors
```

* Expected: the API required by requirements (1)–(4) exists.
* Observed: 12 distinct missing items. Notably there is **no** way to remove, detach, insert or
  replace a node once it is attached, and no way to copy a subtree into another document.

## 7. `sh_python_bindings` — no PyO3 layer

Command: `docs/design/evidence/probes/run_probe.sh docs/design/evidence/probes/sh_python_bindings.sh`
Raw output: `probes/raw/sh_python_bindings.txt`, exit `0` (nothing found, which is the finding).

```
--- grep -ri pyo3 (rust sources + manifests)
  (none)
--- cargo metadata workspace members
  biodivine-lib-xml-dom
--- look for a -py-sys crate or python package
--- python packages installed in this environment matching the library
  biodivine_lib_xml_dom_sys: None
  biodivine_lib_xml_dom: None
--- any Python source/test files in the repo
```

## 8. `sh_docs_and_repo_state` — docs book, MSRV and repository hygiene

Command: `docs/design/evidence/probes/run_probe.sh docs/design/evidence/probes/sh_docs_and_repo_state.sh`
Raw output: `probes/raw/sh_docs_and_repo_state.txt`, exit `0`.

Findings:

* no `conf.py`/`index.rst`/`mkdocs.yml`/`book.toml` anywhere — requirement (7)'s docs book does not exist;
* no `missing_docs` lint and no rustdoc-denied-warnings configuration;
* no `rust-toolchain.toml` and no `rust-version` key in `Cargo.toml`, although CI pins
  `rust-version: 1.95.0` / `min-rust-version: 1.88.0` — the MSRV is undeclared where tooling
  (`cargo msrv`, dependents, docs.rs) would look for it;
* `output.xml` (0 bytes) is a stray artefact committed in the crate root, and `src/main.rs`
  (184 lines) is a demo binary living inside a library crate instead of an `examples/` target;
* `git branch -a` shows only `master` (+ `origin/master`) — the requested `rewrite` branch does not exist;
* `.coderabbit.yaml` and `.github/workflows/{build,release}.yml` exist (reused `sybila/github-workflows`
  reusable workflows pinned to `@main`), and `.gitignore` covers only `/target` and `.idea`.

---

## Status after G3 (I/O rewrite)

Re-run with the same runner; fresh transcripts are saved next to the originals as
`probes/raw/<probe>.after-g3.txt`. Three probes target parse/serialize and were re-run; the other
five describe parts of the system that have since been replaced, and are marked accordingly below.

| probe | after G3 | evidence |
| --- | --- | --- |
| `it_prefix_loss` | **fixed** — `PROBE_EXIT=0` | `output: <html:html xmlns:html="http://www.w3.org/1999/xhtml"><html:body>hi</html:body></html:html>` |
| `it_entity_panic` | **fixed** — `PROBE_EXIT=0` | `OK: parsed and re-serialized as "<a>AT&amp;T</a>"` and `OK: typed error returned: undeclared entity reference \`&undefined;\`` |
| `it_ns_silent_loss` | **still fails, by design** — `PROBE_EXIT=101` | `serialized: <ex:r ex:a="v"/>` — the serializer writes the prefixes and declarations that are stored in the tree and never invents one (requirement (3)). Detecting the resulting inconsistency is the job of whole-document validation (G4), not of the serializer. |
| `it_cycle_race` | superseded — the scenario is now the permanent test `tests/concurrency.rs::two_threads_adding_opposite_children_never_create_a_cycle` (0 cycles, exactly one winner per pair). It no longer compiles because `add_child_element` was replaced by `append_child`/`append_child_checked`. |
| `cf_missing_api` | superseded — most of the listed items now exist (`remove`, `detach`, `replace_with`, `insert_child`, `deep_clone`, `deep_clone_into`, `namespaces_in_scope`, `remove_namespace_declaration`, `belongs_to`, `Node`). The remaining one (`Document::validate`) is G4. |
| `sh_python_bindings` | unchanged — Python bindings are G5. |
| `sh_docs_and_repo_state` | partially outdated — the `rewrite` branch, the removal of `output.xml` and the move of `src/main.rs` to `examples/tour.rs` are done; the docs book and the MSRV pin are G6/G7. |

Two probes were adapted by one line each to keep them runnable against the new API
(`add_attribute` → `set_attribute`; the `set_root` call no longer returns a `Result`, it returns the
previous root). Their original transcripts from the pre-rewrite tree are unchanged and still in
`probes/raw/<probe>.txt`.
