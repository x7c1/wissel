---
status: completed
pipeline_phase: null
plan: null
base_ref: null
perspectives: [completeness, clarity, rust-module-structure, user-experience, error-type-design]
max_refine_rounds: 3
retries_remaining: 1
check_command: "cargo fmt --check && cargo build && cargo test && cargo clippy --all-targets -- -D warnings && ! grep -rqiE 'not (implemented )?yet|does not launch' crates/wissel/src/ docs/guides/ README.md"
assignee: null
branch: task/1006-0437-launch-chosen-browser
created_at: 2026-10-05T19:37:10Z
updated_at: 2026-10-05T20:28:00Z
---

# feat: open the URLs in the chosen browser

## Overview

The picker currently prints the chosen browser's desktop id on stdout. Make
wissel do its job instead: once the user picks a browser, launch that
browser's desktop entry with all the URLs wissel was given, then exit 0.
Nothing is printed on stdout on success. Cancelling still exits 1.

Launch through GIO: resolve the chosen id with `gio::DesktopAppInfo::new`
and call `launch_uris` with the URLs. Pass a launch context from the GTK
display (`gdk::Display::app_launch_context`) when the picker was shown, so
the browser receives an activation token and its window comes to the front
on Wayland instead of GNOME showing a "ready" notification; without a display
fall back to a plain `gio::AppLaunchContext`. GIO spawns the browser
detached (and moves it into its own systemd scope on a systemd session), so
wissel can exit right after the launch call returns. If the browser is
already running, its desktop entry hands the URL to the running instance and
the spawned process exits by itself; that is the browser's business.

Add a non-interactive form for scripting and for the tests:
`wissel --open <desktop-id> <url>...` launches that entry with the URLs
without showing the picker. It uses the same launch code as the picker path.

When a launch fails (the id does not resolve, or spawning fails), print
`wissel: could not open <url> with <name or id>: <error>` on stderr and exit
1. A visible error dialog is out of scope for this task.

Put the launch function next to the detection code under
`crates/wissel/src/browsers/`, keeping the existing one-public-item-per-file
layout. The picker's `pick` keeps returning the chosen `Browser`; `main`
wires pick and launch together.

Tests go in `crates/wissel/tests/cli.rs` against the committed fixture under
`crates/wissel/tests/fixtures/applications/`. Add a fixture entry whose
`Exec` writes its arguments to the file named by the environment variable
`WISSEL_TEST_OUT` (GIO passes wissel's environment to the child), for
example a `/bin/sh -c` command, and register it in `mimeinfo.cache` so it
lists as a browser. Mind the desktop-entry quoting rules for `Exec`
(double-quoted arguments, backslash-escaped `"`, `$` and `\`). The test
runs `wissel --open <that id> <url1> <url2>` as a child process with the
same XDG variables as the existing tests plus `WISSEL_TEST_OUT`, then
waits briefly for the file and asserts that it names both URLs in order.

Update the documentation that still says the chosen browser is not launched:
the crate doc in `main.rs`, `docs/guides/20-default-browser.md` (the
"To check" paragraph now says the link opens in the chosen browser; the note
about restoring the default right after checking is no longer needed) and
the README's status line, which can now describe wissel as usable.

## Acceptance criteria

### Automated (pipeline-verified)

- [x] `wissel --open <fixture id> <url1> <url2>` launches the fixture entry
  with both URLs in order and exits 0 with nothing on stdout (integration
  test reading `WISSEL_TEST_OUT`).
- [x] `wissel --open <unknown id> <url>` exits 1 and names the id in a
  message on stderr (integration test).
- [x] `wissel --list` output and the no-argument exit code are unchanged.
- [x] No source or documentation line still says launching is not
  implemented or that the chosen browser is not launched (grep gate in
  `check_command`).

### Before merge (verified outside the check command)

- [ ] On the development machine, `cargo run -- https://example.com/<unique>`
  and choosing Firefox, then again choosing Google Chrome, opens that URL in
  the chosen browser (its window title shows the page), and no wissel process
  is left running.
- [ ] The browser window that opens comes to the front, with no "is ready"
  notification from GNOME.
- [ ] With `make dev-install` in effect and wissel set as the default
  browser, `xdg-open https://example.com/<unique>` goes through the picker
  to the chosen browser. The previous default is restored afterwards.

## Out of scope

- Remembering a choice, per-site rules, or any configuration.
- An error dialog for launch failures.
- Packaging and CI.
