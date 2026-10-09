# Verification

This document is the evidence that the rewritten library does what the task asked for. It is
**generated from a run of one command** - `scripts/verify.sh`, also `make verify` - so it and the
artefact cannot disagree:

```sh
scripts/verify.sh          # runs every gate below and prints each command with its exit code
```

The run recorded here was made on the final commit of the `rewrite` branch, in this sandbox.

## Environment

```
=== environment
rustc 1.99.0 (b940084d7 2026-09-28)
cargo 1.99.0 (5f94df478 2026-08-27)
rustc 1.88.0 (6b00bc388 2025-06-23)
rustc 1.95.0 (59807616e 2026-04-14)
Python 3.11.2
maturin 1.15.0
pytest 9.1.1
sphinx 9.0.4
```

## Gates

| command | exit |
| --- | --- |
| `cargo fmt --check` | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| `cargo clippy -p biodivine-lib-xml-dom --all-targets --all-features -- -D warnings` | 0 |
| `cargo test --workspace` | 0 |
| `cargo test --workspace --release` | 0 |
| `cargo +1.88.0 test --workspace` | 0 |
| `cargo +1.95.0 test --workspace` | 0 |
| `cargo build --examples` | 0 |
| `cargo run --quiet --example book_editing` | 0 |
| `cargo run --quiet --example book_getting_started` | 0 |
| `cargo run --quiet --example book_namespaces` | 0 |
| `cargo run --quiet --example book_parsing` | 0 |
| `cargo run --quiet --example book_pythonic` | 0 |
| `cargo run --quiet --example book_threads` | 0 |
| `cargo run --quiet --example book_validation` | 0 |
| `cargo run --quiet --example tour` | 0 |
| `env RUSTDOCFLAGS=-D warnings cargo doc --no-deps --workspace` | 0 |
| `/sandbox/biodivine-lib-xml-dom/.venv/bin/python docs/check_doc_sections.py --self-test` | 0 |
| `/sandbox/biodivine-lib-xml-dom/.venv/bin/python docs/check_doc_sections.py` | 0 |
| `/sandbox/biodivine-lib-xml-dom/.venv/bin/python docs/check_book.py` | 0 |
| `bash docs/build_docs.sh` | 0 |
| `/sandbox/biodivine-lib-xml-dom/.venv/bin/python docs/check_book.py --built` | 0 |
| `/sandbox/biodivine-lib-xml-dom/.venv/bin/python -m pytest biodivine-lib-xml-dom-py-sys/tests-python` | 0 |

All 23 gates passed. `--all-features` is applied to the **core** crate only: the binding
crate's `extension-module` feature is deliberately enabled by maturin for wheel builds, and turning
it on for `cargo clippy` would check the configuration that is explicitly *not* the in-process-test
one. Clippy runs with `-D warnings`, so "0 warnings" is a gate rather than a snapshot.

## Test counts

| suite | tests |
| --- | --- |
| `cargo test --workspace` | 250 |
| `cargo test --workspace --release` | 249 |
| `cargo +1.88.0 test --workspace` | 250 |
| `cargo +1.95.0 test --workspace` | 250 |
| `pytest` (Python bindings) | 41 |

The suites break down as: 83 lib tests in release (84 in debug - one test asserts the debug-only
re-entrancy guard, which compiles away), 13 `errors`, 15 `cloning`, 5 `concurrency`, 9 `element`,
35 `io`, 7 `properties` (`proptest`: 256-case round-trip and validation properties, 512-case
no-panic properties over arbitrary bytes/strings/mutations), 28 `structure`, 21 `validation`,
6 in-process binding tests, 27 doctests, and 41 Python tests.

The full transcript, including every command's output, is at the end of this document.

## Requirement by requirement

Statuses are deliberately narrow: "satisfied" means the acceptance criterion has a test or a gate
that demonstrates it, and anything narrower is stated as such.

| # | requirement | status | where |
| --- | --- | --- | --- |
| (1) | one document shared between threads; arena of nodes plus **one** document-level lock; `Node`/`Element` as ids; deep/shallow/handle clones; `Arc` dedup kept | satisfied | `src/document.rs` (the only `RwLock` in the crate, `DocumentInner`), `src/arena.rs` (`Arena`, `NodeId`, `Snapshot`), `src/interner.rs`; tests: `tests/concurrency.rs` (4 stress tests with a 30 s watchdog), `document::tests::handles_are_send_and_sync`, `document::tests::re_entrant_access_to_the_same_document_panics_in_debug_builds`, `tests/cloning.rs`, `interner::tests::*` |
| (2) | several documents; nodes bound to a document; attach/detach; copy between documents | satisfied | `src/node.rs` (`detach`, `remove`, `append_child`, `insert_*`, `replace_with`, `deep_clone_into`), `src/error.rs::ForeignDocument`; tests: `tests/structure.rs::foreign_documents_are_rejected`, `tests/cloning.rs::deep_clone_into_another_document_rebuilds_the_tree_there`, `tests/structure.rs::a_detached_subtree_stays_editable_clonable_and_reattachable` |
| (3) | safe, ergonomic namespaces; no magic; breakage found by validation | satisfied | `src/namespace.rs`, `src/qualified_name.rs` (expanded names, resolution, `xmlns=""`), `src/element.rs` (declaration API), serialization of exactly the stored declarations in `src/io/write.rs`; tests: `tests/element.rs`, `tests/io.rs::namespaces_are_resolved_and_reported`, `tests/validation.rs::editing_is_silent_and_validation_is_what_reports` |
| (4)(1) | low-level integrity enforced by construction | satisfied | `src/xml_spec.rs` (six validated newtypes), `src/arena.rs` (attributes keyed by expanded name), the parser's construction path; tests: `xml_spec::tests::*` (each anchored to a `specification/rules/` file), `tests/errors.rs` (one per documented error condition), `tests/io.rs` |
| (4)(2) | whole-document integrity, all issues at once | satisfied | `src/validation.rs`, `src/xml_spec/validation.rs`, `Document::validate`/`is_valid`; tests: `tests/validation.rs` (21 tests, including `all_problems_are_reported_in_one_call` with 4 issues, the `xml:id` matrix, and `generated_documents_validate_clean` as a 256-case no-false-positive property) |
| (5) | XML-spec compliance, rule annotations, `xml_spec` convention | satisfied **within the declared scope** | XML 1.0 + Namespaces 1.0, UTF-8 only, no DTD processing (`AGENTS.md`). Rules live in `src/xml_spec*`, each annotation naming its rule file, and `docs/design/evidence/rule-enforcement.md` gives a verdict for **all 194 rule files**: 30 local, 55 parser, 9 validation, 2 serializer, 5 not applicable, 93 deliberately out of scope (DTD validity, non-UTF-8 encodings). Of the 66 B/C/D rules: 54 enforced, 6 partial (the DTD half of a rule whose non-DTD half is implemented), 4 deferred (UTF-16, DTD validity), 2 not applicable. Tests: `tests/io.rs::rules_referenced_by_the_parser_and_serializer_exist`, `xml_spec::rules::tests`, `tests/validation.rs::every_error_kind_names_an_existing_rule_file` |
| (6) | PyO3 bindings in three layers | satisfied | core crate has no PyO3 (`cargo tree -p biodivine-lib-xml-dom --edges normal` lists only `parking_lot`, `quick-xml`, `thiserror`), `biodivine-lib-xml-dom-py-sys` mirrors the API, `python/biodivine_lib_xml_dom` is the idiomatic layer; audit in `docs/design/BINDINGS.md`; tests: `tests-python/` (41 pytest cases) plus 6 in-process binding tests |
| (7) | rustdoc + Sphinx + a book with switchable Rust/Python examples | satisfied | `missing_docs` is a workspace lint, `docs/check_doc_sections.py` (145 core + 156 binding public functions, floors enforced and self-tested), `biodivine-lib-xml-dom-py-sys/docs` (Sphinx autodoc, 0 warnings), `docs/book` (12 chapters, 19 Rust/Python pairs, `docs/check_book.py`); every example is a real file executed by the suites |
| (8) | versioning, clippy, formatting, small commits on a `rewrite` branch | satisfied | `cargo fmt --check` and `cargo clippy ... -D warnings` are gates; one version literal in `[workspace.package]` asserted from Python; `rust-version` declared and verified; conventional commits on `rewrite`, nothing pushed, `master` untouched |

## Concurrency evidence

| guarantee | evidence |
| --- | --- |
| exactly one lock per document, and nothing else | `grep -rn "RwLock<" src/` → one declaration (`src/document.rs`, `DocumentInner::arena`); `src/arena.rs` and every other module take `&Arena`/`&mut Arena` |
| no re-entrancy, hence no deadlock | the lock is acquired only in `read_arena`/`write_arena`; a debug-only guard keyed by document identity panics on re-entry (`document::tests::re_entrant_access_to_the_same_document_panics_in_debug_builds`) and tolerates nesting across *different* documents (`document::tests::nesting_access_to_two_different_documents_is_allowed`) |
| no two locks are ever held | cross-document copies snapshot the source, release it, then write to the target (`Node::copy_into`); `tests/cloning.rs::copying_while_the_source_is_mutated_produces_consistent_copies` exercises it against a concurrent mutator |
| edits are atomic | `tests/concurrency.rs::structural_edits_stay_consistent_under_contention` (4 threads, 300 iterations, invariants re-derived afterwards) and `readers_and_writers_do_not_deadlock` (6 threads, readers and writers, 30 s watchdog) |
| the old cycle race is gone | `tests/concurrency.rs::two_threads_adding_opposite_children_never_create_a_cycle` reports **0 cycles** and exactly one winner per opposing pair (500 rounds x 256 pairs); the pre-rewrite probe `docs/design/evidence/probes/it_cycle_race.rs` created 37170/128000 |
| handles are `Send + Sync` | `document::tests::handles_are_send_and_sync` (compile-time, all handle types) and `py-sys src/tests.rs::handles_are_send_and_sync` (`Py<...>` too) |
| Python inherits the guarantees | the GIL is released around parse/write/validate/cross-document copies (`BINDINGS.md` §3); `tests-python/test_book_examples.py::threads` shares one document between four Python threads with a 30 s join timeout |

## MSRV

`rust-version = "1.88"` is declared in `[workspace.package]` and inherited by both crates, so it sits
where tooling looks for it, and it matches the CI workflow's `min-rust-version: 1.88.0`. It is
**verified, not aspirational**: `rustup toolchain install 1.88.0` was run in this sandbox and
`cargo +1.88.0 test --workspace` passes the full suite (see the gates table). The CI-pinned
`1.95.0` is verified the same way. Nothing had to be raised.

## Miri

Attempted, as promised in `PLAN.md` §3.4. Full evidence and raw output: `docs/design/evidence/miri.md`.

* **Passes**: `MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri test --lib` → 84/84 library
  tests, no undefined behaviour. (`-Zmiri-disable-isolation` is required only because a *test
  helper* reads the rule files; no library code touches the filesystem.)
* **Does not run**: the lock-contending tests abort inside `parking_lot_core`'s futex call, because
  Miri's `SYS_futex` shim rejects the argument type `parking_lot` passes. That is a
  Miri/`parking_lot` interaction, not a finding about this crate.
* Consequence: Miri supports the *data-structure* claims (arena bookkeeping, interning, snapshots,
  validation, the re-entrancy guard) and contributes nothing to the *concurrency* claim, which rests
  on the design (one lock, never re-entered, never two held) plus the native stress tests.

There is no `unsafe` in `biodivine-lib-xml-dom` (`grep -rn unsafe src/` finds nothing outside doc
comments), so Miri's remaining value is in dependencies and in the arena's index bookkeeping.

## Verified and not verified

**Verified by running** (this sandbox, both toolchains): every gate in the table above, the whole
Rust test suite in debug *and* release, the Python suite against a rebuilt extension, the docs builds
(rustdoc, Sphinx API reference, the book), all 7 book examples and the `tour` example, the two
documentation checkers including their self-test, and Miri on the library tests.

