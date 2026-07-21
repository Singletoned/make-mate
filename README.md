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
mate --makefile-syntax
mate --llm-guidance
```

`mate release 1.2.3` invokes plain make as:

```sh
make release VERSION=1.2.3 TARGET=all
```

The equivalent plain-`make` command remains valid.

## Installation

### Current checkout for local testing

From this repository, install the latest local Rust binary with:

```sh
cargo install --path . --locked --force
```

Confirm the installed command is the version you expect:

```sh
which mate
mate --version
mate --list
```

You can also install through the intended Python packaging path from this checkout:

```sh
uv tool install --force .
mate --version
```

Uninstall that tool-managed copy with:

```sh
uv tool uninstall make-mate
```

### Shell completion

Generate completion glue from the installed command and source it in your shell. For example, in zsh:

```sh
mate --shell-completion zsh > /tmp/mate.zsh
source /tmp/mate.zsh
```

See [`docs/completion.md`](docs/completion.md) for bash, fish, and more debugging commands.

### Future PyPI install

The project is intended to be distributed as a native Rust binary in Python wheels so users can eventually install it with:

```sh
uv tool install make-mate
```

## Development

Useful recipes:

```sh
make help
make test
make test-shells
make smoke
make install-dev
make completion
make package
```

Or run commands directly:

```sh
cargo test
cargo run -- --help
cargo run -- --makefile-syntax
cargo run -- --llm-guidance
maturin build
```
