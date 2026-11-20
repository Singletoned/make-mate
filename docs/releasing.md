# Releasing make-mate

[GitHub CI](../.github/workflows/ci.yml) runs Rust checks and builds and installs release wheels on Linux x86-64 (glibc 2.28+) and Apple Silicon macOS. Pushes to `main` and pull requests do not publish anything.

## One-time setup

1. In GitHub, create an Actions environment named `pypi`. Restrict deployments to release tags (`v*`) and require a reviewer if available.
2. In [PyPI's publishing settings](https://pypi.org/manage/account/publishing/), add a pending [Trusted Publisher](https://docs.pypi.org/trusted-publishers/creating-a-project-through-oidc/) for the first release:

   | Field             | Value         |
   | ----------------- | ------------- |
   | PyPI project name | `make-mate`   |
   | Owner             | `Singletoned` |
   | Repository        | `make-mate`   |
   | Workflow filename | `release.yml` |
   | Environment name  | `pypi`        |

   No PyPI API token is needed. If you already configured a publisher for `ci.yml`, replace it with one for `release.yml`. If the project already exists under your account, configure the publisher on the project instead.

## Cut a release

Install Python 3.11+ and [the GitHub CLI](https://cli.github.com/), and authenticate with `gh auth login`. Ensure `release.yml` is already on `origin/main` before the first release.

From `main`, run:

```sh
mate release 0.1.15
# Or, without mate:
make release VERSION=0.1.15
```

The command prepares the requested version from committed `main` in a temporary worktree. It updates `Cargo.toml`, `Cargo.lock`, and `pyproject.toml` if needed; it doesn't make a version commit when the requested version is already set. After local checks pass, write the release notes in the tag editor. The command pushes `main` and the annotated tag atomically, then waits for the matching GitHub Actions run.

[`release.yml`](../.github/workflows/release.yml) rebuilds and tests both wheels, publishes them to PyPI, and creates a GitHub release with the same wheels and tag notes. If publishing succeeds but the GitHub release step fails, rerun only the failed job. Don't move a published tag or try to replace a published wheel; cut a new version if you need to change the artifacts.

Verify `uv tool install make-mate` in a clean environment on the supported platforms. After the first release, make that the headline install command in `README.md` before announcing.
