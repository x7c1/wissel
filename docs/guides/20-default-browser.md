# Default Browser Registration

## Overview

wissel does its job only while it is the default web browser: that is what
makes the desktop hand every link to it. This guide explains how that
registration works on Ubuntu (GNOME), how to set it up while developing, and
how to undo it.

## How the desktop picks a browser

Two independent layers decide what opens a link.

1. **Candidates.** Every `.desktop` file declares in `MimeType=` which content
   it can open. A web browser claims `x-scheme-handler/http` and
   `x-scheme-handler/https`. The desktop indexes these claims in
   `mimeinfo.cache`, one per `applications/` directory, rebuilt by
   `update-desktop-database`. Being a candidate is what makes an app appear in
   Settings → Apps → Default Apps → Web.
2. **Default.** `~/.config/mimeapps.list` records, per MIME type, which
   candidate is the default. It is per user; a package cannot and should not
   set it. Lower-priority layers (`/etc/xdg/mimeapps.list`, the distribution's
   `ubuntu-mimeapps.list`) apply only when the user has not chosen.

Applications resolve the link through GIO or `xdg-open`, which read both
layers. Sandboxed apps (snap, Flatpak) go through `xdg-desktop-portal`, which
resolves on the host the same way.

`data/wissel.desktop` is the entry wissel ships. `Exec=wissel %u` receives the
URL as the first argument.

## Development setup

The debug binary is not on `PATH`, so the development entry points at its
absolute path. `make dev-install` builds, writes the entry to
`~/.local/share/applications/wissel.desktop` with that path, and refreshes the
index.

```bash
make dev-install
gio mime x-scheme-handler/https   # wissel.desktop appears under "Recommended applications"
```

This only makes wissel a candidate. Make it the default with either of these;
they are equivalent and both write `~/.config/mimeapps.list`:

- Settings → Apps → Default Apps → Web → choose **wissel**
- `xdg-settings set default-web-browser wissel.desktop`

Note the previous default first so you can restore it:

```bash
xdg-settings get default-web-browser
```

Then open a link. From a terminal, wissel inherits the terminal and prints
the URL there:

```bash
xdg-open https://example.com
```

From a GUI application the launch goes through the desktop (or the portal, for
sandboxed apps), and the output lands in the user journal under the
identifier `wissel.desktop`:

```bash
gdbus call --session --dest org.freedesktop.portal.Desktop \
  --object-path /org/freedesktop/portal/desktop \
  --method org.freedesktop.portal.OpenURI.OpenURI "" "https://example.com" "{}"
journalctl --user -t wissel.desktop -n 5
```

## Undo

Restore the previous default, then remove the entry:

```bash
xdg-settings set default-web-browser <previous>.desktop
make dev-uninstall
```

Removing the entry while wissel is still the default does not break link
opening: GIO skips candidates that no longer exist and falls back to the next
one. The stale line in `mimeapps.list` stays until the user picks another
browser.

## Not covered

- Debian's `update-alternatives --config x-www-browser` is a separate,
  terminal-oriented mechanism (`sensible-browser`). GNOME does not consult it.
- wissel does not yet offer to make itself the default from its own UI.
