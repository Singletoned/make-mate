# make-mate

`make-mate` provides the `mate` command: a thin wrapper around `make` for task-style Makefiles.

It is aimed at Makefiles that are mostly `.PHONY` entries and keeps the Makefile useful under plain `make`.

## Current conventions

See [`docs/conventions.md`](docs/conventions.md) for the full Makefile convention and [`docs/completion.md`](docs/completion.md) for shell setup.

Describe targets with the common `##` convention:

```make
.PHONY: test release

test: ## run tests
	cargo test

# args: version target?=all
release: ## publish a release
	./scripts/release.sh $(VERSION) $(TARGET)
```

Then run:

```sh
mate --version
mate --list
mate test
mate release 1.2.3
mate --shell-completion zsh
```

`mate release 1.2.3` invokes plain make as:

```sh
make release VERSION=1.2.3 TARGET=all
```

The equivalent plain-`make` command remains valid.

## Development

Useful recipes:

```sh
make help
make test
make test-shells
make smoke
make install-dev
make package
```

Or run commands directly:

```sh
cargo test
cargo run -- --help
maturin build
```

## Installation goal

The project is intended to be distributed as a native Rust binary in Python wheels so users can install it with:

```sh
uv tool install make-mate
```
