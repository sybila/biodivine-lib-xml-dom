# biodivine-lib-xml-dom 0.2.0 — rewrite report

This is the hand-off for the rewrite of `biodivine-lib-xml-dom`. It says what was asked for, what
was built, how to build/test/document it, what is *verified* and what is not, and where the evidence
lives. The full verification transcript is [`docs/design/VERIFICATION.md`](docs/design/VERIFICATION.md);
the audit of the original code is [`docs/design/REVIEW.md`](docs/design/REVIEW.md); the design and
every deviation from it are in [`docs/design/PLAN.md`](docs/design/PLAN.md).

## What was built

The task asked for an XML DOM library that can be shared between threads, with an arena of nodes
behind one document-level lock, safe and ergonomic namespaces, integrity enforced partly by the type
system and partly by a whole-document validation pass, XML-spec-conformant behaviour, Python
bindings in three layers, rustdoc + Sphinx + a documentation book with switchable examples, and
disciplined version control. All of that is in place on the `rewrite` branch.

**The architecture, in two sentences.** A document owns an arena (`Vec<NodeSlot>`) behind a single
`RwLock`; node payloads refer to each other by index, so there are no reference cycles and no
per-node locks, and a `Node`/`Element` is just a document handle plus an index. Every operation
acquires that one lock exactly once, never re-enters it and never holds two of them — which is why
the library can be shared between threads without any possibility of deadlock, and why edits such as
"append this child if it does not create a cycle" are atomic rather than racy.

**Three layers.**

```text
biodivine-lib-xml-dom            the Rust library — no PyO3 code at all
biodivine-lib-xml-dom-py-sys     a thin PyO3 mirror of the Rust API, imported as `..._sys`
biodivine_lib_xml_dom            the pure-Python package: coercion, Pythonic protocols, conveniences
```

The mapping between the Rust API and the Python one, including what is deliberately *not* mirrored,
is itemised in [`docs/design/BINDINGS.md`](docs/design/BINDINGS.md).

## How to build, test and document it

```sh
make build              # cargo build --workspace
make test               # cargo test --workspace
make python-extension   # build the native extension into .venv (maturin develop --release)
make test-python        # pytest against the built extension
make examples           # run every Rust example the book includes
make lint               # cargo fmt --check, cargo clippy, pytest
make docs               # rustdoc + the Python API reference + the book
make verify             # every gate, with each command's exit code   <-- start here
```

`make verify` (i.e. [`scripts/verify.sh`](scripts/verify.sh)) is the entry point to reproduce the
evidence: it runs formatting, clippy (`-D warnings`), the workspace tests in debug and release, the
declared MSRV toolchain and the CI-pinned one, the examples, rustdoc, the documentation checkers and
builds, and pytest, printing each command with its exit code and failing if anything is red.
`docs/design/VERIFICATION.md` is generated from a run of it, and `docs/check_facts.py` (part of the
same gate) checks the numbers that document states against the artefacts.

## Evidence index

| where | what |
| --- | --- |
| `docs/design/VERIFICATION.md` | the generated transcript: gates with exit codes, test counts, the requirement-by-requirement table, concurrency evidence, MSRV, Miri, and the verified/not-verified split |
| `docs/design/REVIEW.md` | the audit of the 0.1 implementation: per-requirement status, eleven ranked defects, what was kept |
| `docs/design/PLAN.md` | the target architecture, the deadlock-freedom and panic-safety arguments, the API sketch, the test seams, the risks, and §15 with every deviation and its reason |
| `docs/design/BINDINGS.md` | the Python bindings audit: mirroring verdicts, error mapping, GIL policy, `Send`/`Sync` justifications |
| `docs/design/evidence/baseline.md` | the pre-rewrite gates |
| `docs/design/evidence/probes/` | eight single-command defect reproductions with raw transcripts, plus their status after the rewrite |
| `docs/design/evidence/rule-inventory.md` | all 194 specification rules with an enforcement layer |
| `docs/design/evidence/rule-enforcement.md` | a verdict (enforced / partial / deferred / n/a) for each of the 66 parser, validation and serializer rules |
| `docs/design/evidence/miri.md` | the Miri attempt, its output, and what it does and does not add |
| `docs/design/evidence/{docs,bindings}-build.md` | the docs and bindings build transcripts with tool versions |
| `docs/book` | the tutorial book: 12 chapters, 19 Rust/Python example pairs, all executed by the suites |

