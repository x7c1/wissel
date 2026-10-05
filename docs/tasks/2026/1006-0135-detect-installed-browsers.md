---
status: completed
pipeline_phase: null
plan: null
base_ref: null
perspectives: [completeness, clarity, rust-module-structure]
max_refine_rounds: 3
retries_remaining: 1
check_command: "cargo fmt --check && cargo build && cargo test && cargo clippy --all-targets -- -D warnings"
assignee: null
branch: task/1006-0135-detect-installed-browsers
created_at: 2026-10-05T16:35:03Z
updated_at: 2026-10-05T17:08:00Z
---

# feat: detect installed browsers from desktop entries

## Overview

wissel will show the browsers installed on the system and launch the one the
user picks. This task adds the detection: a module that lists installed
browsers, and a `--list` flag on the binary that prints them so the result
can be checked from a terminal and from an integration test. The picker UI and
launching are later work.

Use GIO through the `gio` crate (gtk-rs). The project has decided on GTK4,
and GIO already implements the XDG lookup: it walks `XDG_DATA_HOME` and
`XDG_DATA_DIRS`, reads each directory's `mimeinfo.cache`, and honours
desktop-entry precedence. `gio-2.0` is available via pkg-config on the
development machine; GTK4 itself is not required for this task and must not be
added.

Add a module under `crates/wissel/src/` that exposes:

- `pub struct Browser { pub id: String, pub name: String, pub icon: Option<String> }`
  where `id` is the desktop file id (for example `firefox_firefox.desktop`),
  `name` is the display name, and `icon` is the icon's string form (a theme
  icon name or a file path, from `gio::Icon::to_string`).
- `pub fn installed() -> Vec<Browser>` built from
  `gio::AppInfo::all_for_type("x-scheme-handler/https")`, keeping only
  entries for which `should_show()` is true (this drops `NoDisplay=true`
  duplicates such as Chrome's `com.google.Chrome.desktop`), dropping wissel's
  own entry (`wissel.desktop`, the file name under `data/`), and sorted by
  `name` case-insensitively with `id` as the tie-breaker.

Extend `crates/wissel/src/main.rs`: when the only argument is `--list`,
print one line per browser as `<id>\t<name>\t<icon>` (empty third column when
there is no icon) and exit 0. Any other argument list keeps the current
behaviour (URLs are echoed; no arguments prints usage and exits 1). Hand-rolled
argument handling is fine; do not add a CLI crate for one flag.

Test it with an integration test in `crates/wissel/tests/` that spawns the
binary (`env!("CARGO_BIN_EXE_wissel")`) against a fixture directory committed
under `crates/wissel/tests/fixtures/`. GIO reads the XDG variables once per
process, so the test must run the binary as a child process with
`XDG_DATA_DIRS` pointing at the fixture root and `XDG_DATA_HOME`,
`XDG_CONFIG_HOME`, `XDG_CONFIG_DIRS` pointing at empty temporary
directories, so that the host's real entries and `mimeapps.list` never leak
in. Two GIO behaviours shape the fixture, both verified on the development
machine:

- GIO discards a desktop entry whose `Exec=` program is not found on
  `PATH`, so every fixture entry uses `Exec=/bin/true %u`.
- `all_for_type` returns nothing for a directory without `mimeinfo.cache`,
  so the fixture's `applications/` directory commits a hand-written
  `mimeinfo.cache` (`[MIME Cache]` section, one `<mime>=<id>;...` line per
  type) consistent with its entries.

The fixture covers: a browser with an icon, a browser without one whose name
sorts before the first only when compared case-insensitively, a
`NoDisplay=true` browser, a non-browser (`text/plain` only), and
`wissel.desktop` itself.

## Acceptance criteria

### Automated (pipeline-verified)

- [x] `wissel --list` against the fixture prints exactly the two visible
  browsers and nothing else: the `NoDisplay` entry, the non-browser, and
  `wissel.desktop` are absent.
- [x] The two lines are ordered by display name case-insensitively, and the
  icon column is the icon string for the entry that has one and empty for the
  other.
- [x] `wissel <url>` still prints the URL and exits 0, and `wissel` with no
  arguments still exits 1 (covered by the same integration test file).
- [x] `Cargo.toml` of the crate depends on `gio` and not on `gtk4` or
  `libadwaita`.

### Before merge (verified outside the check command)

- [x] On the development machine, `cargo run -- --list` shows Firefox,
  Google Chrome once, Vivaldi, Web (Epiphany) and the user's custom Firefox
  entry, and does not show wissel while `make dev-install` is in effect.

## Out of scope

- Launching the chosen browser and the picker UI.
- Any detection criterion other than handling `x-scheme-handler/https`
  (`Categories=WebBrowser` is deliberately not used: custom entries often
  omit it).
- Packaging and CI.
