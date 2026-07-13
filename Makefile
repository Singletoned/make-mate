.PHONY: test test-shells check fmt build package

test: ## run Rust tests
	cargo test

test-shells: ## run shell completion smoke tests
	cargo test --test shell_completion

check: ## run Rust checks
	cargo check

fmt: ## format source files
	cargo fmt

build: ## build the CLI
	cargo build

package: ## build Python wheels with maturin
	maturin build