## Verified, and not verified

**Verified in this sandbox** (see the gates table in `docs/design/VERIFICATION.md`, all exit 0):
`cargo fmt --check`; `cargo clippy` with `-D warnings` for the workspace and with `--all-features` for
the core crate; `cargo test --workspace` in debug (250 tests) and release (249 — one test asserts the
debug-only re-entrancy guard); the same suite on the **declared MSRV, rustc 1.88.0**, and on the
**CI-pinned rustc 1.95.0**; `cargo build --examples` and all eight examples run; rustdoc with
`-D warnings` and `missing_docs`; the two documentation checkers (including a self-test that proves
they can fail); the docs builds (rustdoc, the Sphinx API reference, the book); and 41 Python tests
against a freshly built extension. Miri was run on the library tests (84/84 pass).

**Not verified here:**

* **No CI job ran.** GitHub Actions does not run in this sandbox; `.github/workflows/*` are
  unverified by execution. Their substance (toolchains, fmt, clippy, tests) is what `make verify`
  runs locally.
* **Miri cannot run the concurrency tests**: they abort inside `parking_lot_core`'s futex call, a
  Miri/`parking_lot` interaction (details in `docs/design/evidence/miri.md`). Miri therefore supports
  the data-structure claims but not the concurrency claim, which rests on the design and on the
  native stress tests.
* **No `abi3` wheel, no published wheel.** `maturin develop --release` builds for the interpreter it
  finds (CPython 3.11 here). `abi3`/`abi3t` are packaging decisions for a release
  (`docs/design/BINDINGS.md` §8).
* **The branch is local.** Nothing was pushed and `master` is untouched at `76beb74`.
* **No downstream project was rebuilt** against the new API, so compatibility is argued (names kept
  where possible) rather than measured.

## Deliberate limitations

The book has a *Known limitations and gotchas* chapter for users, and `PLAN.md` §15 records each with
its reason. In short:

* XML 1.0 + Namespaces 1.0, **UTF-8 only**, and **no DTD processing** — so no DTD-defined entities,
  attribute types or content models, and the two `xml:lang`/`xml:space` "must be declared" rules are
  validity and therefore out of scope (their value shapes *are* checked);
* comments and processing instructions outside the root element are discarded, and empty text nodes
  have no XML representation;
* **edits are silent** about namespaces and structure — `Document::validate()` is the only reporter,
  which is the deliberate trade-off that keeps edits fast and predictable, and the serializer never
  invents or removes a declaration;
* arena slots are never reclaimed, so `node_count()` only grows and `NodeId`s never dangle;
* attribute order is not preserved and there is no output formatting.

## Commits on `rewrite`

conventional-commit style, each self-contained and green; nothing pushed, `master` untouched. The
list is `git log --oneline rewrite ^master` and the count is `git rev-list --count rewrite ^master`
(not written down here, because every commit changes it); at the transcript commit it was 29
(`git rev-list --count 8254b3b ^master` = 29), and `docs/check_facts.py` verifies the numbers that
*are* written down - commit counts for named commits, gate and test counts, rule and book counts,
and the version - every time the gate runs. The sequence is: the review and plan (`docs: add baseline verification evidence and defect
probes`, `docs: classify all 194 specification rules by enforcement layer`, `docs: audit the current
implementation against the requirements`, `docs: add the long-term design and implementation plan`),
the core rewrite (`refactor(xml_spec): back content newtypes with Arc<str>`, `feat: replace per-node
locks with a document-level arena`, `test: cover structural editing, clone semantics and
concurrency`, `fix: make self-replacement a no-op and audit every # Errors section`, …), the spec and
I/O work (`feat(xml_spec): add the declaration type, value predicates and a rule anchor module`,
`feat(io): rewrite the parser and serializer`, `test(io): add round-trip, idempotence and no-panic
property tests`, …), validation (`feat: add whole-document validation`), the bindings
(`feat: add the Python bindings (py-sys crate and pure-Python package)`, plus the interner fix that
building them exposed), the documentation (`docs: deny missing_docs and check the Errors/Panics
conventions mechanically`, `docs: add the Python API reference`, `docs: add the documentation book
with Rust and Python examples`, `docs: add the README, the docs build script and the build
transcript`) and the release numbers (`build: make the release numbers single-source and verify the
declared MSRV`). `git log --oneline` on the branch is the authoritative list.