**Not verified here, and why**:

* **No CI job was executed.** `.github/workflows/{build,release}.yml` delegate to reusable
  `sybila/github-workflows`; GitHub Actions does not run in this sandbox, so the workflows are
  unverified by execution. What they assert (the toolchain versions, fmt/clippy/test) is exactly what
  `scripts/verify.sh` runs locally, including on the pinned toolchains.
* **No wheel was published and no `abi3` wheel is produced.** `maturin develop --release` builds for
  the interpreter it finds (CPython 3.11 here); `abi3`/`abi3t` are packaging decisions recorded in
  `docs/design/BINDINGS.md` §8, not taken.
* **The branch is local.** 27 commits on `rewrite`, nothing pushed, no remote `rewrite`, `master`
  untouched at `76beb74`.
* **`tests/io.rs`, `tests/properties.rs` and the integration suites were not run under Miri** (see
  above).
* **Downstream compatibility is asserted, not measured**: no project outside this repository was
  rebuilt against the new API.

## Deliberate limitations

Listed for users in the book's *Known limitations and gotchas* chapter and for reviewers in
`PLAN.md` §15 (with a reason per entry). The short version: XML 1.0 + UTF-8 only and no DTD
processing; top-level comments and processing instructions are discarded; empty text nodes have no
XML representation; edits are deliberately silent about namespaces and structure, with
`validate()` as the only reporter; the serializer never invents or removes declarations; arena slots
are never reclaimed; attribute order is not preserved; there is no output formatting.

## Transcript

