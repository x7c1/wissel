---
status: completed
pipeline_phase: null
plan: null
base_ref: null
perspectives: [completeness, clarity, rust-module-structure]
max_refine_rounds: 3
retries_remaining: 1
check_command: "make check && grep -rqE 'set_current_dir' crates/wissel/src/ && grep -rqE 'rmdir|remove_dir' crates/wissel/tests/"
assignee: null
branch: task/1121-0301-launch-from-home-when-cwd-is-gone
created_at: 2026-11-20T18:01:49Z
updated_at: 2026-11-20T18:07:51Z
---

# fix: launch the browser from $HOME when the working directory is gone

## Overview

wissel inherits the working directory of the process that opened the link
(for example a terminal), and GIO passes that working directory on to the
browser it launches. When that directory has been deleted, Google Chrome
cannot hand the URL to its running instance and also aborts its own start on
`SingletonLock`, so the link silently does not open. Chrome started directly
from a deleted directory behaves the same, and opens normally from a valid
one, so wissel has to give the browser a working directory that exists.

At the start of `main` in `crates/wissel/src/main.rs`, before GTK or GIO is
used and before any of the subcommands runs, check whether the current
working directory can be obtained (`std::env::current_dir()`). When it can,
leave it unchanged. When it cannot, change to `$HOME`
(`std::env::set_current_dir`); if `HOME` is unset, empty, or the change to it
fails, change to `/`. This covers both the picker path (`wissel <url>...`) and
`wissel --open`, since both launch through `browsers::launch`. Put the logic
in a small function of its own (its own module under `crates/wissel/src/`,
following the existing one-public-item-per-file layout) rather than inline in
`main`. If even `/` fails, carry on in the current state rather than exit:
the launch may still work for browsers that do not care. Do not print
anything on stdout; a change of directory is not an error worth a message on
stderr either.

Document the behavior in the module doc comment of `main.rs` (one sentence
saying the browser is started from `$HOME`, or `/`, when wissel's working
directory no longer exists).

Tests go in `crates/wissel/tests/cli.rs`. Add a fixture entry under
`crates/wissel/tests/fixtures/applications/` whose `Exec` writes its working
directory (`pwd -P` or `$PWD` resolved) to the file named by `WISSEL_TEST_OUT`
(writing to a temporary name and renaming, as `recorder.desktop` does). Mark
it `NoDisplay=true` and register it in `mimeinfo.cache` like `hidden.desktop`
so `--list` output, and the existing list test, stay unchanged; `--open`
resolves it by id regardless. A child process cannot be spawned with a
working directory that no longer exists, so run wissel through `/bin/sh -c`
that creates a temporary directory, `cd`s into it, removes it (`rmdir`), and
then `exec`s the wissel binary with `--open <fixture id> <url>`. Cover:

- working directory deleted, `HOME` set to an existing temporary directory →
  the browser runs in that `HOME` directory;
- working directory deleted, `HOME` set to a path that does not exist →
  the browser runs in `/`;
- working directory valid → the browser runs in that directory (unchanged).

Compare paths after canonicalizing the expected directories (the temp dir
may sit behind a symlink).

Out of scope: a visible error dialog, and any change to how the URL is
passed to the browser.

## Acceptance criteria

### Automated (pipeline-verified)

- [x] `wissel --open <id> <url>` run from a deleted working directory with
      `HOME` pointing at an existing directory launches the entry with that
      directory as its working directory (test in `crates/wissel/tests/cli.rs`
      that removes the directory with `rmdir`; `make check`)
- [x] With the working directory deleted and `HOME` pointing at a missing
      path, the entry runs in `/` (test; `make check`)
- [x] With a valid working directory, the entry runs in that directory, so
      wissel does not change a working directory that exists (test;
      `make check`)
- [x] The fallback is implemented with `std::env::set_current_dir` under
      `crates/wissel/src/` (grep in `check_command`), and `--list` output is
      unchanged (the existing list test passes)
