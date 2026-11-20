# make-mate

`mate` adds task discovery, tab completion, and positional arguments to the Makefile you already have. It's a thin wrapper around GNU Make: your teammates can keep using plain `make` without installing anything new.

## Install

`make-mate` is in alpha and isn't on PyPI yet. With [uv](https://docs.astral.sh/uv/) and a Rust toolchain (1.88 or newer), install from a checkout:

```sh
git clone https://github.com/Singletoned/make-mate.git
cd make-mate
uv tool install .
mate --version
```

If you already use Cargo, run `cargo install --path . --locked` from the checkout instead. Make sure GNU Make is available as `make` on your `PATH`. The initial wheel targets are Linux x86-64 and macOS on Apple Silicon; Windows isn't tested.

## Try it

In a scratch directory, save this as `Makefile` (use a tab before the recipe):

```make
.PHONY: greet

# args: name greeting?=Hello
greet: ## greet someone
	@printf '%s, %s!\n' "$(GREETING)" "$(NAME)"
```

Discover tasks and their arguments:

```console
$ mate --list
greet <name> [greeting=Hello]  greet someone
```

Run the task either way:

```sh
mate greet 'Ada Lovelace'
make greet NAME='Ada Lovelace' GREETING=Hello
```

Both print `Hello, Ada Lovelace!`. You can list and complete tasks in existing Makefiles without adding metadata. Use `##` for descriptions and `# args:` for positional arguments. See the [Makefile conventions](docs/conventions.md), or run `mate --makefile-syntax` for a quick reference and `mate --llm` for coding-agent guidance.

Only use `mate` with Makefiles you trust: listing and completion ask GNU Make to read the file, which can evaluate expressions such as `$(shell ...)`.

## Shell completion

In zsh, with `compinit` already initialized:

```sh
mate --shell-completion zsh > /tmp/mate.zsh
source /tmp/mate.zsh
```

Type `mate ` and press Tab in your example directory. See [shell setup](docs/completion.md) for Bash, fish, and persistent installation.

## Development

Install Bash, zsh, and fish to run the shell integration tests. Useful commands:

```sh
make check
make test
make smoke
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
```

See [releasing](docs/releasing.md) for the wheel build and release process.

## License

[MIT](LICENSE).
