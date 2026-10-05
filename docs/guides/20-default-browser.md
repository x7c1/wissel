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

`data/io.github.x7c1.wissel.desktop` is the entry wissel ships. `Exec=wissel %u`
receives the URL as the first argument. The file name matches wissel's
application id, `io.github.x7c1.wissel`: on Wayland, GNOME links a window to
its desktop entry by that match, which gives the picker its name and icon in
the shell.

## Development setup

The debug binary is not on `PATH`, so the development entry points at its
absolute path. `make dev-install` builds, writes the entry to
`~/.local/share/applications/io.github.x7c1.wissel.desktop` with that path, and
refreshes the index.

```bash
make dev-install
# io.github.x7c1.wissel.desktop appears under "Recommended applications"
gio mime x-scheme-handler/https
```

This only makes wissel a candidate. Note the current default so you can
restore it later, then make wissel the default with either of these; both
write `~/.config/mimeapps.list`:

```bash
xdg-settings get default-web-browser
```

- Settings → Apps → Default Apps → Web → choose **wissel**
- `xdg-settings set default-web-browser io.github.x7c1.wissel.desktop`

To check, open a link. wissel shows its picker, and the link opens in the
browser you choose:

```bash
xdg-open https://example.com
```

If the browser cannot be launched, wissel prints the reason on stderr. From a
GUI application, that output lands in the user journal under the identifier
`io.github.x7c1.wissel.desktop`:

```bash
journalctl --user -t io.github.x7c1.wissel.desktop -n 5
```

## Undo

Restore the previous default, then remove the entry:

```bash
xdg-settings set default-web-browser <previous>.desktop
make dev-uninstall
```

Development entries installed before the desktop entry was renamed are called
`wissel.desktop`. While one is installed, the picker lists it as a browser
named wissel. `make dev-uninstall` does not remove them; if the default
browser is still `wissel.desktop`, restore the previous default first, then:

```bash
rm ~/.local/share/applications/wissel.desktop
update-desktop-database ~/.local/share/applications
```
