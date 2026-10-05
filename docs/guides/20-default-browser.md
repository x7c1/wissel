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
   candidate is the default. It is per-user state, so installing wissel never
   changes it.

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

This only makes wissel a candidate. Note the current default so you can
restore it later, then make wissel the default with either of these; both
write `~/.config/mimeapps.list`:

```bash
xdg-settings get default-web-browser
```

- Settings → Apps → Default Apps → Web → choose **wissel**
- `xdg-settings set default-web-browser wissel.desktop`

To check, open a link. From a terminal, wissel prints the URL there:

```bash
xdg-open https://example.com
```

From a GUI application, the output lands in the user journal under the
identifier `wissel.desktop`:

```bash
journalctl --user -t wissel.desktop -n 5
```

## Undo

Restore the previous default, then remove the entry:

```bash
xdg-settings set default-web-browser <previous>.desktop
make dev-uninstall
```
