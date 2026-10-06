# Release

## Overview

How to cut a wissel release. Releases are driven by a single rolling pull
request that a bot keeps in sync with `main`; cutting a release is just
merging that PR.

wissel uses a "merge the PR" release model. A bot opens and updates one
rolling release PR against `main`; merging it triggers the `Release`
workflow, which creates the matching `vX.Y.Z` tag and a GitHub Release. The
Release body is a summary written by hand in the release PR; the generated
per-commit changelog stays on the PR. Once the Release exists, the Debian
package is built and attached to it (see [The package](#the-package)).

## Normal flow (patch bump)

1. The `Create Release PR` workflow opens or refreshes a single open PR
   titled `Release vX.Y.Z` on every push to `main`. By default `X.Y.Z` is
   the last tag patch-bumped (e.g. `v0.1.0` → `Release v0.1.1`). Before the
   first release there is no tag, so the default is `Release v0.0.1`.
2. The PR body has two parts: a summary region you write by hand, and the
   changelog since the previous tag below it (auto-generated from `git log`
   via `generate-changelog.sh`). See [Release summary](#release-summary).
3. Write the summary. The PR cannot be merged while it is unwritten.
4. Merge the PR when you want to cut the release. The `Release` workflow
   then runs after CI completes on `main`, creates the `vX.Y.Z` tag, and
   publishes a GitHub Release carrying that summary plus links back to the
   release PR and the compare view.

## Release summary

The release PR body is split in two by a marker line:

```markdown
## Summary

<!-- release-summary:todo -->
_Write the release summary here. It becomes the body of the GitHub Release.
Delete the marker comment above once written._

<!-- changelog:auto -->

## Features

- feat: ...
```

- Everything **above** `<!-- changelog:auto -->` is yours. The bot carries it
  through verbatim every time it regenerates the body, so it survives the
  force-pushes that rebuild the release branch on each push to `main`.
- Everything from the marker down is regenerated from `git log` on every push
  to `main`; edits made there are overwritten.

Because the summary is the body of the GitHub Release, the release is gated on
it: `Validate Release PR` fails while the summary is empty or still carries the
`<!-- release-summary:todo -->` sentinel. That workflow also runs on body
edits, so saving the summary re-runs the check and turns it green without any
further push. If a release nonetheless reaches the `Release` workflow without
a summary, that workflow fails before creating the tag rather than publishing
a Release without one.

## Promoting to minor or major

To cut a minor or major release, **edit the PR title only**:

- `Release v0.1.1` → `Release v0.2.0` (minor)
- `Release v0.1.1` → `Release v1.0.0` (major)

Saving the title edit triggers the bot immediately — you do not have
to wait for the next push to `main`. (The next push works too, so either
path gets you there.) Body-only edits and no-op title saves are ignored,
so re-opening the edit dialog without changing the title does not
retrigger the bot. When the bot does run, it:

1. Reads the new title and extracts the target version.
2. Runs `cargo set-version -p wissel <version>` and `cargo update --workspace`
   to update `crates/wissel/Cargo.toml` and `Cargo.lock`.
3. Force-pushes the result to the existing release branch.
4. Updates the PR body's compare link.

The title is the single source of truth for the next version; you never
edit `Cargo.toml` by hand.

Until the first release, `crates/wissel/Cargo.toml` carries `0.0.0`, the
same baseline the validator uses when there is no tag. Every release title
therefore names a version different from the one on `main`, which is what
lets the bot commit the bump and the `Release` workflow see a version change.

## Allowed title transitions

`validate-release-pr-title.sh` runs as a required check on every release PR
edit and enforces a strict semver progression against `last_tag`. Only
single-step bumps with the lower components reset are allowed:

| Last tag | PR title | Result |
|---|---|---|
| v0.3.10 | `Release v0.3.11` | ✅ patch |
| v0.3.10 | `Release v0.4.0` | ✅ minor |
| v0.3.10 | `Release v1.0.0` | ✅ major |
| v0.3.10 | `Release v0.3.9` | ❌ downgrade |
| v0.3.10 | `Release v0.4.1` | ❌ minor bump with non-zero patch |
| v0.3.10 | `Release v0.5.0` | ❌ minor skip |
| v0.3.10 | `Release v2.0.0` | ❌ major skip |
| v0.3.10 | `Release 0.3.11` | ❌ malformed title |

With no tag yet, `v0.0.0` is the baseline, so the first release may be
`v0.0.1`, `v0.1.0` or `v1.0.0`. `validate-release-pr-title.test.sh` covers
each row and that baseline; `make check` runs it.

The PR cannot be merged until the title satisfies this validator (branch
protection requires the check).

## Branch naming

The release branch is named `release/since-<UTC %Y-%m-%d-%H%M>` (e.g.
`release/since-2026-10-06-1431`). The `since-` prefix labels it as the time
the branch was opened, not the release time.

The branch name is intentionally decoupled from the version it carries —
the version lives in the PR title and `Cargo.toml`, while the branch name
is fixed for the lifetime of the PR. Promoting the title from patch to
minor or major keeps reusing the same branch, so the PR's head pointer
stays in sync with every force-push.

## The package

Every Release carries one unsigned Debian package for x86_64,
`wissel_<version>-1_amd64.deb`, built by `cargo deb` (cargo-deb) from the
`[package.metadata.deb]` table in `crates/wissel/Cargo.toml`.

It installs the binary to `/usr/bin/wissel` and the desktop entry to
`/usr/share/applications/io.github.x7c1.wissel.desktop`. Its dependencies are
derived from the binary's shared libraries (`depends = "$auto"`), so they name
`libgtk-4-1` and `libadwaita-1-0`. The version in the file name is the package
version, so it matches the tag by construction; `-1` is the Debian revision.

The package is built on `ubuntu-26.04`. Ubuntu 26.04 is the oldest release
wissel supports, so building there keeps the derived dependencies in step
with the libraries that release ships.

- **Order.** The `deb` job of the `Release` workflow runs only after the
  `release` job has created the tag and the Release, builds the tagged
  commit, and uploads to that Release with `gh release upload`. A package
  failure therefore never undoes the tag or the Release; it shows up as a
  failed `deb` job.
- **Rerunning.** Open the failed `Release` run in the Actions tab and use
  "Re-run failed jobs": only `deb` runs again, against the same tag and
  Release.
- **Pull requests.** The `deb` job of `CI` runs the same build on every pull
  request and push to `main` and uploads the package as the workflow artifact
  `wissel-deb`, so a broken package is caught before a release depends on it
  and reviewers can download and try it.

To build the package locally, install cargo-deb (`cargo install cargo-deb
--locked`) and run `cargo deb -p wissel`; the file lands in `target/debian/`.

## Workflows involved

- `.github/workflows/ci.yml` — runs `make check` and builds the package on
  every pull request to `main` and every push to `main`.
- `.github/workflows/create-release-pr.yml` — opens or updates the
  release PR on every push to `main` (skipped when the push is itself
  the merge of a release PR, to avoid looping). It also runs when a
  release-labelled PR's title is edited, so promoting the title is
  picked up immediately rather than waiting for the next main push.
- `.github/workflows/validate-release-pr.yml` — on every release PR edit,
  enforces the allowed title transitions above and fails while the release
  summary is unwritten.
- `.github/workflows/release.yml` — when CI completes successfully on
  `main`, checks whether the package version changed; if it did, reads the
  summary from the merged release PR and then creates the matching tag and
  GitHub Release; its `deb` job then builds the package and attaches it. See
  [The package](#the-package).

For the underlying setup (the `RELEASE_PAT` secret, why a user PAT is
required instead of `GITHUB_TOKEN`), see
[Release automation setup](#release-automation-setup) below.

## Recovery

- A failed `Create Release PR` run can be re-run from the Actions tab; the
  script is idempotent (it rebuilds the release branch from `origin/main`
  on every invocation).
- If you need to abandon the current release PR for any reason, close it
  and delete the branch — the next push to `main` opens a fresh PR with
  the default patch-bumped title.
- The `Release` workflow gates on a green CI: a red `main` will never tag
  a release. Fix CI first and the next successful CI run cuts the
  release automatically.
- If `Release` failed because the summary was missing, no tag was created.
  Write the summary into the merged release PR's body (a merged PR's
  description is still editable) and re-run the failed run from the Actions
  tab — before another commit lands on `main`. The version-change check
  compares the package version against the previous commit, so once the
  bump is no longer the newest change the automation stops seeing a version
  change and will not cut that release on its own.
- If only the `deb` job failed, the tag and the Release exist; re-run the
  failed job as described under [The package](#the-package).

## Release automation setup

The flow above is developer-facing; this section covers the supporting
setup: how the release PR is opened under a user PAT and why that is
required.

The `Create Release PR` workflow (`.github/workflows/create-release-pr.yml`)
opens and updates the rolling release PR. It runs on two triggers: every push
to `main`, and `pull_request: types: [edited]` so that promoting a release
PR's title (e.g. `Release v0.1.1` → `Release v0.2.0`) is picked up immediately
rather than waiting for the next main push. The `pull_request` branch is
gated on an **open** PR carrying the `release` label whose **title actually
changed** (`changes.title.from != null`), so body-only edits, no-op title
saves, and bot self-edits do not retrigger the workflow. It pushes a
`release/since-<UTC date+time>` branch (chosen once when the PR is opened and
reused for the lifetime of that PR) and calls `gh pr create` under a
**user-scoped personal access token**, exposed to the workflow as the
repository secret **`RELEASE_PAT`**.

A user-scoped token is required because GitHub's recursion-prevention rule
suppresses `pull_request` workflow runs on PRs authored by `github-actions[bot]`.
With the default `GITHUB_TOKEN` the release PR would be bot-authored, so `CI`
and `Validate Release PR` would sit in `action_required` and never go green.
Pushing and opening the PR under a user PAT makes the PR user-authored, which
lets the existing checks trigger normally.

**Required setup (one-time, per repo).** A maintainer must register
`RELEASE_PAT` in the repository's Actions secrets with these scopes:

- `contents: write` — push the `release/since-<UTC date+time>` branch.
- `pull-requests: write` — create and edit the release PR.

Without the secret, the workflow fails loudly at the `git push` step. There is
no fallback to `GITHUB_TOKEN` by design: a silent fallback would mask exactly
the misconfiguration the PAT is solving.

The `Release` workflow (`.github/workflows/release.yml`) that tags and publishes
after the release PR is merged keeps using the default `GITHUB_TOKEN` — it runs
under a human-triggered merge event, so recursion is not a concern, and no
workflow in this repo listens to tag pushes.
