#!/bin/sh
set -eu

usage() {
	echo 'release: provide a version as X.Y.Z (mate release X.Y.Z or make release VERSION=X.Y.Z)' >&2
	exit 1
}

version=${1:-}
case "$version" in
*[!0-9.]*) usage ;;
esac
printf '%s\n' "$version" | grep -Eq '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$' || usage
tag="v$version"
root=$(git rev-parse --show-toplevel)
cd "$root"
if [ "$(git branch --show-current)" != main ]; then
	echo 'release: run this command from main' >&2
	exit 1
fi
git fetch --tags origin
if ! git merge-base --is-ancestor origin/main main; then
	echo 'release: main has not caught up with origin/main; update it first' >&2
	exit 1
fi
if git show-ref --verify --quiet "refs/tags/$tag"; then
	echo "release: $tag already exists" >&2
	exit 1
fi
command -v gh >/dev/null || {
	echo 'release: install the GitHub CLI (gh)' >&2
	exit 1
}

scratch=$(mktemp -d)
worktree="$scratch/worktree"
trap 'cd "$root"; if [ -d "$worktree" ]; then git worktree remove --force "$worktree"; fi; rmdir "$scratch"' EXIT
git worktree add --quiet --detach "$worktree" main
cd "$worktree"
if [ ! -f .github/workflows/release.yml ]; then
	echo 'release: commit .github/workflows/release.yml on main before releasing' >&2
	exit 1
fi

python3 - "$version" <<'PY'
import re
import sys
import tomllib
from pathlib import Path

version = sys.argv[1]
files = [(Path("Cargo.toml"), "package"), (Path("pyproject.toml"), "project")]
versions = [tomllib.loads(path.read_text())[section]["version"] for path, section in files]
if versions[0] != versions[1]:
    raise SystemExit("release: Cargo and Python versions differ; reconcile them first")
current = versions[0]
if tuple(map(int, version.split("."))) < tuple(map(int, current.split("."))):
    raise SystemExit(f"release: {version} is older than the current version {current}")
if version != current:
    for path, section in files:
        text = path.read_text()
        start = text.index(f"[{section}]\n")
        end = text.find("\n[", start + 1)
        if end == -1:
            end = len(text)
        updated, count = re.subn(
            r'^version\s*=\s*"[^"\n]+"$', f'version = "{version}"', text[start:end], flags=re.M
        )
        if count != 1:
            raise SystemExit(f"release: expected one version line in {path} [{section}]")
        path.write_text(text[:start] + updated + text[end:])
PY

# cargo check also refreshes Cargo.lock if the package version changed.
make check
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
if ! git diff --quiet -- Cargo.toml Cargo.lock pyproject.toml; then
	git add Cargo.toml Cargo.lock pyproject.toml
	git commit -m "Release $version"
fi
commit=$(git rev-parse HEAD)
git tag -a "$tag"
if ! git push --atomic origin HEAD:refs/heads/main "refs/tags/$tag"; then
	echo "release: push failed; check origin before deleting or retrying local tag $tag" >&2
	exit 1
fi

cd "$root"
if [ "$(git branch --show-current)" != main ] || ! git merge --ff-only "$tag"; then
	echo "release: couldn't fast-forward local main; reconcile it with origin/main separately" >&2
fi

# GitHub can take a moment to register the workflow for a freshly pushed tag.
attempt=0
while [ "$attempt" -lt 30 ]; do
	run_id=$(gh run list --workflow release.yml --event push --branch "$tag" --commit "$commit" \
		--json databaseId --jq '.[0].databaseId // empty')
	if [ -n "$run_id" ]; then
		gh run watch "$run_id" --exit-status
		exit
	fi
	attempt=$((attempt + 1))
	sleep 2
done
echo "release: $tag was pushed, but no release workflow run appeared; check GitHub Actions" >&2
exit 1
