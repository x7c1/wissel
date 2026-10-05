---
status: completed
pipeline_phase: null
plan: null
base_ref: null
perspectives: [completeness, clarity, rust-module-structure, user-experience]
max_refine_rounds: 3
retries_remaining: 1
check_command: "cargo fmt --check && cargo build && cargo test && cargo clippy --all-targets -- -D warnings && test -f data/io.github.x7c1.wissel.desktop && ! test -e data/wissel.desktop && grep -rq 'io.github.x7c1.wissel.desktop' crates/wissel/src/ && grep -q 'io.github.x7c1.wissel' Makefile"
assignee: null
branch: task/1006-0400-browser-picker-window
created_at: 2026-10-05T19:00:42Z
updated_at: 2026-10-05T19:44:00Z
---

# feat: show a picker window and print the chosen browser

## Overview

`wissel <url>` currently echoes the URL. Replace that with the picker: a
window that lists the installed browsers (from `browsers::installed()`) and
lets the user choose one with the mouse or the keyboard. For now the choice is
reported on stdout as the chosen browser's desktop id on its own line, and the
process exits 0; launching the browser is the next task and will consume that
choice. Closing the window or pressing Escape exits 1 with nothing on stdout.
`wissel --list` and the no-argument usage message are unchanged.

Build the window with GTK4 and libadwaita through the `gtk4` and
`libadwaita` crates (gtk-rs). Pick the current releases and move the existing
`gio` dependency to the version those crates re-export, so only one `gio`
is in the tree. GTK 4.22 and libadwaita 1.9 are installed on the development
machine.

The window:

- An `adw::ApplicationWindow` sized for a short list, with the URL being
  opened shown at the top (the first argument; when several URLs are given,
  they all go to the same browser, so show the first and the count).
- One row per browser in `installed()` order, showing the icon (from
  `Browser::icon`, falling back to a generic web-browser icon), the display
  name, and a shortcut hint.
- Mouse: activating a row chooses it.
- Keyboard: the rows are focusable, Up/Down moves, Enter chooses the focused
  row. Digits `1`-`9` choose the first nine rows directly without needing
  focus. Escape cancels.
- The first row is focused when the window opens, so Enter alone picks the
  first browser.

Keep the mapping between rows and shortcut keys in a module with no GTK
types (which browser gets which digit, and which row a key press selects),
with unit tests; the GTK code only renders it.

Give the application the id `io.github.x7c1.wissel` and rename the desktop
entry to match: `data/wissel.desktop` becomes
`data/io.github.x7c1.wissel.desktop`, the self-exclusion id in
`crates/wissel/src/browsers/` follows, and `Makefile`, `README.md` and
`docs/guides/20-default-browser.md` reference the new file name. On Wayland,
GNOME links a window to its desktop entry by matching the application id to
the entry's file name; with the old name the picker would show up in the
shell with a fallback name and icon.

Run the GTK application so that it does not parse the process arguments
itself (`run_with_args` with an empty slice, or equivalent): the URL
arguments are wissel's to interpret.

Tests: the integration test in `crates/wissel/tests/cli.rs` that expects
`wissel <url>` to echo the URL no longer holds and needs a display to run the
new behaviour, so remove that case; keep the `--list` and no-argument cases.
The key-mapping module carries the unit tests.

## Acceptance criteria

### Automated (pipeline-verified)

- [x] The key-mapping module assigns `1`-`9` to the first nine browsers and
  no digit to the rest, and resolves a digit to the matching index or to
  nothing, covered by unit tests.
- [x] `wissel --list` output and the no-argument exit code are unchanged
  (integration test).
- [x] The crate depends on `gtk4` and `libadwaita`, and `Cargo.lock`
  contains a single `gio` version.
- [x] The desktop entry is `data/io.github.x7c1.wissel.desktop`, the old
  file is gone, and the source and `Makefile` reference the new id (gates in
  `check_command`).

### Before merge (verified outside the check command)

- [x] On the development machine, `cargo run -- https://example.com` opens a
  window that shows the URL and lists the same browsers as `--list`, each
  with its icon, name and shortcut hint, with the first row focused.
- [x] Clicking a row prints that row's desktop id and exits 0; pressing its
  digit does the same; Escape exits 1 with nothing on stdout.
- [x] With `make dev-install` in effect and wissel set as the default
  browser, `xdg-open https://example.com` shows the picker, and the window
  appears in the shell as wissel. The previous default is restored
  afterwards.

## Out of scope

- Launching the chosen browser (next task).
- Remembering a choice, per-site rules, or any configuration.
- Packaging and CI.
