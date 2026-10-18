#!/usr/bin/env bash
# build release, install binary to $prefix
set -euo pipefail

prefix="${1:-/usr/local}"

if [ "$(id -u)" = 0 ]; then
    [ -n "${SUDO_USER:-}" ] || { echo "root has no rustup; run via sudo from your user" >&2; exit 1; }
    # cargo/rustup live in the invoking user's home, not root's
    sudo -H -u "$SUDO_USER" cargo build --release
else
    # default prefix needs root; let sudo re-enter and build as the invoking user
    mkdir -p "$prefix/bin" 2>/dev/null
    [ -w "$prefix/bin" ] || exec sudo -- "$0" "$prefix"
    cargo build --release
fi

install -d "$prefix/bin"
install -m 755 target/release/rizzoo "$prefix/bin/rizzoo"

echo "installed rizzoo to $prefix/bin/rizzoo"
