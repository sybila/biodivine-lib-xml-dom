# Convenience entry points. Everything here is also runnable by hand; see docs/design/build.md.
.PHONY: help build test lint docs verify examples

help:
	@echo "make build            build the Rust workspace"
	@echo "make test             run the Rust test suite (both crates)"
	@echo "make examples         run every Rust example"
	@echo "make lint             cargo fmt --check, cargo clippy (warnings denied)"
	@echo "make docs             build rustdoc and run the documentation checkers"
	@echo "make verify           run every gate and print each command with its exit code"

build:
	cargo build --workspace

test:
	cargo test --workspace

examples:
	@for example in $$(ls examples/*.rs | sed 's|examples/||; s|\.rs||'); do \
		echo "== $$example"; cargo run --quiet --example $$example >/dev/null || exit 1; \
	done

lint:
	cargo fmt --check
	cargo clippy --workspace --all-targets -- -D warnings

docs:
	docs/build_docs.sh

verify:
	scripts/verify.sh
