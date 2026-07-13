# Shell completion

`mate` keeps shell integration deliberately small. The shell stub calls back into the binary with:

```sh
mate --complete -- <words>
```

All completion logic lives in the Rust binary, not in regenerated shell scripts.

## zsh

For local testing in the current shell:

```sh
mate --shell-completion zsh > /tmp/mate.zsh
source /tmp/mate.zsh
```

Then try:

```sh
mate <TAB>
mate --<TAB>
mate release <TAB>
```

For a persistent setup, place the generated file somewhere already on your zsh `fpath`, or source it from your shell startup file.

## bash

For local testing:

```sh
mate --shell-completion bash > /tmp/mate.bash
source /tmp/mate.bash
```

## fish

For local testing:

```fish
mate --shell-completion fish > /tmp/mate.fish
source /tmp/mate.fish
```

## Debugging completion

Check the installed binary and version:

```sh
which mate
mate --version
```

Check the completion contract directly:

```sh
mate --complete -- --
mate --complete -- rel
mate --complete -- release ""
```

If completion behaves oddly after reinstalling, regenerate and re-source the shell stub. The stub can change during development.
