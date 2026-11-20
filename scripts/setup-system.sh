#!/usr/bin/env bash
set -euo pipefail

# Install the system packages `make check` needs on Debian/Ubuntu.
#
# Usage: scripts/setup-system.sh   (from any working directory)
#
# Installs, with apt-get, the C toolchain, pkg-config and make, and the GTK 4
# and libadwaita development files wissel links against. Rust itself is not
# installed here; use rustup.
#
# CI runs this script too, so the package list has one definition.
#
# Runs apt-get through sudo unless already root. Safe to run again: when every
# package is already installed, apt-get is not run at all.

PACKAGES="build-essential make pkg-config libgtk-4-dev libadwaita-1-dev"

die() {
    printf 'setup-system: %s\n' "$1" >&2
    exit 1
}

command -v apt-get >/dev/null 2>&1 \
    || die "apt-get not found; this script supports Debian/Ubuntu only"

if [ "$(id -u)" = 0 ]; then
    as_root() { "$@"; }
else
    command -v sudo >/dev/null 2>&1 || die "not running as root and sudo not found; rerun as root"
    as_root() { sudo "$@"; }
fi

missing=""
for pkg in $PACKAGES; do
    status="$(dpkg-query -W -f='${Status}' "$pkg" 2>/dev/null || true)"
    [ "$status" = "install ok installed" ] || missing="$missing $pkg"
done

if [ -z "$missing" ]; then
    printf 'setup-system: apt packages already installed\n'
    exit 0
fi

printf 'setup-system: installing apt packages:%s\n' "$missing"
as_root env DEBIAN_FRONTEND=noninteractive apt-get update
# shellcheck disable=SC2086 # one argument per package
as_root env DEBIAN_FRONTEND=noninteractive apt-get install -y $missing
printf 'setup-system: done\n'
