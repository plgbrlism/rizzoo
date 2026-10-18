#!/usr/bin/env bash
# build release and install the binary
set -euo pipefail

prefix="${1:-/usr/local}"

cargo build --release

install -d "$prefix/bin"
install -m755 target/release/rizzoo "$prefix/bin/rizzoo"

echo "installed rizzoo to $prefix/bin/rizzoo"
