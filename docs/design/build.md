# Building, testing and documenting the project

Everything needed to set the repository up from a fresh checkout, to run the checks, and to build
the documentation. Every command below is runnable from the repository root and is also available as
a `make` target (see the table at the end).

## 1. What the repository contains

| directory | contents |
| --- | --- |
| `src/` | the Rust library (`biodivine-lib-xml-dom`) |
| `biodivine-lib-xml-dom-py-sys/` | the thin PyO3 wrappers (the `_sys` crate; not published) |
| `tests/`, `examples/` | integration tests and runnable examples |
| `specification/` | the XML and XML Namespaces specifications, and the rule summaries |
| `docs/design/` | design notes, this document, the bindings map, the rule coverage |
| `docs/assets/` | diagrams shared by the crate documentation and the Biodivine book |
| `scripts/` | the verification entry point and the documentation generators |

The Python interface and the tutorial book are not built here:

* The wrapper crate is linked into the `biodivine-sys` extension, and the Python module
  `biodivine.xml_dom` wraps it; both are in the
  [`biodivine-py`](https://github.com/sybila/biodivine-py) repository, together with the Python
  test suite and the Python API reference. A change to the wrappers is tested there as well.
* The Biodivine book, with the tutorial chapters of this library and their Rust and Python
  examples, is part of the [`biodivine`](https://github.com/sybila/biodivine) workspace repository,
  which has this repository and `biodivine-py` as submodules.

## 2. Rust

Install a toolchain with [rustup](https://rustup.rs). The crate builds with stable Rust; the declared
minimum version is the `rust-version` in `Cargo.toml` and is tested by the verification script, so
that value — not this document — is authoritative.

```sh
rustup toolchain install stable
cargo build --workspace          # both crates
cargo test --workspace           # unit, integration and documentation tests
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
```

The tests of the wrapper crate start a Python interpreter in-process, so `cargo test --workspace`
links against the Python library. Install a development version of Python (`python3-dev` on
Debian/Ubuntu) first, or test only the core crate with `cargo test -p biodivine-lib-xml-dom`.

## 3. Documentation

```sh
make docs          # or: bash docs/build_docs.sh
```

builds the Rust API reference (`target/doc/biodivine_lib_xml_dom/index.html`) with warnings denied,
and runs the documentation checkers. `cargo doc --no-deps --workspace --open` builds and opens it.
The design documents in `docs/design/` (except this one) are part of the crate documentation, in the
`documentation` module.

## 4. Checks

```sh
make verify        # or: bash scripts/verify.sh
```

The script runs every check and prints each command with its exit code: formatting, clippy in both
profiles, the test suites in debug and release, the declared minimum Rust version and the CI-pinned
toolchain, Miri, every example, the rustdoc build (of the core crate on its own, as docs.rs builds
it, and of the whole workspace, with and without private items) with warnings denied, and the
documentation checkers. It is the command to run before opening a pull request. The repository
scripts need only a plain `python3`.

Narrower targets:

| command | what it checks |
| --- | --- |
| `make lint` | formatting and clippy |
| `make examples` | runs every Rust example |
| `python3 scripts/gen_rule_set.py --check` | `xml_spec::rule_set` matches `specification/rules/` |
| `python3 scripts/gen_rule_coverage.py --check` | the rule coverage table is up to date |
| `python3 scripts/gen_diagrams.py --check` | the diagrams embedded in the documents match `docs/assets/` |
| `python3 docs/check_doc_sections.py` | every public function that can fail or panic documents it |

## 5. Undefined behaviour (Miri)

The data structures are exercised under [Miri](https://github.com/rust-lang/miri), which needs a
nightly toolchain with the `miri` component:

```sh
rustup toolchain install nightly --profile minimal --component miri,rust-src
cargo +nightly miri setup
MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri test --lib
cargo +nightly miri test --test gc -- --skip handles_released_by_another_thread
```

`scripts/verify.sh` runs both when the component is available and reports the run as *skipped* when
it is not — and it counts the skip as a failure, so a missing component can never be mistaken for a
pass. In other words, **a fully green `make verify` needs a nightly toolchain with Miri installed**;
without it every other gate still runs, and the summary names the missing one. The two-thread
garbage-collection test is excluded from the second command: `parking_lot` aborts inside Miri's
futex shim, so lock contention cannot be checked there. `-Zmiri-disable-isolation` is only needed
because a test helper reads the rule summaries from the working tree.

## 6. Make targets

| target | runs |
| --- | --- |
| `make build` | `cargo build --workspace` |
| `make test` | `cargo test --workspace` |
| `make lint` | `cargo fmt --check`, `cargo clippy` with warnings denied |
| `make examples` | runs every `examples/*.rs` |
| `make docs` | rustdoc, plus the documentation checkers |
| `make verify` | `scripts/verify.sh` |
