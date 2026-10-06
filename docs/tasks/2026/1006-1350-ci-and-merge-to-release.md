---
status: completed
pipeline_phase: null
plan: null
base_ref: null
perspectives: [completeness, clarity]
max_refine_rounds: 3
retries_remaining: 1
check_command: "make check && test -f .github/workflows/ci.yml && test -f .github/workflows/create-release-pr.yml && test -f .github/workflows/validate-release-pr.yml && test -f .github/workflows/release.yml && grep -q 'package.metadata.deb' crates/wissel/Cargo.toml && test -f docs/guides/30-release.md"
assignee: null
branch: task/1006-1350-ci-and-merge-to-release
created_at: 2026-10-06T04:50:05Z
updated_at: 2026-10-06T05:58:00Z
---

# ci: add CI and the merge-to-release workflow with a .deb package

## Overview

wissel has no CI and no release process. Add both, following the
"merge the release PR" model: a bot keeps one rolling `Release vX.Y.Z` pull
request in sync with `main`; merging it tags the release and publishes a
GitHub Release with the `.deb` attached. A maintainer never edits the
version by hand or runs a release script.

The model and its scripts already exist in another project by the same
author; port them rather than redesigning. Copy its release scripts, the
version-change check, the three release workflows and the release guide
into this repository, keep their behaviour and their comments, and adapt
only what differs here. This repository's documentation stands on its own
and does not refer to the other project.

What differs here:

- The version lives in `crates/wissel/Cargo.toml` under `[package]`, not
  in a workspace table. Read it with `cargo metadata --no-deps` selecting
  the `wissel` package, and bump it with `cargo set-version -p wissel
  <version>` (cargo-edit) followed by `cargo update --workspace` so
  `Cargo.lock` follows. The release commit adds `crates/wissel/Cargo.toml`
  and `Cargo.lock`.
- There is no desktop-shell bundle. The release artifact is a Debian
  package built with `cargo deb` (cargo-deb). Add a `[package.metadata.deb]`
  table to `crates/wissel/Cargo.toml` with the maintainer
  `aida.t <994424+x7c1@users.noreply.github.com>`, section `web`, priority
  `optional`, `depends = "$auto"`, an extended description, and assets that
  install `target/release/wissel` to `usr/bin/` (mode 755) and
  `data/io.github.x7c1.wissel.desktop` to `usr/share/applications/`
  (mode 644). The package name is `wissel`.
- Builds need the GTK 4 and libadwaita development packages. Every job that
  compiles runs on `ubuntu-26.04` and installs `libgtk-4-dev
  libadwaita-1-dev` first. Ubuntu 26.04 is the oldest supported release,
  because it is the oldest the author can verify on; say so in the README.

Add a `check` target to the Makefile that runs the canonical gate through
`$(CARGO)`: `fmt --check`, `build --locked`, `test --locked`, `clippy
--all-targets -- -D warnings`, then `bash .github/scripts/release-summary.test.sh`,
`bash .github/scripts/validate-release-pr-title.test.sh`, and `bash -n` over
every script under `.github/scripts/` and `scripts/`. CI runs `make check`,
so the gate has one definition.

Write `.github/scripts/validate-release-pr-title.test.sh` in the style of
`release-summary.test.sh`: one case per row of the allowed-transitions table
in the release guide (patch, minor, major accepted; downgrade, minor bump
with non-zero patch, minor skip, major skip, malformed title rejected) plus
the no-tag baseline (`v0.0.0`), asserting on the exit code.

Workflows:

- `.github/workflows/ci.yml`, named `CI`: on pull requests to `main` and
  pushes to `main`. A `check` job (toolchain with clippy and rustfmt, cargo
  cache keyed on `Cargo.lock`, `make check`) and a `deb` job that runs
  `cargo deb` and uploads the resulting package as a workflow artifact, so
  a broken package is caught before a release depends on it.
- `.github/workflows/create-release-pr.yml`, `validate-release-pr.yml` and
  `release.yml` as in the reference, with the version handling above. In
  `release.yml`, replace the bundle job with a `deb` job that runs after the
  `release` job on `ubuntu-26.04`, builds with `cargo deb`, and attaches the
  file with `gh release upload v<version> <file>`; a package failure must not
  undo the tag or the Release.

Documentation:

- `docs/guides/30-release.md`: the release guide, adapted from the reference
  guide. Keep its sections (overview, normal flow, release summary, promoting
  to minor or major, allowed title transitions, branch naming, the package,
  workflows involved, recovery, release automation setup including the
  `RELEASE_PAT` secret and why `GITHUB_TOKEN` is not enough). Describe the
  `.deb` instead of desktop bundles.
- `README.md`: an "Install" section before "Getting started": download
  `wissel_<version>-1_amd64.deb` from the latest GitHub Release and install
  it with `sudo apt install ./wissel_<version>-1_amd64.deb` on Ubuntu 26.04
  or later; then choose Wissel under Settings → Apps → Default Apps → Web.
  Keep the existing status line consistent with that.

## Acceptance criteria

### Automated (pipeline-verified)

- [x] `make check` runs the canonical gate and both script test files, and
  passes.
- [x] `validate-release-pr-title.test.sh` covers every row of the allowed
  transitions table and the no-tag baseline.
- [x] The four workflow files exist, `Cargo.toml` carries
  `[package.metadata.deb]`, and `docs/guides/30-release.md` exists (gates in
  `check_command`).

### Before merge (verified outside the check command)

- [x] On the development machine, `cargo deb` produces
  `target/debian/wissel_<version>-1_amd64.deb`; `dpkg-deb -c` lists
  `./usr/bin/wissel` and `./usr/share/applications/io.github.x7c1.wissel.desktop`,
  and `dpkg-deb -f ... Depends` names `libgtk-4-1` and `libadwaita-1-0`.
- [ ] Needs a person: `sudo apt install ./wissel_<version>-1_amd64.deb` on
  the development machine installs cleanly, `wissel --list` runs, and Wissel
  appears under Settings → Apps → Default Apps → Web.
- [ ] After this PR merges with the `RELEASE_PAT` secret in place, the
  `Create Release PR` workflow opens a `Release v…` pull request whose body
  carries the summary placeholder and the changelog.

## Out of scope

- Publishing to a PPA or to Flathub.
- Signing the package.
- Branch protection settings (configured separately once the checks exist).