```
=== environment
rustc 1.99.0 (b940084d7 2026-09-28)
cargo 1.99.0 (5f94df478 2026-08-27)
rustc 1.88.0 (6b00bc388 2025-06-23)
rustc 1.95.0 (59807616e 2026-04-14)
Python 3.11.2
maturin 1.15.0
pytest 9.1.1
sphinx 9.0.4

$ cargo fmt --check
-> exit 0

$ cargo clippy --workspace --all-targets -- -D warnings
   Compiling pyo3-ffi v0.29.3
    Checking biodivine-lib-xml-dom v0.2.0 (/sandbox/biodivine-lib-xml-dom)
   Compiling pyo3 v0.29.3
    Checking biodivine-lib-xml-dom-py-sys v0.2.0 (/sandbox/biodivine-lib-xml-dom/biodivine-lib-xml-dom-py-sys)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.18s
-> exit 0

$ cargo clippy -p biodivine-lib-xml-dom --all-targets --all-features -- -D warnings
    Checking biodivine-lib-xml-dom v0.2.0 (/sandbox/biodivine-lib-xml-dom)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.14s
-> exit 0

$ cargo test --workspace
   Compiling biodivine-lib-xml-dom v0.2.0 (/sandbox/biodivine-lib-xml-dom)
   Compiling pyo3-ffi v0.29.3
   Compiling pyo3 v0.29.3
   Compiling biodivine-lib-xml-dom-py-sys v0.2.0 (/sandbox/biodivine-lib-xml-dom/biodivine-lib-xml-dom-py-sys)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.77s
     Running unittests src/lib.rs (target/debug/deps/biodivine_lib_xml_dom-6f2b08f50d2a4b44)

running 84 tests
test arena::tests::attach_and_detach_maintain_both_directions ... ok
test arena::tests::exceeding_the_arena_capacity_panics_instead_of_wrapping - should panic ... ok
test arena::tests::attached_root_cannot_be_reset_as_root ... ok
test arena::tests::detaching_a_detached_node_is_a_no_op ... ok
test arena::tests::cyclic_attachments_are_rejected ... ok
test arena::tests::relative_insertion ... ok
test arena::tests::namespaces_in_scope_shadow_outer_declarations ... ok
test arena::tests::out_of_range_index_is_an_error_and_changes_nothing ... ok
test arena::tests::replace_keeps_links_consistent ... ok
test arena::tests::replace_by_self_is_a_no_op ... ok
test arena::tests::replace_rejects_an_ancestor_of_the_replaced_node ... ok
test arena::tests::root_cannot_be_attached ... ok
test arena::tests::the_capacity_guard_allows_everything_below_the_limit ... ok
test arena::tests::snapshot_round_trip_preserves_the_subtree ... ok
test arena::tests::the_index_is_interpreted_after_detaching ... ok
test document::tests::documents_are_equal_by_identity ... ok
test document::tests::empty_documents_have_no_root ... ok
test document::tests::handles_are_send_and_sync ... ok
test document::tests::nesting_access_to_two_different_documents_is_allowed ... ok
test interner::tests::interning_deduplicates_by_value ... ok
test document::tests::re_entrant_access_to_the_same_document_panics_in_debug_builds - should panic ... ok
test namespace::tests::test_namespace_equality ... ok
test interner::tests::interning_never_rewrites_a_prefix ... ok
test namespace::tests::test_namespace_is_equal_ns ... ok
test namespace::tests::test_namespace_support ... ok
test namespace::tests::test_unicode_prefixes ... ok
test node::tests::interning_does_not_change_equality ... ok
test node::tests::re_interns_names_in_the_target_document ... ok
test qualified_name::tests::test_creation_and_error ... ok
test qualified_name::tests::test_equality_and_ordering ... ok
test qualified_name::tests::test_hashing_semantic_equality ... ok
test qualified_name::tests::test_ord_consistent_with_partial_eq ... ok
test qualified_name::tests::test_qualified_name_string_no_prefix_ns ... ok
test qualified_name::tests::test_resolve_attribute_ignores_default_ns ... ok
test qualified_name::tests::test_resolve_attribute_with_map_ignores_default_ns ... ok
test qualified_name::tests::test_resolve_attribute_with_map_prefixed ... ok
test qualified_name::tests::test_resolve_attribute_with_prefix ... ok
test qualified_name::tests::test_resolve_no_prefix ... ok
test qualified_name::tests::test_resolve_attribute_xml_prefix_auto ... ok
test qualified_name::tests::test_resolve_undefined_prefix ... ok
test qualified_name::tests::test_resolve_with_map_prefixed ... ok
test qualified_name::tests::test_resolve_with_map_undefined_prefix ... ok
test qualified_name::tests::test_resolve_with_map_xml_prefix_auto ... ok
test qualified_name::tests::test_resolve_with_prefix ... ok
test qualified_name::tests::test_resolve_with_parent_ns ... ok
test qualified_name::tests::test_resolve_xml_prefix_auto ... ok
test qualified_name::tests::test_resolve_xmlns_prefix_rejected ... ok
test tests::test_add_children ... ok
test tests::test_create_document ... ok
test tests::test_create_element ... ok
test tests::test_document_reference ... ok
test tests::test_dropping_a_document_frees_its_nodes ... ok
test tests::test_namespace_declaration ... ok
test tests::test_qualified_name_resolution ... ok
test xml_spec::declaration::tests::accessors ... ok
test xml_spec::declaration::tests::declaration_rendering ... ok
test xml_spec::declaration::tests::encoding_names_are_case_insensitive ... ok
test xml_spec::declaration::tests::pseudo_attributes ... ok
test xml_spec::g3_tests::content_that_cannot_be_escaped_is_normalised_at_construction ... ok
test xml_spec::g3_tests::language_tags ... ok
test xml_spec::g3_tests::line_end_normalisation ... ok
test xml_spec::g3_tests::xml_space_values ... ok
test xml_spec::tests::test_cdata_wrapper ... ok
test xml_spec::rules::tests::the_rules_directory_is_where_we_think_it_is ... ok
test xml_spec::tests::test_namespace_validation ... ok
test xml_spec::tests::test_comment_wrapper ... ok
test xml_spec::tests::test_ncname_helper ... ok
test xml_spec::tests::test_pi_data_wrapper ... ok
test xml_spec::tests::test_pi_target_wrapper ... ok
test xml_spec::tests::test_prefix_ncname_validation ... ok
test xml_spec::tests::test_reserved_xml_prefix ... ok
test xml_spec::tests::test_reserved_xmlns_prefix ... ok
test xml_spec::tests::test_split_qname_invalid ... ok
test xml_spec::tests::test_split_qname_valid ... ok
test xml_spec::validation::tests::attributes_ignore_the_default_namespace ... ok
test xml_spec::tests::test_uri_comparison_case_sensitive ... ok
test xml_spec::validation::tests::prefixed_names_must_be_declared ... ok
test xml_spec::tests::test_text_wrapper ... ok
test xml_spec::tests::test_validate_resolved_prefix ... ok
test xml_spec::validation::tests::default_namespace_rules ... ok
test xml_spec::validation::tests::inner_declarations_shadow_outer_ones ... ok
test xml_spec::validation::tests::the_xml_prefix_needs_no_declaration ... ok
test xml_spec::validation::tests::the_xmlns_prefix_can_never_become_a_name_prefix ... ok
test node::tests::repeated_names_share_one_allocation ... ok

test result: ok. 84 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/cloning.rs (target/debug/deps/cloning-1afb5a6b24db666c)

running 15 tests
test cloning_a_handle_refers_to_the_same_node ... ok
test a_deep_clone_is_cheap_because_names_are_shared ... ok
test a_deep_copy_can_be_edited_independently_of_its_source ... ok
test a_copied_subtree_can_be_attached_in_the_target_document ... ok
test display_of_non_element_nodes ... ok
test deep_clone_into_the_same_document_behaves_like_deep_clone ... ok
test deep_clone_into_another_document_rebuilds_the_tree_there ... ok
test deep_clone_copies_the_whole_subtree ... ok
test display_renders_the_subtree_as_xml ... ok
test non_element_nodes_can_be_copied_between_documents ... ok
test node_content_exposes_every_kind ... ok
test shallow_clone_of_a_text_node_copies_the_content ... ok
test shallow_clone_copies_the_payload_but_not_the_children ... ok
test shallow_clone_into_another_document_keeps_only_the_node ... ok
test copying_while_the_source_is_mutated_produces_consistent_copies ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/concurrency.rs (target/debug/deps/concurrency-1515948d9de680fe)

running 5 tests
test a_node_handle_remains_valid_after_being_moved_by_another_thread ... ok
test handles_can_be_moved_between_threads ... ok
test structural_edits_stay_consistent_under_contention ... ok
test readers_and_writers_do_not_deadlock ... ok
test two_threads_adding_opposite_children_never_create_a_cycle ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s

     Running tests/element.rs (target/debug/deps/element-12a86330486bce83)

running 9 tests
test an_empty_default_declaration_removes_the_default_namespace ... ok
test declarations_are_inherited_and_shadowed ... ok
test names_accessors_agree ... ok
test attribute_crud_is_keyed_by_expanded_name ... ok
test invalid_attribute_values_are_rejected ... ok
test removing_a_declaration_is_silent_even_though_the_subtree_relies_on_it ... ok
test namespace_declarations_can_be_added_changed_and_removed ... ok
test resolution_uses_the_in_scope_declarations ... ok
test namespaced_documents_round_trip_through_the_parser ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/errors.rs (target/debug/deps/errors-1c44746252c15543)

running 13 tests
test namespace_construction_errors ... ok
test insert_child_errors ... ok
test append_child_errors ... ok
test element_errors ... ok
test node_construction_errors ... ok
test parse_file_reports_io_errors ... ok
test qualified_name_construction_errors ... ok
test qualified_name_resolution_errors ... ok
test replace_with_errors ... ok
test relative_insertion_errors ... ok
test set_root_errors ... ok
test parse_errors ... ok
test unclosed_elements_and_xml_target_pis_are_rejected ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/io.rs (target/debug/deps/io-56c3c281a202850c)

running 35 tests
test a_document_without_a_root_serializes_to_nothing_useful_but_does_not_fail ... ok
test a_declaration_must_be_the_very_first_thing ... ok
test an_optional_utf8_byte_order_mark_is_accepted ... ok
test adjacent_text_nodes_are_merged_on_output ... ok
test a_literal_carriage_return_in_text_round_trips ... ok
test cdata_and_processing_instructions_must_be_well_formed ... ok
test an_empty_default_declaration_removes_the_default_namespace ... ok
test character_references_are_expanded ... ok
test declaration_style_is_honoured ... ok
test comments_must_be_well_formed ... ok
test empty_text_nodes_have_no_representation_and_are_omitted ... ok
test attribute_value_normalisation_in_both_directions ... ok
test empty_elements_and_write_options ... ok
test illegal_character_references_are_rejected ... ok
test comments_cdata_and_processing_instructions_are_written_verbatim ... ok
test invalid_utf8_is_a_typed_error ... ok
test less_than_is_rejected_in_attribute_values ... ok
test line_ends_are_normalised_in_text ... ok
test namespace_declarations_are_never_invented_or_removed ... ok
test namespaces_are_resolved_and_reported ... ok
test predefined_entities_are_expanded ... ok
test prefixes_are_preserved ... ok
test tags_must_nest_and_close ... ok
test text_containing_a_cdata_close_round_trips ... ok
test text_is_escaped_so_that_it_round_trips ... ok
test processing_instruction_content_is_kept_verbatim ... ok
test rules_referenced_by_the_parser_and_serializer_exist ... ok
test the_declaration_is_interpreted_and_preserved ... ok
test undeclared_entity_references_are_typed_errors_not_panics ... ok
test there_must_be_exactly_one_root_element ... ok
test unsupported_declarations_are_rejected ... ok
test top_level_content_policy ... ok
test xml_lang_and_xml_space_round_trip_with_the_implicit_xml_prefix ... ok
test parser_never_panics_on_odd_input ... ok
test a_deeply_nested_document_round_trips ... ok

test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s

     Running tests/properties.rs (target/debug/deps/properties-091b04b342776f42)

running 7 tests
test arbitrary_strings_never_panic ... ok
test arbitrary_bytes_never_panic ... ok
test writing_is_idempotent ... ok
test the_declaration_survives_a_round_trip ... ok
test generated_documents_validate_clean ... ok
test round_trip_preserves_the_document ... ok
test mutated_well_formed_documents_never_panic ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.49s

     Running tests/structure.rs (target/debug/deps/structure-dfb1e68450e4349e)

running 28 tests
test a_foreign_element_cannot_become_the_root ... ok
test an_attached_element_cannot_become_the_root ... ok
test attachment_state_is_only_about_the_root_of_the_document ... ok
test attaching_a_detached_subtree_restores_it ... ok
test a_detached_subtree_stays_editable_clonable_and_reattachable ... ok
test cycles_are_rejected ... ok
test attaching_and_detaching_keeps_the_tree_consistent ... ok
test detaching_is_idempotent_and_returns_the_previous_parent ... ok
test foreign_documents_are_rejected ... ok
test descendants_are_returned_in_document_pre_order ... ok
test insertion_positions_are_stable ... ok
test non_elements_cannot_be_parents ... ok
test mixed_content_node_kinds_are_preserved ... ok
test out_of_range_insertions_are_rejected ... ok
test panicking_append_child_on_a_foreign_document_leaves_everything_unchanged ... ok
test panicking_insert_child_leaves_the_document_unchanged ... ok
test panicking_append_child_leaves_the_document_unchanged ... ok
test relative_insertion_requires_a_sibling_of_the_same_parent ... ok
test remove_returns_the_detached_node_itself ... ok
test panicking_set_root_leaves_the_document_unchanged ... ok
test panicking_set_attribute_leaves_the_document_unchanged ... ok
test replacing_a_node_by_itself_is_a_documented_no_op ... ok
test panicking_replace_with_leaves_the_document_unchanged ... ok
test replacing_swaps_the_node_in_place ... ok
test sibling_and_index_queries_agree_with_the_child_list ... ok
test setting_the_root_returns_the_previous_one ... ok
test the_root_cannot_be_attached_as_a_child ... ok
test a_deeply_nested_document_does_not_overflow_the_stack ... ok

test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

     Running tests/validation.rs (target/debug/deps/validation-3d645127ae7e363e)

running 21 tests
test a_consistent_document_has_no_structural_problems ... ok
test a_prefix_bound_to_a_different_uri_is_reported ... ok
test an_empty_document_has_no_root ... ok
test an_element_in_a_namespace_needs_it_declared ... ok
test a_detached_copy_does_not_conflict_with_its_original ... ok
test all_problems_are_reported_in_one_call ... ok
test attributes_are_not_affected_by_the_default_namespace ... ok
test detached_subtrees_are_validated_against_their_own_scope ... ok
test the_error_type_is_a_std_error ... ok
test every_error_kind_names_an_existing_rule_file ... ok
test default_namespace_scope_must_match_the_name ... ok
test moving_an_element_can_break_its_namespace_and_validation_says_so ... ok
test editing_is_silent_and_validation_is_what_reports ... ok
test the_xml_prefix_is_always_available ... ok
test validation_is_deterministic ... ok
test validation_does_not_change_the_document ... ok
test xml_id_values_must_be_names_and_unique ... ok
test xml_lang_and_xml_space_values_are_checked ... ok
test the_xml_id_policy_is_pinned_in_both_directions ... ok
test every_parsed_fixture_is_valid ... ok
test consistent_documents_produce_no_structural_issues ... ok

test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

     Running unittests src/lib.rs (target/debug/deps/biodivine_lib_xml_dom_sys-d92e208f034d9451)

running 6 tests
test tests::handles_are_send_and_sync ... ok
test tests::handles_can_be_moved_into_a_plain_rust_thread ... ok
test tests::a_document_can_be_shared_by_several_threads ... ok
test tests::a_validation_failure_carries_every_problem ... ok
test tests::a_document_round_trips_through_the_bindings ... ok
test tests::a_parsed_document_is_the_same_tree_as_the_serialized_one ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests biodivine_lib_xml_dom

running 27 tests
test src/lib.rs - (line 107) ... ok
test src/lib.rs - (line 63) ... ok
test src/namespace.rs - namespace::Namespace::prefix (line 136) ... ok
test src/document.rs - document::Document::validate (line 390) ... ok
test src/lib.rs - (line 87) ... ok
test src/namespace.rs - namespace::Namespace::new (line 58) ... ok
test src/namespace.rs - namespace::Namespace::prefixed (line 107) ... ok
test src/namespace.rs - namespace::Namespace::is_equal_ns (line 169) ... ok
test src/namespace.rs - namespace::Namespace::without_prefix (line 88) ... ok
test src/namespace.rs - namespace::Namespace::uri (line 122) ... ok
test src/node.rs - node::Node (line 63) ... ok
test src/namespace.rs - namespace::Namespace::prefix_str (line 153) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName (line 43) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName::resolve_attribute_with_namespace_map (line 347) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName::resolve_element (line 163) ... ok
test src/lib.rs - (line 134) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName::resolve_element_with_namespace_map (line 318) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName::with_namespace (line 105) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName::resolve_attribute (line 195) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName::without_namespace (line 87) ... ok
test src/xml_spec.rs - xml_spec::CData (line 282) ... ok
test src/xml_spec.rs - xml_spec::Comment (line 313) ... ok
test src/xml_spec.rs - xml_spec::NCName::as_str (line 220) ... ok
test src/xml_spec.rs - xml_spec::PiTarget (line 348) ... ok
test src/xml_spec.rs - xml_spec::NCName (line 193) ... ok
test src/xml_spec.rs - xml_spec::PiData (line 380) ... ok
test src/xml_spec.rs - xml_spec::Text (line 239) ... ok

test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

all doctests ran in 0.13s; merged doctests compilation took 0.13s
   Doc-tests biodivine_lib_xml_dom_sys

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

-> exit 0

$ cargo test --workspace --release
    Finished `release` profile [optimized] target(s) in 0.01s
     Running unittests src/lib.rs (target/release/deps/biodivine_lib_xml_dom-8f66900261159030)

running 83 tests
test arena::tests::attach_and_detach_maintain_both_directions ... ok
test arena::tests::attached_root_cannot_be_reset_as_root ... ok
test arena::tests::cyclic_attachments_are_rejected ... ok
test arena::tests::out_of_range_index_is_an_error_and_changes_nothing ... ok
test arena::tests::relative_insertion ... ok
test arena::tests::replace_by_self_is_a_no_op ... ok
test arena::tests::exceeding_the_arena_capacity_panics_instead_of_wrapping - should panic ... ok
test arena::tests::detaching_a_detached_node_is_a_no_op ... ok
test arena::tests::replace_keeps_links_consistent ... ok
test arena::tests::namespaces_in_scope_shadow_outer_declarations ... ok
test arena::tests::replace_rejects_an_ancestor_of_the_replaced_node ... ok
test arena::tests::root_cannot_be_attached ... ok
test arena::tests::snapshot_round_trip_preserves_the_subtree ... ok
test arena::tests::the_capacity_guard_allows_everything_below_the_limit ... ok
test arena::tests::the_index_is_interpreted_after_detaching ... ok
test document::tests::documents_are_equal_by_identity ... ok
test document::tests::empty_documents_have_no_root ... ok
test document::tests::handles_are_send_and_sync ... ok
test document::tests::nesting_access_to_two_different_documents_is_allowed ... ok
test interner::tests::interning_deduplicates_by_value ... ok
test interner::tests::interning_never_rewrites_a_prefix ... ok
test namespace::tests::test_namespace_equality ... ok
test namespace::tests::test_namespace_is_equal_ns ... ok
test namespace::tests::test_namespace_support ... ok
test namespace::tests::test_unicode_prefixes ... ok
test node::tests::interning_does_not_change_equality ... ok
test node::tests::re_interns_names_in_the_target_document ... ok
test qualified_name::tests::test_creation_and_error ... ok
test qualified_name::tests::test_equality_and_ordering ... ok
test qualified_name::tests::test_hashing_semantic_equality ... ok
test qualified_name::tests::test_qualified_name_string_no_prefix_ns ... ok
test qualified_name::tests::test_resolve_attribute_ignores_default_ns ... ok
test qualified_name::tests::test_ord_consistent_with_partial_eq ... ok
test qualified_name::tests::test_resolve_attribute_with_map_ignores_default_ns ... ok
test qualified_name::tests::test_resolve_attribute_xml_prefix_auto ... ok
test qualified_name::tests::test_resolve_attribute_with_map_prefixed ... ok
test qualified_name::tests::test_resolve_no_prefix ... ok
test node::tests::repeated_names_share_one_allocation ... ok
test qualified_name::tests::test_resolve_undefined_prefix ... ok
test qualified_name::tests::test_resolve_with_map_prefixed ... ok
test qualified_name::tests::test_resolve_with_map_undefined_prefix ... ok
test qualified_name::tests::test_resolve_with_map_xml_prefix_auto ... ok
test qualified_name::tests::test_resolve_with_prefix ... ok
test qualified_name::tests::test_resolve_xml_prefix_auto ... ok
test qualified_name::tests::test_resolve_xmlns_prefix_rejected ... ok
test xml_spec::declaration::tests::accessors ... ok
test tests::test_document_reference ... ok
test tests::test_create_document ... ok
test xml_spec::declaration::tests::declaration_rendering ... ok
test xml_spec::declaration::tests::pseudo_attributes ... ok
test xml_spec::g3_tests::content_that_cannot_be_escaped_is_normalised_at_construction ... ok
test xml_spec::declaration::tests::encoding_names_are_case_insensitive ... ok
test xml_spec::g3_tests::language_tags ... ok
test tests::test_create_element ... ok
test xml_spec::g3_tests::line_end_normalisation ... ok
test xml_spec::g3_tests::xml_space_values ... ok
test xml_spec::rules::tests::the_rules_directory_is_where_we_think_it_is ... ok
test xml_spec::tests::test_cdata_wrapper ... ok
test xml_spec::tests::test_comment_wrapper ... ok
test xml_spec::tests::test_namespace_validation ... ok
test xml_spec::tests::test_ncname_helper ... ok
test xml_spec::tests::test_pi_data_wrapper ... ok
test qualified_name::tests::test_resolve_attribute_with_prefix ... ok
test tests::test_dropping_a_document_frees_its_nodes ... ok
test qualified_name::tests::test_resolve_with_parent_ns ... ok
test xml_spec::tests::test_prefix_ncname_validation ... ok
test xml_spec::tests::test_pi_target_wrapper ... ok
test xml_spec::tests::test_reserved_xml_prefix ... ok
test xml_spec::tests::test_reserved_xmlns_prefix ... ok
test xml_spec::tests::test_split_qname_invalid ... ok
test xml_spec::tests::test_split_qname_valid ... ok
test xml_spec::tests::test_text_wrapper ... ok
test xml_spec::tests::test_uri_comparison_case_sensitive ... ok
test xml_spec::tests::test_validate_resolved_prefix ... ok
test xml_spec::validation::tests::attributes_ignore_the_default_namespace ... ok
test xml_spec::validation::tests::default_namespace_rules ... ok
test xml_spec::validation::tests::inner_declarations_shadow_outer_ones ... ok
test xml_spec::validation::tests::prefixed_names_must_be_declared ... ok
test xml_spec::validation::tests::the_xml_prefix_needs_no_declaration ... ok
test xml_spec::validation::tests::the_xmlns_prefix_can_never_become_a_name_prefix ... ok
test tests::test_add_children ... ok
test tests::test_qualified_name_resolution ... ok
test tests::test_namespace_declaration ... ok

test result: ok. 83 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/cloning.rs (target/release/deps/cloning-60aca54ac6a78a3d)

running 15 tests
test a_deep_clone_is_cheap_because_names_are_shared ... ok
test cloning_a_handle_refers_to_the_same_node ... ok
test a_copied_subtree_can_be_attached_in_the_target_document ... ok
test a_deep_copy_can_be_edited_independently_of_its_source ... ok
test deep_clone_copies_the_whole_subtree ... ok
test deep_clone_into_another_document_rebuilds_the_tree_there ... ok
test deep_clone_into_the_same_document_behaves_like_deep_clone ... ok
test display_of_non_element_nodes ... ok
test display_renders_the_subtree_as_xml ... ok
test node_content_exposes_every_kind ... ok
test non_element_nodes_can_be_copied_between_documents ... ok
test shallow_clone_copies_the_payload_but_not_the_children ... ok
test shallow_clone_into_another_document_keeps_only_the_node ... ok
test shallow_clone_of_a_text_node_copies_the_content ... ok
test copying_while_the_source_is_mutated_produces_consistent_copies ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/concurrency.rs (target/release/deps/concurrency-b520ace3713d4b72)

running 5 tests
test a_node_handle_remains_valid_after_being_moved_by_another_thread ... ok
test handles_can_be_moved_between_threads ... ok
test structural_edits_stay_consistent_under_contention ... ok
test readers_and_writers_do_not_deadlock ... ok
test two_threads_adding_opposite_children_never_create_a_cycle ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

     Running tests/element.rs (target/release/deps/element-e12c79aa144ddfbc)

running 9 tests
test an_empty_default_declaration_removes_the_default_namespace ... ok
test attribute_crud_is_keyed_by_expanded_name ... ok
test invalid_attribute_values_are_rejected ... ok
test declarations_are_inherited_and_shadowed ... ok
test names_accessors_agree ... ok
test namespace_declarations_can_be_added_changed_and_removed ... ok
test removing_a_declaration_is_silent_even_though_the_subtree_relies_on_it ... ok
test resolution_uses_the_in_scope_declarations ... ok
test namespaced_documents_round_trip_through_the_parser ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/errors.rs (target/release/deps/errors-ac275d8870dcb29c)

running 13 tests
test append_child_errors ... ok
test element_errors ... ok
test insert_child_errors ... ok
test namespace_construction_errors ... ok
test node_construction_errors ... ok
test parse_file_reports_io_errors ... ok
test parse_errors ... ok
test qualified_name_construction_errors ... ok
test qualified_name_resolution_errors ... ok
test relative_insertion_errors ... ok
test replace_with_errors ... ok
test set_root_errors ... ok
test unclosed_elements_and_xml_target_pis_are_rejected ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/io.rs (target/release/deps/io-39af72257fdb05d1)

running 35 tests
test a_document_without_a_root_serializes_to_nothing_useful_but_does_not_fail ... ok
test a_declaration_must_be_the_very_first_thing ... ok
test a_literal_carriage_return_in_text_round_trips ... ok
test adjacent_text_nodes_are_merged_on_output ... ok
test an_optional_utf8_byte_order_mark_is_accepted ... ok
test an_empty_default_declaration_removes_the_default_namespace ... ok
test cdata_and_processing_instructions_must_be_well_formed ... ok
test character_references_are_expanded ... ok
test attribute_value_normalisation_in_both_directions ... ok
test comments_cdata_and_processing_instructions_are_written_verbatim ... ok
test comments_must_be_well_formed ... ok
test declaration_style_is_honoured ... ok
test empty_elements_and_write_options ... ok
test empty_text_nodes_have_no_representation_and_are_omitted ... ok
test illegal_character_references_are_rejected ... ok
test invalid_utf8_is_a_typed_error ... ok
test less_than_is_rejected_in_attribute_values ... ok
test line_ends_are_normalised_in_text ... ok
test namespace_declarations_are_never_invented_or_removed ... ok
test namespaces_are_resolved_and_reported ... ok
test predefined_entities_are_expanded ... ok
test parser_never_panics_on_odd_input ... ok
test prefixes_are_preserved ... ok
test processing_instruction_content_is_kept_verbatim ... ok
test text_containing_a_cdata_close_round_trips ... ok
test tags_must_nest_and_close ... ok
test text_is_escaped_so_that_it_round_trips ... ok
test rules_referenced_by_the_parser_and_serializer_exist ... ok
test there_must_be_exactly_one_root_element ... ok
test the_declaration_is_interpreted_and_preserved ... ok
test top_level_content_policy ... ok
test undeclared_entity_references_are_typed_errors_not_panics ... ok
test unsupported_declarations_are_rejected ... ok
test xml_lang_and_xml_space_round_trip_with_the_implicit_xml_prefix ... ok
test a_deeply_nested_document_round_trips ... ok

test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/properties.rs (target/release/deps/properties-62051133e5db9b0b)

running 7 tests
test arbitrary_strings_never_panic ... ok
test arbitrary_bytes_never_panic ... ok
test generated_documents_validate_clean ... ok
test the_declaration_survives_a_round_trip ... ok
test writing_is_idempotent ... ok
test round_trip_preserves_the_document ... ok
test mutated_well_formed_documents_never_panic ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/structure.rs (target/release/deps/structure-9b2556c717bdc7e8)

running 28 tests
test a_detached_subtree_stays_editable_clonable_and_reattachable ... ok
test a_foreign_element_cannot_become_the_root ... ok
test an_attached_element_cannot_become_the_root ... ok
test attaching_a_detached_subtree_restores_it ... ok
test attaching_and_detaching_keeps_the_tree_consistent ... ok
test attachment_state_is_only_about_the_root_of_the_document ... ok
test cycles_are_rejected ... ok
test descendants_are_returned_in_document_pre_order ... ok
test detaching_is_idempotent_and_returns_the_previous_parent ... ok
test foreign_documents_are_rejected ... ok
test insertion_positions_are_stable ... ok
test mixed_content_node_kinds_are_preserved ... ok
test out_of_range_insertions_are_rejected ... ok
test non_elements_cannot_be_parents ... ok
test panicking_append_child_on_a_foreign_document_leaves_everything_unchanged ... ok
test panicking_append_child_leaves_the_document_unchanged ... ok
test panicking_insert_child_leaves_the_document_unchanged ... ok
test panicking_replace_with_leaves_the_document_unchanged ... ok
test panicking_set_attribute_leaves_the_document_unchanged ... ok
test relative_insertion_requires_a_sibling_of_the_same_parent ... ok
test panicking_set_root_leaves_the_document_unchanged ... ok
test remove_returns_the_detached_node_itself ... ok
test replacing_a_node_by_itself_is_a_documented_no_op ... ok
test replacing_swaps_the_node_in_place ... ok
test setting_the_root_returns_the_previous_one ... ok
test sibling_and_index_queries_agree_with_the_child_list ... ok
test the_root_cannot_be_attached_as_a_child ... ok
test a_deeply_nested_document_does_not_overflow_the_stack ... ok

test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/validation.rs (target/release/deps/validation-627e221dbf12f4ae)

running 21 tests
test a_consistent_document_has_no_structural_problems ... ok
test a_detached_copy_does_not_conflict_with_its_original ... ok
test a_prefix_bound_to_a_different_uri_is_reported ... ok
test all_problems_are_reported_in_one_call ... ok
test an_element_in_a_namespace_needs_it_declared ... ok
test an_empty_document_has_no_root ... ok
test attributes_are_not_affected_by_the_default_namespace ... ok
test detached_subtrees_are_validated_against_their_own_scope ... ok
test default_namespace_scope_must_match_the_name ... ok
test editing_is_silent_and_validation_is_what_reports ... ok
test every_error_kind_names_an_existing_rule_file ... ok
test moving_an_element_can_break_its_namespace_and_validation_says_so ... ok
test the_error_type_is_a_std_error ... ok
test the_xml_id_policy_is_pinned_in_both_directions ... ok
test every_parsed_fixture_is_valid ... ok
test the_xml_prefix_is_always_available ... ok
test validation_does_not_change_the_document ... ok
test validation_is_deterministic ... ok
test xml_id_values_must_be_names_and_unique ... ok
test xml_lang_and_xml_space_values_are_checked ... ok
test consistent_documents_produce_no_structural_issues ... ok

test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running unittests src/lib.rs (target/release/deps/biodivine_lib_xml_dom_sys-f1723a3c1e3f35e7)

running 6 tests
test tests::handles_are_send_and_sync ... ok
test tests::handles_can_be_moved_into_a_plain_rust_thread ... ok
test tests::a_document_can_be_shared_by_several_threads ... ok
test tests::a_validation_failure_carries_every_problem ... ok
test tests::a_document_round_trips_through_the_bindings ... ok
test tests::a_parsed_document_is_the_same_tree_as_the_serialized_one ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests biodivine_lib_xml_dom

running 27 tests
test src/document.rs - document::Document::validate (line 390) ... ok
test src/lib.rs - (line 107) ... ok
test src/lib.rs - (line 87) ... ok
test src/lib.rs - (line 63) ... ok
test src/lib.rs - (line 134) ... ok
test src/namespace.rs - namespace::Namespace::prefix (line 136) ... ok
test src/namespace.rs - namespace::Namespace::is_equal_ns (line 169) ... ok
test src/namespace.rs - namespace::Namespace::prefix_str (line 153) ... ok
test src/namespace.rs - namespace::Namespace::uri (line 122) ... ok
test src/namespace.rs - namespace::Namespace::without_prefix (line 88) ... ok
test src/namespace.rs - namespace::Namespace::prefixed (line 107) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName (line 43) ... ok
test src/namespace.rs - namespace::Namespace::new (line 58) ... ok
test src/node.rs - node::Node (line 63) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName::resolve_attribute (line 195) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName::resolve_element (line 163) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName::resolve_attribute_with_namespace_map (line 347) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName::resolve_element_with_namespace_map (line 318) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName::with_namespace (line 105) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName::without_namespace (line 87) ... ok
test src/xml_spec.rs - xml_spec::Comment (line 313) ... ok
test src/xml_spec.rs - xml_spec::CData (line 282) ... ok
test src/xml_spec.rs - xml_spec::NCName (line 193) ... ok
test src/xml_spec.rs - xml_spec::NCName::as_str (line 220) ... ok
test src/xml_spec.rs - xml_spec::PiTarget (line 348) ... ok
test src/xml_spec.rs - xml_spec::Text (line 239) ... ok
test src/xml_spec.rs - xml_spec::PiData (line 380) ... ok

test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

all doctests ran in 0.15s; merged doctests compilation took 0.15s
   Doc-tests biodivine_lib_xml_dom_sys

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

-> exit 0

$ cargo +1.88.0 test --workspace
   Compiling biodivine-lib-xml-dom v0.2.0 (/sandbox/biodivine-lib-xml-dom)
   Compiling biodivine-lib-xml-dom-py-sys v0.2.0 (/sandbox/biodivine-lib-xml-dom/biodivine-lib-xml-dom-py-sys)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.77s
     Running unittests src/lib.rs (target/debug/deps/biodivine_lib_xml_dom-24225d7e2074f8a2)

running 84 tests
test arena::tests::cyclic_attachments_are_rejected ... ok
test arena::tests::attach_and_detach_maintain_both_directions ... ok
test arena::tests::detaching_a_detached_node_is_a_no_op ... ok
test arena::tests::attached_root_cannot_be_reset_as_root ... ok
test arena::tests::out_of_range_index_is_an_error_and_changes_nothing ... ok
test arena::tests::namespaces_in_scope_shadow_outer_declarations ... ok
test arena::tests::exceeding_the_arena_capacity_panics_instead_of_wrapping - should panic ... ok
test arena::tests::replace_by_self_is_a_no_op ... ok
test arena::tests::relative_insertion ... ok
test arena::tests::replace_keeps_links_consistent ... ok
test arena::tests::replace_rejects_an_ancestor_of_the_replaced_node ... ok
test arena::tests::root_cannot_be_attached ... ok
test arena::tests::the_capacity_guard_allows_everything_below_the_limit ... ok
test arena::tests::snapshot_round_trip_preserves_the_subtree ... ok
test arena::tests::the_index_is_interpreted_after_detaching ... ok
test document::tests::documents_are_equal_by_identity ... ok
test document::tests::empty_documents_have_no_root ... ok
test document::tests::handles_are_send_and_sync ... ok
test document::tests::nesting_access_to_two_different_documents_is_allowed ... ok
test interner::tests::interning_deduplicates_by_value ... ok
test interner::tests::interning_never_rewrites_a_prefix ... ok
test document::tests::re_entrant_access_to_the_same_document_panics_in_debug_builds - should panic ... ok
test namespace::tests::test_namespace_equality ... ok
test namespace::tests::test_namespace_is_equal_ns ... ok
test namespace::tests::test_unicode_prefixes ... ok
test namespace::tests::test_namespace_support ... ok
test node::tests::interning_does_not_change_equality ... ok
test qualified_name::tests::test_creation_and_error ... ok
test node::tests::re_interns_names_in_the_target_document ... ok
test qualified_name::tests::test_hashing_semantic_equality ... ok
test qualified_name::tests::test_equality_and_ordering ... ok
test qualified_name::tests::test_ord_consistent_with_partial_eq ... ok
test qualified_name::tests::test_qualified_name_string_no_prefix_ns ... ok
test qualified_name::tests::test_resolve_attribute_ignores_default_ns ... ok
test qualified_name::tests::test_resolve_attribute_with_map_ignores_default_ns ... ok
test qualified_name::tests::test_resolve_attribute_with_map_prefixed ... ok
test qualified_name::tests::test_resolve_attribute_xml_prefix_auto ... ok
test qualified_name::tests::test_resolve_with_map_undefined_prefix ... ok
test qualified_name::tests::test_resolve_with_map_xml_prefix_auto ... ok
test qualified_name::tests::test_resolve_with_prefix ... ok
test qualified_name::tests::test_resolve_attribute_with_prefix ... ok
test tests::test_document_reference ... ok
test tests::test_create_document ... ok
test tests::test_add_children ... ok
test xml_spec::declaration::tests::accessors ... ok
test xml_spec::declaration::tests::pseudo_attributes ... ok
test qualified_name::tests::test_resolve_undefined_prefix ... ok
test xml_spec::g3_tests::line_end_normalisation ... ok
test qualified_name::tests::test_resolve_with_map_prefixed ... ok
test qualified_name::tests::test_resolve_xmlns_prefix_rejected ... ok
test xml_spec::tests::test_cdata_wrapper ... ok
test qualified_name::tests::test_resolve_with_parent_ns ... ok
test xml_spec::tests::test_namespace_validation ... ok
test xml_spec::tests::test_comment_wrapper ... ok
test qualified_name::tests::test_resolve_no_prefix ... ok
test xml_spec::tests::test_pi_data_wrapper ... ok
test xml_spec::tests::test_ncname_helper ... ok
test qualified_name::tests::test_resolve_xml_prefix_auto ... ok
test tests::test_qualified_name_resolution ... ok
test tests::test_dropping_a_document_frees_its_nodes ... ok
test tests::test_namespace_declaration ... ok
test xml_spec::g3_tests::language_tags ... ok
test xml_spec::declaration::tests::encoding_names_are_case_insensitive ... ok
test xml_spec::g3_tests::content_that_cannot_be_escaped_is_normalised_at_construction ... ok
test xml_spec::tests::test_pi_target_wrapper ... ok
test xml_spec::g3_tests::xml_space_values ... ok
test xml_spec::tests::test_prefix_ncname_validation ... ok
test xml_spec::rules::tests::the_rules_directory_is_where_we_think_it_is ... ok
test xml_spec::tests::test_reserved_xml_prefix ... ok
test xml_spec::declaration::tests::declaration_rendering ... ok
test xml_spec::tests::test_reserved_xmlns_prefix ... ok
test xml_spec::tests::test_split_qname_invalid ... ok
test xml_spec::tests::test_split_qname_valid ... ok
test xml_spec::tests::test_text_wrapper ... ok
test xml_spec::tests::test_uri_comparison_case_sensitive ... ok
test tests::test_create_element ... ok
test xml_spec::validation::tests::default_namespace_rules ... ok
test xml_spec::validation::tests::inner_declarations_shadow_outer_ones ... ok
test xml_spec::tests::test_validate_resolved_prefix ... ok
test xml_spec::validation::tests::prefixed_names_must_be_declared ... ok
test xml_spec::validation::tests::the_xmlns_prefix_can_never_become_a_name_prefix ... ok
test xml_spec::validation::tests::the_xml_prefix_needs_no_declaration ... ok
test xml_spec::validation::tests::attributes_ignore_the_default_namespace ... ok
test node::tests::repeated_names_share_one_allocation ... ok

test result: ok. 84 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/cloning.rs (target/debug/deps/cloning-e0a42b85a0e0c3b5)

running 15 tests
test cloning_a_handle_refers_to_the_same_node ... ok
test a_deep_clone_is_cheap_because_names_are_shared ... ok
test a_deep_copy_can_be_edited_independently_of_its_source ... ok
test a_copied_subtree_can_be_attached_in_the_target_document ... ok
test display_of_non_element_nodes ... ok
test deep_clone_into_the_same_document_behaves_like_deep_clone ... ok
test deep_clone_copies_the_whole_subtree ... ok
test deep_clone_into_another_document_rebuilds_the_tree_there ... ok
test node_content_exposes_every_kind ... ok
test non_element_nodes_can_be_copied_between_documents ... ok
test display_renders_the_subtree_as_xml ... ok
test shallow_clone_copies_the_payload_but_not_the_children ... ok
test shallow_clone_of_a_text_node_copies_the_content ... ok
test shallow_clone_into_another_document_keeps_only_the_node ... ok
test copying_while_the_source_is_mutated_produces_consistent_copies ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/concurrency.rs (target/debug/deps/concurrency-ddc56dd505f4fdd8)

running 5 tests
test a_node_handle_remains_valid_after_being_moved_by_another_thread ... ok
test handles_can_be_moved_between_threads ... ok
test structural_edits_stay_consistent_under_contention ... ok
test readers_and_writers_do_not_deadlock ... ok
test two_threads_adding_opposite_children_never_create_a_cycle ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s

     Running tests/element.rs (target/debug/deps/element-d7fced86236f1bdd)

running 9 tests
test an_empty_default_declaration_removes_the_default_namespace ... ok
test declarations_are_inherited_and_shadowed ... ok
test attribute_crud_is_keyed_by_expanded_name ... ok
test names_accessors_agree ... ok
test invalid_attribute_values_are_rejected ... ok
test resolution_uses_the_in_scope_declarations ... ok
test namespace_declarations_can_be_added_changed_and_removed ... ok
test removing_a_declaration_is_silent_even_though_the_subtree_relies_on_it ... ok
test namespaced_documents_round_trip_through_the_parser ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/errors.rs (target/debug/deps/errors-c4d43ca35311bdf4)

running 13 tests
test insert_child_errors ... ok
test append_child_errors ... ok
test namespace_construction_errors ... ok
test element_errors ... ok
test node_construction_errors ... ok
test qualified_name_construction_errors ... ok
test parse_file_reports_io_errors ... ok
test parse_errors ... ok
test relative_insertion_errors ... ok
test qualified_name_resolution_errors ... ok
test replace_with_errors ... ok
test set_root_errors ... ok
test unclosed_elements_and_xml_target_pis_are_rejected ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/io.rs (target/debug/deps/io-b0668a97b41d6e50)

running 35 tests
test a_document_without_a_root_serializes_to_nothing_useful_but_does_not_fail ... ok
test a_declaration_must_be_the_very_first_thing ... ok
test a_literal_carriage_return_in_text_round_trips ... ok
test adjacent_text_nodes_are_merged_on_output ... ok
test an_optional_utf8_byte_order_mark_is_accepted ... ok
test cdata_and_processing_instructions_must_be_well_formed ... ok
test an_empty_default_declaration_removes_the_default_namespace ... ok
test character_references_are_expanded ... ok
test comments_must_be_well_formed ... ok
test comments_cdata_and_processing_instructions_are_written_verbatim ... ok
test declaration_style_is_honoured ... ok
test empty_text_nodes_have_no_representation_and_are_omitted ... ok
test attribute_value_normalisation_in_both_directions ... ok
test empty_elements_and_write_options ... ok
test illegal_character_references_are_rejected ... ok
test invalid_utf8_is_a_typed_error ... ok
test less_than_is_rejected_in_attribute_values ... ok
test line_ends_are_normalised_in_text ... ok
test namespace_declarations_are_never_invented_or_removed ... ok
test predefined_entities_are_expanded ... ok
test namespaces_are_resolved_and_reported ... ok
test prefixes_are_preserved ... ok
test text_containing_a_cdata_close_round_trips ... ok
test tags_must_nest_and_close ... ok
test rules_referenced_by_the_parser_and_serializer_exist ... ok
test there_must_be_exactly_one_root_element ... ok
test text_is_escaped_so_that_it_round_trips ... ok
test the_declaration_is_interpreted_and_preserved ... ok
test undeclared_entity_references_are_typed_errors_not_panics ... ok
test parser_never_panics_on_odd_input ... ok
test unsupported_declarations_are_rejected ... ok
test processing_instruction_content_is_kept_verbatim ... ok
test top_level_content_policy ... ok
test xml_lang_and_xml_space_round_trip_with_the_implicit_xml_prefix ... ok
test a_deeply_nested_document_round_trips ... ok

test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s

     Running tests/properties.rs (target/debug/deps/properties-b3ded858489b1aac)

running 7 tests
test arbitrary_strings_never_panic ... ok
test arbitrary_bytes_never_panic ... ok
test the_declaration_survives_a_round_trip ... ok
test round_trip_preserves_the_document ... ok
test writing_is_idempotent ... ok
test generated_documents_validate_clean ... ok
test mutated_well_formed_documents_never_panic ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s

     Running tests/structure.rs (target/debug/deps/structure-5cbfe486d4dd8df8)

running 28 tests
test a_foreign_element_cannot_become_the_root ... ok
test a_detached_subtree_stays_editable_clonable_and_reattachable ... ok
test attachment_state_is_only_about_the_root_of_the_document ... ok
test attaching_and_detaching_keeps_the_tree_consistent ... ok
test an_attached_element_cannot_become_the_root ... ok
test attaching_a_detached_subtree_restores_it ... ok
test cycles_are_rejected ... ok
test descendants_are_returned_in_document_pre_order ... ok
test detaching_is_idempotent_and_returns_the_previous_parent ... ok
test foreign_documents_are_rejected ... ok
test insertion_positions_are_stable ... ok
test non_elements_cannot_be_parents ... ok
test mixed_content_node_kinds_are_preserved ... ok
test out_of_range_insertions_are_rejected ... ok
test panicking_append_child_on_a_foreign_document_leaves_everything_unchanged ... ok
test panicking_set_attribute_leaves_the_document_unchanged ... ok
test panicking_insert_child_leaves_the_document_unchanged ... ok
test relative_insertion_requires_a_sibling_of_the_same_parent ... ok
test panicking_append_child_leaves_the_document_unchanged ... ok
test remove_returns_the_detached_node_itself ... ok
test panicking_replace_with_leaves_the_document_unchanged ... ok
test panicking_set_root_leaves_the_document_unchanged ... ok
test replacing_a_node_by_itself_is_a_documented_no_op ... ok
test the_root_cannot_be_attached_as_a_child ... ok
test setting_the_root_returns_the_previous_one ... ok
test replacing_swaps_the_node_in_place ... ok
test sibling_and_index_queries_agree_with_the_child_list ... ok
test a_deeply_nested_document_does_not_overflow_the_stack ... ok

test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s

     Running tests/validation.rs (target/debug/deps/validation-f720c61abd33e25e)

running 21 tests
test a_consistent_document_has_no_structural_problems ... ok
test a_prefix_bound_to_a_different_uri_is_reported ... ok
test a_detached_copy_does_not_conflict_with_its_original ... ok
test an_empty_document_has_no_root ... ok
test an_element_in_a_namespace_needs_it_declared ... ok
test all_problems_are_reported_in_one_call ... ok
test attributes_are_not_affected_by_the_default_namespace ... ok
test default_namespace_scope_must_match_the_name ... ok
test detached_subtrees_are_validated_against_their_own_scope ... ok
test the_error_type_is_a_std_error ... ok
test every_error_kind_names_an_existing_rule_file ... ok
test editing_is_silent_and_validation_is_what_reports ... ok
test moving_an_element_can_break_its_namespace_and_validation_says_so ... ok
test the_xml_prefix_is_always_available ... ok
test xml_id_values_must_be_names_and_unique ... ok
test xml_lang_and_xml_space_values_are_checked ... ok
test validation_is_deterministic ... ok
test validation_does_not_change_the_document ... ok
test the_xml_id_policy_is_pinned_in_both_directions ... ok
test every_parsed_fixture_is_valid ... ok
test consistent_documents_produce_no_structural_issues ... ok

test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

     Running unittests src/lib.rs (target/debug/deps/biodivine_lib_xml_dom_sys-68b2ae5ab7a03b8a)

running 6 tests
test tests::handles_are_send_and_sync ... ok
test tests::handles_can_be_moved_into_a_plain_rust_thread ... ok
test tests::a_document_can_be_shared_by_several_threads ... ok
test tests::a_validation_failure_carries_every_problem ... ok
test tests::a_parsed_document_is_the_same_tree_as_the_serialized_one ... ok
test tests::a_document_round_trips_through_the_bindings ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests biodivine_lib_xml_dom

running 27 tests
test src/namespace.rs - namespace::Namespace::prefix (line 136) ... ok
test src/namespace.rs - namespace::Namespace::new (line 58) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName (line 43) ... ok
test src/lib.rs - (line 107) ... ok
test src/namespace.rs - namespace::Namespace::prefixed (line 107) ... ok
test src/document.rs - document::Document::validate (line 390) ... ok
test src/namespace.rs - namespace::Namespace::prefix_str (line 153) ... ok
test src/lib.rs - (line 134) ... ok
test src/lib.rs - (line 87) ... ok
test src/namespace.rs - namespace::Namespace::is_equal_ns (line 169) ... ok
test src/namespace.rs - namespace::Namespace::without_prefix (line 88) ... ok
test src/lib.rs - (line 63) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName::resolve_attribute (line 195) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName::resolve_attribute_with_namespace_map (line 347) ... ok
test src/node.rs - node::Node (line 63) ... ok
test src/namespace.rs - namespace::Namespace::uri (line 122) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName::resolve_element (line 163) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName::without_namespace (line 87) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName::with_namespace (line 105) ... ok
test src/xml_spec.rs - xml_spec::CData (line 282) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName::resolve_element_with_namespace_map (line 318) ... ok
test src/xml_spec.rs - xml_spec::PiData (line 380) ... ok
test src/xml_spec.rs - xml_spec::PiTarget (line 348) ... ok
test src/xml_spec.rs - xml_spec::NCName (line 193) ... ok
test src/xml_spec.rs - xml_spec::NCName::as_str (line 220) ... ok
test src/xml_spec.rs - xml_spec::Text (line 239) ... ok
test src/xml_spec.rs - xml_spec::Comment (line 313) ... ok

test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests biodivine_lib_xml_dom_sys

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

-> exit 0

$ cargo +1.95.0 test --workspace
   Compiling biodivine-lib-xml-dom v0.2.0 (/sandbox/biodivine-lib-xml-dom)
   Compiling biodivine-lib-xml-dom-py-sys v0.2.0 (/sandbox/biodivine-lib-xml-dom/biodivine-lib-xml-dom-py-sys)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.14s
     Running unittests src/lib.rs (target/debug/deps/biodivine_lib_xml_dom-cc4c35bad89bdcfe)

running 84 tests
test arena::tests::attach_and_detach_maintain_both_directions ... ok
test arena::tests::detaching_a_detached_node_is_a_no_op ... ok
test arena::tests::cyclic_attachments_are_rejected ... ok
test arena::tests::exceeding_the_arena_capacity_panics_instead_of_wrapping - should panic ... ok
test arena::tests::namespaces_in_scope_shadow_outer_declarations ... ok
test arena::tests::out_of_range_index_is_an_error_and_changes_nothing ... ok
test arena::tests::relative_insertion ... ok
test arena::tests::replace_by_self_is_a_no_op ... ok
test arena::tests::attached_root_cannot_be_reset_as_root ... ok
test arena::tests::replace_keeps_links_consistent ... ok
test arena::tests::root_cannot_be_attached ... ok
test arena::tests::replace_rejects_an_ancestor_of_the_replaced_node ... ok
test arena::tests::snapshot_round_trip_preserves_the_subtree ... ok
test arena::tests::the_capacity_guard_allows_everything_below_the_limit ... ok
test arena::tests::the_index_is_interpreted_after_detaching ... ok
test document::tests::documents_are_equal_by_identity ... ok
test document::tests::empty_documents_have_no_root ... ok
test document::tests::handles_are_send_and_sync ... ok
test document::tests::nesting_access_to_two_different_documents_is_allowed ... ok
test document::tests::re_entrant_access_to_the_same_document_panics_in_debug_builds - should panic ... ok
test interner::tests::interning_never_rewrites_a_prefix ... ok
test interner::tests::interning_deduplicates_by_value ... ok
test namespace::tests::test_namespace_is_equal_ns ... ok
test namespace::tests::test_namespace_equality ... ok
test namespace::tests::test_unicode_prefixes ... ok
test namespace::tests::test_namespace_support ... ok
test node::tests::interning_does_not_change_equality ... ok
test node::tests::re_interns_names_in_the_target_document ... ok
test qualified_name::tests::test_creation_and_error ... ok
test qualified_name::tests::test_equality_and_ordering ... ok
test qualified_name::tests::test_hashing_semantic_equality ... ok
test qualified_name::tests::test_ord_consistent_with_partial_eq ... ok
test qualified_name::tests::test_qualified_name_string_no_prefix_ns ... ok
test qualified_name::tests::test_resolve_attribute_with_map_ignores_default_ns ... ok
test qualified_name::tests::test_resolve_attribute_with_map_prefixed ... ok
test qualified_name::tests::test_resolve_attribute_ignores_default_ns ... ok
test qualified_name::tests::test_resolve_attribute_with_prefix ... ok
test qualified_name::tests::test_resolve_no_prefix ... ok
test qualified_name::tests::test_resolve_attribute_xml_prefix_auto ... ok
test qualified_name::tests::test_resolve_undefined_prefix ... ok
test qualified_name::tests::test_resolve_with_map_prefixed ... ok
test qualified_name::tests::test_resolve_with_parent_ns ... ok
test qualified_name::tests::test_resolve_xml_prefix_auto ... ok
test qualified_name::tests::test_resolve_with_map_undefined_prefix ... ok
test qualified_name::tests::test_resolve_xmlns_prefix_rejected ... ok
test tests::test_create_document ... ok
test tests::test_add_children ... ok
test qualified_name::tests::test_resolve_with_map_xml_prefix_auto ... ok
test qualified_name::tests::test_resolve_with_prefix ... ok
test tests::test_qualified_name_resolution ... ok
test tests::test_dropping_a_document_frees_its_nodes ... ok
test xml_spec::declaration::tests::declaration_rendering ... ok
test xml_spec::tests::test_ncname_helper ... ok
test xml_spec::tests::test_pi_target_wrapper ... ok
test xml_spec::g3_tests::language_tags ... ok
test xml_spec::declaration::tests::encoding_names_are_case_insensitive ... ok
test tests::test_create_element ... ok
test tests::test_namespace_declaration ... ok
test tests::test_document_reference ... ok
test xml_spec::g3_tests::xml_space_values ... ok
test xml_spec::declaration::tests::pseudo_attributes ... ok
test xml_spec::declaration::tests::accessors ... ok
test xml_spec::tests::test_cdata_wrapper ... ok
test xml_spec::tests::test_namespace_validation ... ok
test xml_spec::tests::test_pi_data_wrapper ... ok
test xml_spec::tests::test_reserved_xml_prefix ... ok
test xml_spec::tests::test_prefix_ncname_validation ... ok
test xml_spec::tests::test_text_wrapper ... ok
test xml_spec::g3_tests::line_end_normalisation ... ok
test xml_spec::tests::test_uri_comparison_case_sensitive ... ok
test xml_spec::tests::test_validate_resolved_prefix ... ok
test xml_spec::validation::tests::attributes_ignore_the_default_namespace ... ok
test xml_spec::validation::tests::default_namespace_rules ... ok
test xml_spec::validation::tests::inner_declarations_shadow_outer_ones ... ok
test xml_spec::validation::tests::the_xml_prefix_needs_no_declaration ... ok
test xml_spec::validation::tests::prefixed_names_must_be_declared ... ok
test xml_spec::rules::tests::the_rules_directory_is_where_we_think_it_is ... ok
test xml_spec::validation::tests::the_xmlns_prefix_can_never_become_a_name_prefix ... ok
test xml_spec::tests::test_split_qname_valid ... ok
test xml_spec::g3_tests::content_that_cannot_be_escaped_is_normalised_at_construction ... ok
test xml_spec::tests::test_split_qname_invalid ... ok
test xml_spec::tests::test_reserved_xmlns_prefix ... ok
test node::tests::repeated_names_share_one_allocation ... ok
test xml_spec::tests::test_comment_wrapper ... ok

test result: ok. 84 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/cloning.rs (target/debug/deps/cloning-7c0f7c6c487d1260)

running 15 tests
test cloning_a_handle_refers_to_the_same_node ... ok
test a_deep_clone_is_cheap_because_names_are_shared ... ok
test a_deep_copy_can_be_edited_independently_of_its_source ... ok
test a_copied_subtree_can_be_attached_in_the_target_document ... ok
test deep_clone_copies_the_whole_subtree ... ok
test deep_clone_into_another_document_rebuilds_the_tree_there ... ok
test display_of_non_element_nodes ... ok
test deep_clone_into_the_same_document_behaves_like_deep_clone ... ok
test display_renders_the_subtree_as_xml ... ok
test node_content_exposes_every_kind ... ok
test non_element_nodes_can_be_copied_between_documents ... ok
test shallow_clone_copies_the_payload_but_not_the_children ... ok
test shallow_clone_of_a_text_node_copies_the_content ... ok
test shallow_clone_into_another_document_keeps_only_the_node ... ok
test copying_while_the_source_is_mutated_produces_consistent_copies ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/concurrency.rs (target/debug/deps/concurrency-9b877b9de1686022)

running 5 tests
test a_node_handle_remains_valid_after_being_moved_by_another_thread ... ok
test handles_can_be_moved_between_threads ... ok
test structural_edits_stay_consistent_under_contention ... ok
test readers_and_writers_do_not_deadlock ... ok
test two_threads_adding_opposite_children_never_create_a_cycle ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s

     Running tests/element.rs (target/debug/deps/element-6e450ca5bb1739fb)

running 9 tests
test an_empty_default_declaration_removes_the_default_namespace ... ok
test declarations_are_inherited_and_shadowed ... ok
test attribute_crud_is_keyed_by_expanded_name ... ok
test names_accessors_agree ... ok
test invalid_attribute_values_are_rejected ... ok
test namespace_declarations_can_be_added_changed_and_removed ... ok
test removing_a_declaration_is_silent_even_though_the_subtree_relies_on_it ... ok
test resolution_uses_the_in_scope_declarations ... ok
test namespaced_documents_round_trip_through_the_parser ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/errors.rs (target/debug/deps/errors-333e2e796641fc33)

running 13 tests
test append_child_errors ... ok
test insert_child_errors ... ok
test element_errors ... ok
test namespace_construction_errors ... ok
test node_construction_errors ... ok
test parse_file_reports_io_errors ... ok
test qualified_name_construction_errors ... ok
test qualified_name_resolution_errors ... ok
test relative_insertion_errors ... ok
test replace_with_errors ... ok
test set_root_errors ... ok
test unclosed_elements_and_xml_target_pis_are_rejected ... ok
test parse_errors ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/io.rs (target/debug/deps/io-b802fa12f8c2fb5c)

running 35 tests
test a_document_without_a_root_serializes_to_nothing_useful_but_does_not_fail ... ok
test a_declaration_must_be_the_very_first_thing ... ok
test a_literal_carriage_return_in_text_round_trips ... ok
test adjacent_text_nodes_are_merged_on_output ... ok
test an_optional_utf8_byte_order_mark_is_accepted ... ok
test character_references_are_expanded ... ok
test cdata_and_processing_instructions_must_be_well_formed ... ok
test comments_cdata_and_processing_instructions_are_written_verbatim ... ok
test attribute_value_normalisation_in_both_directions ... ok
test declaration_style_is_honoured ... ok
test comments_must_be_well_formed ... ok
test empty_elements_and_write_options ... ok
test empty_text_nodes_have_no_representation_and_are_omitted ... ok
test illegal_character_references_are_rejected ... ok
test an_empty_default_declaration_removes_the_default_namespace ... ok
test invalid_utf8_is_a_typed_error ... ok
test less_than_is_rejected_in_attribute_values ... ok
test line_ends_are_normalised_in_text ... ok
test namespace_declarations_are_never_invented_or_removed ... ok
test predefined_entities_are_expanded ... ok
test namespaces_are_resolved_and_reported ... ok
test prefixes_are_preserved ... ok
test text_containing_a_cdata_close_round_trips ... ok
test tags_must_nest_and_close ... ok
test rules_referenced_by_the_parser_and_serializer_exist ... ok
test text_is_escaped_so_that_it_round_trips ... ok
test there_must_be_exactly_one_root_element ... ok
test processing_instruction_content_is_kept_verbatim ... ok
test the_declaration_is_interpreted_and_preserved ... ok
test parser_never_panics_on_odd_input ... ok
test unsupported_declarations_are_rejected ... ok
test undeclared_entity_references_are_typed_errors_not_panics ... ok
test top_level_content_policy ... ok
test xml_lang_and_xml_space_round_trip_with_the_implicit_xml_prefix ... ok
test a_deeply_nested_document_round_trips ... ok

test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s

     Running tests/properties.rs (target/debug/deps/properties-e94f36dc1c78b34a)

running 7 tests
test arbitrary_strings_never_panic ... ok
test arbitrary_bytes_never_panic ... ok
test round_trip_preserves_the_document ... ok
test writing_is_idempotent ... ok
test generated_documents_validate_clean ... ok
test the_declaration_survives_a_round_trip ... ok
test mutated_well_formed_documents_never_panic ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s

     Running tests/structure.rs (target/debug/deps/structure-ad68daaf7e0e18fe)

running 28 tests
test a_foreign_element_cannot_become_the_root ... ok
test a_detached_subtree_stays_editable_clonable_and_reattachable ... ok
test an_attached_element_cannot_become_the_root ... ok
test attaching_and_detaching_keeps_the_tree_consistent ... ok
test attaching_a_detached_subtree_restores_it ... ok
test attachment_state_is_only_about_the_root_of_the_document ... ok
test cycles_are_rejected ... ok
test detaching_is_idempotent_and_returns_the_previous_parent ... ok
test descendants_are_returned_in_document_pre_order ... ok
test foreign_documents_are_rejected ... ok
test mixed_content_node_kinds_are_preserved ... ok
test non_elements_cannot_be_parents ... ok
test insertion_positions_are_stable ... ok
test out_of_range_insertions_are_rejected ... ok
test panicking_append_child_on_a_foreign_document_leaves_everything_unchanged ... ok
test panicking_insert_child_leaves_the_document_unchanged ... ok
test panicking_append_child_leaves_the_document_unchanged ... ok
test panicking_set_attribute_leaves_the_document_unchanged ... ok
test relative_insertion_requires_a_sibling_of_the_same_parent ... ok
test panicking_replace_with_leaves_the_document_unchanged ... ok
test replacing_a_node_by_itself_is_a_documented_no_op ... ok
test replacing_swaps_the_node_in_place ... ok
test setting_the_root_returns_the_previous_one ... ok
test remove_returns_the_detached_node_itself ... ok
test panicking_set_root_leaves_the_document_unchanged ... ok
test sibling_and_index_queries_agree_with_the_child_list ... ok
test the_root_cannot_be_attached_as_a_child ... ok
test a_deeply_nested_document_does_not_overflow_the_stack ... ok

test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

     Running tests/validation.rs (target/debug/deps/validation-7ea8e33840eb447f)

running 21 tests
test a_prefix_bound_to_a_different_uri_is_reported ... ok
test a_detached_copy_does_not_conflict_with_its_original ... ok
test all_problems_are_reported_in_one_call ... ok
test a_consistent_document_has_no_structural_problems ... ok
test an_empty_document_has_no_root ... ok
test an_element_in_a_namespace_needs_it_declared ... ok
test attributes_are_not_affected_by_the_default_namespace ... ok
test every_error_kind_names_an_existing_rule_file ... ok
test detached_subtrees_are_validated_against_their_own_scope ... ok
test default_namespace_scope_must_match_the_name ... ok
test the_error_type_is_a_std_error ... ok
test moving_an_element_can_break_its_namespace_and_validation_says_so ... ok
test editing_is_silent_and_validation_is_what_reports ... ok
test the_xml_prefix_is_always_available ... ok
test the_xml_id_policy_is_pinned_in_both_directions ... ok
test xml_id_values_must_be_names_and_unique ... ok
test validation_does_not_change_the_document ... ok
test xml_lang_and_xml_space_values_are_checked ... ok
test validation_is_deterministic ... ok
test every_parsed_fixture_is_valid ... ok
test consistent_documents_produce_no_structural_issues ... ok

test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

     Running unittests src/lib.rs (target/debug/deps/biodivine_lib_xml_dom_sys-ff2822be7a0d9c69)

running 6 tests
test tests::handles_are_send_and_sync ... ok
test tests::handles_can_be_moved_into_a_plain_rust_thread ... ok
test tests::a_document_can_be_shared_by_several_threads ... ok
test tests::a_validation_failure_carries_every_problem ... ok
test tests::a_document_round_trips_through_the_bindings ... ok
test tests::a_parsed_document_is_the_same_tree_as_the_serialized_one ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests biodivine_lib_xml_dom

running 27 tests
test src/lib.rs - (line 107) ... ok
test src/document.rs - document::Document::validate (line 390) ... ok
test src/namespace.rs - namespace::Namespace::is_equal_ns (line 169) ... ok
test src/lib.rs - (line 63) ... ok
test src/namespace.rs - namespace::Namespace::new (line 58) ... ok
test src/namespace.rs - namespace::Namespace::prefix_str (line 153) ... ok
test src/namespace.rs - namespace::Namespace::prefixed (line 107) ... ok
test src/lib.rs - (line 87) ... ok
test src/lib.rs - (line 134) ... ok
test src/namespace.rs - namespace::Namespace::without_prefix (line 88) ... ok
test src/namespace.rs - namespace::Namespace::uri (line 122) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName (line 43) ... ok
test src/node.rs - node::Node (line 63) ... ok
test src/namespace.rs - namespace::Namespace::prefix (line 136) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName::resolve_attribute (line 195) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName::resolve_attribute_with_namespace_map (line 347) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName::resolve_element_with_namespace_map (line 318) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName::with_namespace (line 105) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName::without_namespace (line 87) ... ok
test src/xml_spec.rs - xml_spec::CData (line 282) ... ok
test src/xml_spec.rs - xml_spec::PiData (line 380) ... ok
test src/xml_spec.rs - xml_spec::PiTarget (line 348) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName::resolve_element (line 163) ... ok
test src/xml_spec.rs - xml_spec::NCName::as_str (line 220) ... ok
test src/xml_spec.rs - xml_spec::NCName (line 193) ... ok
test src/xml_spec.rs - xml_spec::Comment (line 313) ... ok
test src/xml_spec.rs - xml_spec::Text (line 239) ... ok

test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

all doctests ran in 0.14s; merged doctests compilation took 0.14s
   Doc-tests biodivine_lib_xml_dom_sys

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

-> exit 0

$ cargo build --examples
   Compiling biodivine-lib-xml-dom v0.2.0 (/sandbox/biodivine-lib-xml-dom)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.17s
-> exit 0

$ cargo run --quiet --example book_editing
<root><a/><y/><b/><c/><d/></root>
-> exit 0

$ cargo run --quiet --example book_getting_started
<ex:root xmlns:ex="http://example.com"><ex:child id="first">Hello, World!</ex:child></ex:root>
-> exit 0

$ cargo run --quiet --example book_namespaces
<ex:root><ex:child/></ex:root>
-> exit 0

$ cargo run --quiet --example book_parsing
<a>                          -> malformed XML: element `<a>` is never closed
<a/><b/>                     -> the document has more than one root element
text<a/>                     -> content is not allowed outside the root element
<a b="x<y"/>                 -> malformed XML: the value of attribute `b` contains a `<`
<a xmlns:p=""/>              -> invalid namespace: the namespace prefix `p` may not be undeclared with an empty value
<p:a/>                       -> undeclared namespace prefix `p`
<a a="1" a="2"/>             -> duplicate attribute `duplicate attribute at byte 8 (first declared at byte 2)`
<a>&undefined;</a>           -> undeclared entity reference `&undefined;` (only the predefined entities are supported)
<?xml version="1.0" encoding="UTF-8"?><ex:a xmlns:ex="http://e"><ex:b x="1 &amp; 2">t</ex:b><!--c--><![CDATA[<raw>]]><?pi data?></ex:a>
-> exit 0

$ cargo run --quiet --example book_pythonic
2 children
-> exit 0

$ cargo run --quiet --example book_threads
100 children, document valid
-> exit 0

$ cargo run --quiet --example book_validation
0: the element name `ex:root` uses the prefix `ex`, which is not declared in scope [rule.namespace-usage.prefix-declared.md]
0: `xml:id` value "not a name" is not a valid NCName [rule.attributes.id-must-be-name.md]
0: `xml:space` value "preserve-everything" is neither `default` nor `preserve` [rule.document-structure.xml-space-must-be-enumerated-default-preserve.md]
ok
-> exit 0

$ cargo run --quiet --example tour
XML DOM library tour
====================

1. Building a document programmatically
<html:html xmlns:html="http://www.w3.org/1999/xhtml" xmlns:svg="http://www.w3.org/2000/svg"><html:head><html:title>My XML Document</html:title></html:head><html:body><html:p class="example" id="intro">This document was created with the DOM API rather than parsed from text.</html:p><svg:svg><svg:circle fill="blue" r="40"/></svg:svg></html:body></html:html>

2. Parsing XML text
root: bookstore
  Harry Potter [fiction]
  Learning Rust [non-fiction]

3. Inspecting child nodes
  0: text "Hello "
  1: element <b>
  2: text ", welcome!"

4. Editing: move, clone and detach
deep clone is detached: true
bookstore now has 3 books
removed book -> 2 books left, removed node still usable: book

5. Copying between documents
imported subtree has 2 children in the target document

6. Comments, CDATA and processing instructions
<root><!-- a header comment --><![CDATA[raw <content> & text]]><?xml-stylesheet type="text/css" href="a.css"?>plain text</root>
-> exit 0

$ env RUSTDOCFLAGS=-D warnings cargo doc --no-deps --workspace
    Checking biodivine-lib-xml-dom v0.2.0 (/sandbox/biodivine-lib-xml-dom)
 Documenting biodivine-lib-xml-dom v0.2.0 (/sandbox/biodivine-lib-xml-dom)
   Compiling pyo3 v0.29.3
 Documenting biodivine-lib-xml-dom-py-sys v0.2.0 (/sandbox/biodivine-lib-xml-dom/biodivine-lib-xml-dom-py-sys)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.05s
   Generated /sandbox/biodivine-lib-xml-dom/target/doc/biodivine_lib_xml_dom/index.html and 1 other file
-> exit 0

$ /sandbox/biodivine-lib-xml-dom/.venv/bin/python docs/check_doc_sections.py --self-test
self-test corpus: inspected 3 public functions in /tmp/tmp2cv56c6i
self-test corpus: inspected 1 public functions in /tmp/tmp2cv56c6i
self-test: the checker reports missing sections and enforces its floor
-> exit 0

$ /sandbox/biodivine-lib-xml-dom/.venv/bin/python docs/check_doc_sections.py
the core crate: inspected 145 public functions in src
the binding crate: inspected 156 public functions in biodivine-lib-xml-dom-py-sys/src
all public items follow the `# Errors` / `# Panics` conventions
-> exit 0

