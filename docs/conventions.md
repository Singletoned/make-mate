# make-mate Makefile conventions

`make-mate` is intentionally a small layer over plain `make`. A Makefile written for `mate` should still be useful when someone runs `make` directly.

Print the short built-in reference with:

```sh
mate --makefile-syntax
```

## Task targets

Declare human-facing tasks as `.PHONY` targets:

```make
.PHONY: test release

test: ## run tests
	cargo test
```

When `.PHONY` targets are present, `mate` prefers them for completion so file targets and build artifacts do not clutter the menu.

## Descriptions

Use the common `##` convention to describe targets:

```make
test: ## run tests
	cargo test
```

`mate --list` and shell completion display these descriptions.

When a target declares arguments, `mate --list` shows them next to the target name and aligns descriptions in a readable column:

```text
release <version> [target=all]  publish a release
```

## Arguments

Declare positional arguments in a comment immediately before the target:

```make
# args: version target?=all
release: ## publish a release
	./scripts/release.sh $(VERSION) $(TARGET)
```

Then run:

```sh
mate release 1.2.3
```

`mate` invokes plain `make` with ordinary make variables:

```sh
make release VERSION=1.2.3 TARGET=all
```

The plain `make` form remains valid and should be documented enough for users who do not have `mate` installed.

### Quoted arguments with spaces

Quote positional arguments that contain spaces so the shell passes each value as one argument:

```make
.PHONY: new idea

# args: title
new: ## create a new post
	./scripts/new-post.sh "$(TITLE)"

# args: thought
idea: ## capture a rough thought
	./scripts/capture-idea.sh "$(THOUGHT)"
```

Then run:

```sh
mate new "Post title"
mate idea "Rough thought"
```

`mate` passes those values through as ordinary make variables:

```sh
make new TITLE="Post title"
make idea THOUGHT="Rough thought"
```

Quote the make variable inside the recipe too, as in `"$(TITLE)"`, so scripts receive the value as one argument.

## Argument syntax

- `name` declares a required positional argument.
- `name?` declares an optional positional argument with no default.
- `name?=default` declares an optional positional argument with a default value.

Argument names are converted to make variable names by replacing `-` with `_` and uppercasing:

| Argument     | Make variable |
| ------------ | ------------- |
| `version`    | `VERSION`     |
| `target-env` | `TARGET_ENV`  |

## Good agent-generated Makefiles

Prefer simple, legible Makefiles:

- Put task metadata directly above the target it describes.
- Use `.PHONY` for human-facing tasks.
- Keep recipes valid under plain `make`.
- Use ordinary make variables such as `$(VERSION)` in recipes.
- Avoid catch-all targets or hacks that change how plain `make` behaves.
- Avoid relying on dynamic shell-side completion logic.

## Out of scope for now

`mate` does not currently provide argument value completion. The convention leaves room for a future companion config file, but the first version only completes targets, descriptions, and argument placeholders.
