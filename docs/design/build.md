# Building, testing and documenting the project

Everything needed to set the repository up from a fresh checkout, to run the checks, and to build
the three documentation artifacts. Every command below is runnable from the repository root and is
also available as a `make` target (see the table at the end).

## 1. What the repository contains

| directory | contents |
| --- | --- |
| `src/` | the Rust library (`biodivine-lib-xml-dom`) |
| `biodivine-lib-xml-dom-py-sys/` | the PyO3 bindings plus the pure-Python package |
| `tests/`, `examples/` | integration tests and the examples the book includes |
| `specification/` | the XML and XML Namespaces specifications, and the rule summaries |
| `docs/book/` | the documentation book (Sphinx + MyST) |
| `docs/design/` | design notes, this document, the bindings map, the rule coverage |
| `docs/assets/` | diagrams shared by the crate documentation and the book |
| `scripts/` | the verification entry point and the documentation generators |

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

The Python bindings crate is part of the workspace, so `cargo build --workspace` links against the
Python library. Install a development version of Python (`python3-dev` on Debian/Ubuntu) first, or
build only the core crate with `cargo build -p biodivine-lib-xml-dom`.

## 3. Python

The Python package is a thin Pythonic layer over the native extension, so the extension has to be
built before the package can be imported or documented. [maturin](https://maturin.rs) does that:

```sh
python3 -m venv .venv
.venv/bin/pip install --upgrade pip
.venv/bin/pip install maturin pytest
make python-extension     # maturin develop --release into .venv
make test-python          # pytest against the built extension
```

The extension is installed in development mode, so after changing Rust code it has to be rebuilt
before the Python tests see the change.

## 4. Documentation

Three artifacts are built from this repository:

| artifact | toolchain | output |
| --- | --- | --- |
| Rust API reference | rustdoc (part of the Rust toolchain) | `target/doc/biodivine_lib_xml_dom/index.html` |
| Python API reference | Sphinx with autodoc | `biodivine-lib-xml-dom-py-sys/docs/_build/index.html` |
| the book | Sphinx with MyST, sphinx-design and the Shibuya theme | `docs/book/_build/index.html` |

The Python tools are pinned in `docs/requirements.txt`:

```sh
.venv/bin/pip install -r docs/requirements.txt
```

Build everything in one step (this also builds the extension if it is missing, and runs the
documentation checkers):

```sh
make docs          # or: bash docs/build_docs.sh
```

Individual pieces:

```sh
cargo doc --no-deps --workspace --open                     # Rust API
.venv/bin/python -m sphinx -b html biodivine-lib-xml-dom-py-sys/docs biodivine-lib-xml-dom-py-sys/docs/_build
.venv/bin/python -m sphinx -b html docs/book docs/book/_build
```

Building the Python reference imports the package, so the extension has to exist. The book embeds
files from `examples/` (compiled by `cargo test`, run by `make examples`) and
`docs/book/examples/python/` (run by the Python test suite).

## 5. Checks

```sh
make verify        # or: bash scripts/verify.sh
```

The script runs every check and prints each command with its exit code: formatting, clippy in both
profiles, the test suites in debug and release, the declared minimum Rust version and the CI-pinned
toolchain, Miri, every example, the rustdoc build (of each crate on its own and of the workspace)
with warnings denied, the documentation checkers, the three documentation artifacts, and the Python
test suite. It is the command to run before opening a pull request.

Narrower targets:

| command | what it checks |
| --- | --- |
| `make lint` | formatting, clippy, Python tests |
| `make examples` | runs every Rust example |
| `python3 scripts/gen_rule_set.py --check` | `xml_spec::rule_set` matches `specification/rules/` |
| `python3 scripts/gen_rule_coverage.py --check` | the rule coverage table is up to date |
| `python3 scripts/gen_diagrams.py --check` | the diagrams embedded in the documents match `docs/assets/` |
| `python3 docs/check_doc_sections.py` | every public function that can fail or panic documents it |
| `python3 docs/check_book.py` | every book example has a Rust and a Python version, included from a real file |

## 6. Undefined behaviour (Miri)

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

## 7. Make targets

| target | runs |
| --- | --- |
| `make build` | `cargo build --workspace` |
| `make test` | `cargo test --workspace` |
| `make lint` | `cargo fmt --check`, `cargo clippy` with warnings denied, `pytest` |
| `make python-extension` | `maturin develop --release` into `.venv` |
| `make test-python` | the Python test suite (depends on the extension) |
| `make examples` | runs every `examples/*.rs` |
| `make docs` | rustdoc, the Python reference and the book, plus the checkers |
| `make verify` | `scripts/verify.sh` |
