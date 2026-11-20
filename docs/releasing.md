# Releasing make-mate

[GitHub CI](../.github/workflows/ci.yml) runs Rust checks and builds and installs release wheels on Linux x86-64 (glibc 2.28+) and Apple Silicon macOS. Pushes to `main` and pull requests do not publish anything.

## One-time setup

1. In GitHub, create an Actions environment named `pypi`. Restrict deployments to release tags (`v*`) and require a reviewer if available.
2. In [PyPI's publishing settings](https://pypi.org/manage/account/publishing/), add a pending [Trusted Publisher](https://docs.pypi.org/trusted-publishers/creating-a-project-through-oidc/) for the first release:

   | Field | Value |
   | --- | --- |
   | PyPI project name | `make-mate` |
   | Owner | `Singletoned` |
   | Repository | `make-mate` |
   | Workflow filename | `ci.yml` |
   | Environment name | `pypi` |

   No PyPI API token is needed. If the project already exists under your account, add the publisher to the project instead.

## Cut a release

1. Ensure the versions in `Cargo.toml` and `pyproject.toml` match. If bumping the version, run `cargo check` to refresh `Cargo.lock` and commit all three files.
2. Push the release commit to `main` and wait for CI to pass on both platforms.
3. Create an annotated tag on that commit. Write the release notes in the tag message; those become the GitHub release notes. For example, for the first alpha:

   ```sh
   git tag -a v0.1.15
   git push origin v0.1.15
   ```

   Suggested notes: “First public alpha: list Makefile tasks, complete them in Bash, zsh, and fish, and pass positional arguments without giving up plain `make`. Wheels are available for Linux x86-64 and Apple Silicon macOS.”

4. The tag run rebuilds and tests both wheels, publishes them to PyPI, and then creates a GitHub release with the same wheels attached.
5. Verify `uv tool install make-mate` in a clean environment on the supported platforms. After the first release, make that the headline install command in `README.md` before announcing.

If publishing succeeds but the GitHub release step fails, rerun only the failed job. Don't move a published tag or try to replace a published wheel; cut a new version if you need to change the artifacts.
