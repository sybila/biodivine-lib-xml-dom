# Convenience entry points. Everything here is also runnable by hand; see README.md.
.PHONY: help build test test-python lint docs verify python-extension examples

help:
	@echo "make build            build the Rust workspace"
	@echo "make test             run the Rust test suite (both crates)"
	@echo "make python-extension build the native extension into .venv (maturin develop)"
	@echo "make test-python      run the Python test suite (needs python-extension)"
	@echo "make examples         run every Rust example of the documentation book"
	@echo "make lint             cargo fmt --check, cargo clippy, pytest"
	@echo "make docs             build rustdoc, the Python API reference and the book"
	@echo "make verify           run every gate and print each command with its exit code"

build:
	cargo build --workspace

test:
	cargo test --workspace

python-extension:
	cd biodivine-lib-xml-dom-py-sys && VIRTUAL_ENV=$(CURDIR)/.venv ../.venv/bin/maturin develop --release

test-python: python-extension
	.venv/bin/python -m pytest biodivine-lib-xml-dom-py-sys/tests-python

examples:
	@for example in $$(ls examples/book_*.rs | sed 's|examples/||; s|\.rs||'); do \
		echo "== $$example"; cargo run --quiet --example $$example >/dev/null || exit 1; \
	done

lint:
	cargo fmt --check
	cargo clippy --workspace --all-targets
	.venv/bin/python -m pytest biodivine-lib-xml-dom-py-sys/tests-python

docs:
	docs/build_docs.sh

verify:
	scripts/verify.sh
