COMPLETION_SHELL ?= zsh

.PHONY: help test test-contract test-shells check fmt taidy build package install-dev version list smoke smoke-completion smoke-args completion clean

help: ## list available recipes
	@awk 'BEGIN {FS = ":.*##"} /^[a-zA-Z0-9_-]+:.*##/ {printf "%-24s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

test: ## run all Rust tests
	cargo test

test-contract: ## run completion contract integration tests
	cargo test --test completion_contract

test-shells: ## run shell completion smoke tests
	cargo test --test shell_completion

check: ## run Rust checks
	cargo check

fmt: ## format source files
	cargo fmt

taidy: ## format and lint supported project files with taidy
	taidy Cargo.toml pyproject.toml src/main.rs src/lib.rs Makefile README.md PROJECT.md docs/conventions.md docs/completion.md tests/completion_contract.rs tests/shell_completion.rs tests/fixtures/basic/Makefile tests/fixtures/phony-filter/Makefile

build: ## build the CLI
	cargo build --bin mate

package: ## build Python wheels with maturin
	maturin build

install-dev: ## install the current checkout as the local mate command
	cargo install --path . --locked --force

version: ## print the dev binary version
	cargo run --quiet --bin mate -- --version

list: ## list targets from this Makefile with the dev binary
	cargo run --quiet --bin mate -- --list

smoke: version list smoke-completion smoke-args ## run quick local smoke checks

smoke-completion: ## smoke-test completion output from fixtures
	cargo run --quiet --bin mate -- --complete -- --
	cargo run --quiet --bin mate -- -f tests/fixtures/basic/Makefile --complete -- rel
	cargo run --quiet --bin mate -- -f tests/fixtures/basic/Makefile --complete -- release ""

smoke-args: ## smoke-test positional argument mapping through make
	cargo run --quiet --bin mate -- -f tests/fixtures/basic/Makefile release 1.2.3

# args: completion_shell?=zsh
completion: ## print shell completion glue; pass COMPLETION_SHELL=bash|zsh|fish
	cargo run --quiet --bin mate -- --shell-completion $(COMPLETION_SHELL)

clean: ## remove build artifacts
	cargo clean
	dist=$$(find . -maxdepth 1 -type d -name dist -print); if [ -n "$$dist" ]; then rm -rf $$dist; fi
