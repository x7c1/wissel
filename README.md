# wissel

Pick a browser every time you open a link.

## Features

- **Choose on click** — wissel registers as the default web browser. When any application opens a link, it shows the browsers installed on your system and launches the one you pick.
- **Keyboard first** — pick a browser with a single key, so the extra step costs nothing.
- **Nothing to configure** — installed browsers are discovered from their desktop entries.

## Status

Pre-0.1. Set wissel as the default browser and every link opens in the browser you pick. Releases ship a Debian package for Ubuntu 26.04 or later, the oldest supported release; the first target is Ubuntu (GNOME) on Wayland.

## Install

Download `wissel_<version>-1_amd64.deb` from the
[latest GitHub Release](https://github.com/x7c1/wissel/releases/latest) and
install it on Ubuntu 26.04 or later:

```bash
sudo apt install ./wissel_<version>-1_amd64.deb
```

Then choose Wissel under Settings → Apps → Default Apps → Web.

## Getting started

Building needs Rust, `pkg-config`, and the GTK 4 and libadwaita development
files (`sudo apt install pkg-config libgtk-4-dev libadwaita-1-dev` on Ubuntu).

```bash
git clone https://github.com/x7c1/wissel.git
cd wissel
make dev-install   # builds and registers the debug binary as a browser candidate
```

This installs `data/io.github.x7c1.wissel.desktop` for the current user.

Then choose wissel under Settings → Apps → Default Apps → Web.

See [docs/guides/20-default-browser.md](docs/guides/20-default-browser.md)
for how this works and how to undo it.

## Documentation

- [docs/guides/](docs/guides/) — development guides, including
  [how releases are cut](docs/guides/30-release.md)

## License

GPL-3.0. See [LICENSE](LICENSE).