$ /sandbox/biodivine-lib-xml-dom/.venv/bin/python docs/check_book.py
book sources: 12 chapters, 19 language pairs
the book is well-formed
-> exit 0

$ bash docs/build_docs.sh

=== tool versions
python 3.11.2
sphinx 9.0.4
pytest 9.1.1
cargo 1.99.0 (5f94df478 2026-08-27)
rustc 1.99.0 (b940084d7 2026-09-28)
maturin 1.15.0

=== 1. cargo doc (Rust API, warnings denied)
   Compiling pyo3-ffi v0.29.3
   Compiling pyo3 v0.29.3
 Documenting biodivine-lib-xml-dom-py-sys v0.2.0 (/sandbox/biodivine-lib-xml-dom/biodivine-lib-xml-dom-py-sys)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.19s
   Generated /sandbox/biodivine-lib-xml-dom/target/doc/biodivine_lib_xml_dom/index.html and 1 other file
rustdoc: 8 index pages under target/doc

=== 2. documentation sections (# Errors / # Panics)
self-test corpus: inspected 3 public functions in /tmp/tmpna11v6g9
self-test corpus: inspected 1 public functions in /tmp/tmpna11v6g9
self-test: the checker reports missing sections and enforces its floor
the core crate: inspected 145 public functions in src
the binding crate: inspected 156 public functions in biodivine-lib-xml-dom-py-sys/src
all public items follow the `# Errors` / `# Panics` conventions

=== 3. book sources (structure and language pairs)
book sources: 12 chapters, 19 language pairs
the book is well-formed

=== 4. native extension (needed by Sphinx autodoc)
biodivine_lib_xml_dom is importable

=== 5. Python API reference (Sphinx autodoc)
Running Sphinx v9.0.4
loading translations [en]... done
making output directory... done
loading intersphinx inventory 'python' from https://docs.python.org/3/objects.inv ...
myst v5.1.0: MdParserConfig(commonmark_only=False, gfm_only=False, enable_extensions={'colon_fence', 'deflist'}, disable_syntax=[], all_links_external=False, links_external_new_tab=False, url_schemes=('http', 'https', 'mailto', 'ftp'), ref_domains=None, fence_as_directive=set(), number_code_blocks=[], title_to_header=False, heading_anchors=0, heading_slug_func=None, html_meta={}, footnote_sort=True, footnote_transition=True, words_per_minute=200, substitutions={}, linkify_fuzzy_links=True, dmath_allow_labels=True, dmath_allow_space=True, dmath_allow_digits=True, dmath_double_inline=False, update_mathjax=True, mathjax_classes='tex2jax_process|mathjax_process|math|output_area', enable_checkboxes=False, strikethrough_single_tilde=False, colon_fence_exact_match=False, suppress_warnings=[], highlight_code_blocks=True)
building [mo]: targets for 0 po files that are out of date
writing output... 
building [html]: targets for 2 source files that are out of date
updating environment: [new config] 2 added, 0 changed, 0 removed
reading sources... [ 50%] api
reading sources... [100%] index

looking for now-outdated files... none found
pickling environment... done
checking consistency... done
preparing documents... done
copying assets... 
copying static files... 
Writing evaluated template result to /sandbox/biodivine-lib-xml-dom/biodivine-lib-xml-dom-py-sys/docs/_build/_static/basic.css
Writing evaluated template result to /sandbox/biodivine-lib-xml-dom/biodivine-lib-xml-dom-py-sys/docs/_build/_static/documentation_options.js
Writing evaluated template result to /sandbox/biodivine-lib-xml-dom/biodivine-lib-xml-dom-py-sys/docs/_build/_static/language_data.js
Writing evaluated template result to /sandbox/biodivine-lib-xml-dom/biodivine-lib-xml-dom-py-sys/docs/_build/_static/alabaster.css
copying static files: done
copying extra files... 
copying extra files: done
copying assets: done
writing output... [ 50%] api
writing output... [100%] index

generating indices... genindex py-modindex done
writing additional pages... search done
dumping search index in English (code: en)... done
dumping object inventory... done
build succeeded.

The HTML pages are in biodivine-lib-xml-dom-py-sys/docs/_build.
python api docs: 5 HTML files

=== 6. the book
Running Sphinx v9.0.4
loading translations [en]... done
making output directory... done
myst v5.1.0: MdParserConfig(commonmark_only=False, gfm_only=False, enable_extensions={'attrs_inline', 'colon_fence', 'deflist'}, disable_syntax=[], all_links_external=False, links_external_new_tab=False, url_schemes=('http', 'https', 'mailto', 'ftp'), ref_domains=None, fence_as_directive=set(), number_code_blocks=[], title_to_header=False, heading_anchors=3, heading_slug_func=None, html_meta={}, footnote_sort=True, footnote_transition=True, words_per_minute=200, substitutions={}, linkify_fuzzy_links=True, dmath_allow_labels=True, dmath_allow_space=True, dmath_allow_digits=True, dmath_double_inline=False, update_mathjax=True, mathjax_classes='tex2jax_process|mathjax_process|math|output_area', enable_checkboxes=False, strikethrough_single_tilde=False, colon_fence_exact_match=False, suppress_warnings=[], highlight_code_blocks=True)
building [mo]: targets for 0 po files that are out of date
writing output... 
building [html]: targets for 12 source files that are out of date
updating environment: [new config] 12 added, 0 changed, 0 removed
reading sources... [  8%] building-documents
reading sources... [ 17%] design-notes
reading sources... [ 25%] getting-started
reading sources... [ 33%] index
reading sources... [ 42%] limitations
reading sources... [ 50%] migration
reading sources... [ 58%] namespaces
reading sources... [ 67%] parsing-and-serializing
reading sources... [ 75%] python-usage
reading sources... [ 83%] thread-safety
reading sources... [ 92%] traversing-and-editing
reading sources... [100%] validation

looking for now-outdated files... none found
pickling environment... done
checking consistency... done
preparing documents... done
copying assets... 
copying static files... 
Writing evaluated template result to /sandbox/biodivine-lib-xml-dom/docs/book/_build/_static/basic.css
Writing evaluated template result to /sandbox/biodivine-lib-xml-dom/docs/book/_build/_static/documentation_options.js
Writing evaluated template result to /sandbox/biodivine-lib-xml-dom/docs/book/_build/_static/language_data.js
Writing evaluated template result to /sandbox/biodivine-lib-xml-dom/docs/book/_build/_static/alabaster.css
copying static files: done
copying extra files... 
copying extra files: done
copying assets: done
writing output... [  8%] building-documents
writing output... [ 17%] design-notes
writing output... [ 25%] getting-started
writing output... [ 33%] index
writing output... [ 42%] limitations
writing output... [ 50%] migration
writing output... [ 58%] namespaces
writing output... [ 67%] parsing-and-serializing
writing output... [ 75%] python-usage
writing output... [ 83%] thread-safety
writing output... [ 92%] traversing-and-editing
writing output... [100%] validation

generating indices... genindex done
writing additional pages... search done
dumping search index in English (code: en)... done
dumping object inventory... done
build succeeded.

The HTML pages are in docs/book/_build.
book: 14 HTML files

=== 7. built book (chapter titles and language tabs)
book sources: 12 chapters, 19 language pairs
built book: 14 pages, 57 tab-set occurrences
the book is well-formed

=== summary
all documentation built and checked
  rustdoc            target/doc/biodivine_lib_xml_dom/index.html
  python api docs    biodivine-lib-xml-dom-py-sys/docs/_build/index.html
  book               docs/book/_build/index.html
-> exit 0

$ /sandbox/biodivine-lib-xml-dom/.venv/bin/python docs/check_book.py --built
book sources: 12 chapters, 19 language pairs
built book: 14 pages, 57 tab-set occurrences
the book is well-formed
-> exit 0

$ /sandbox/biodivine-lib-xml-dom/.venv/bin/python -m pytest biodivine-lib-xml-dom-py-sys/tests-python
============================= test session starts ==============================
platform linux -- Python 3.11.2, pytest-9.1.1, pluggy-1.6.0
rootdir: /sandbox/biodivine-lib-xml-dom/biodivine-lib-xml-dom-py-sys
configfile: pyproject.toml
collected 41 items

biodivine-lib-xml-dom-py-sys/tests-python/test_book_examples.py ........ [ 19%]
                                                                         [ 19%]
biodivine-lib-xml-dom-py-sys/tests-python/test_public_surface.py ....... [ 36%]
.                                                                        [ 39%]
biodivine-lib-xml-dom-py-sys/tests-python/test_xml_dom.py .............. [ 73%]
...........                                                              [100%]

============================== 41 passed in 0.07s ==============================
-> exit 0

=== summary
all gates passed

```
