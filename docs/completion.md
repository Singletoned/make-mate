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

For a persistent setup, save the generated file somewhere permanent and source it from `~/.zshrc` after `compinit`.

## bash

For local testing:

```sh
mate --shell-completion bash > /tmp/mate.bash
source /tmp/mate.bash
```

For a persistent setup, save the file somewhere permanent and source it from `~/.bashrc`.

## fish

For local testing:

```fish
mate --shell-completion fish > /tmp/mate.fish
source /tmp/mate.fish
```

For a persistent setup, save it as `~/.config/fish/completions/mate.fish`.

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
