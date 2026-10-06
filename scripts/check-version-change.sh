#!/usr/bin/env bash
set -euo pipefail

# Detect whether the package version in crates/wissel/Cargo.toml changed
# between HEAD~1 and HEAD, and report the result via $GITHUB_OUTPUT so a
# workflow step can branch on it.
#
# Outputs (to $GITHUB_OUTPUT, when set):
#   changed=true|false
#   version=<semver>      — only when changed=true; the new version.
#
# Implementation notes:
#   - The version is read through `cargo metadata` rather than by grepping
#     TOML, so the `[package]` section boundary is honoured by cargo
#     itself.
#   - The previous version is obtained by temporarily checking out
#     crates/wissel/Cargo.toml from HEAD~1 and running cargo metadata
#     against that snapshot, then restoring the working-tree copy. cargo metadata
#     only reads Cargo.toml + Cargo.lock; no network access is required.

REPO_ROOT="$(git rev-parse --show-toplevel)"
CARGO_TOML="${REPO_ROOT}/crates/wissel/Cargo.toml"

read_package_version() {
    # The workspace takes every crate under crates/, so select wissel
    # explicitly: its version is the one the release tag carries, and a
    # later member must not change which version is read.
    (cd "${REPO_ROOT}" && \
        cargo metadata --no-deps --format-version 1 \
        | jq -r '.packages[] | select(.name == "wissel") | .version')
}

current_version=$(read_package_version)

# Stash the working-tree Cargo.toml so we can restore it after reading
# the HEAD~1 snapshot.
backup=$(mktemp)
cp "$CARGO_TOML" "$backup"
trap 'cp "$backup" "$CARGO_TOML"; rm -f "$backup"' EXIT

git show HEAD~1:crates/wissel/Cargo.toml > "$CARGO_TOML"
previous_version=$(read_package_version)

echo "Current version: ${current_version}"
echo "Previous version: ${previous_version}"

if [ -z "${GITHUB_OUTPUT:-}" ]; then
    GITHUB_OUTPUT=/dev/null
fi

if [ "$current_version" != "$previous_version" ]; then
    echo "Version changed from ${previous_version} to ${current_version}"
    {
        echo "changed=true"
        echo "version=${current_version}"
    } >> "$GITHUB_OUTPUT"
else
    echo "Version unchanged; skipping release"
    echo "changed=false" >> "$GITHUB_OUTPUT"
fi
