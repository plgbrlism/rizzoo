#!/usr/bin/env bash
# Reproducible example run for rizzoo.
#
# Uses XDG_* env vars to keep the whole run inside ./example - your real
# ~/.config/rizzoo and ~/.cache/rizzoo are never touched.
#
# What it does:
#   -i image          color source (dank-shrek.jpg)
#   -P 0              pick source color 0 (avoids the interactive picker)
#   -r                render all templates in example/rizzoo/templates/ to cache
#   -o                copy rendered templates to their config output paths
#   -p                print the palette table
set -euo pipefail

cd "$(dirname "$0")/.."

export XDG_CONFIG_HOME="$(pwd)/example"
export XDG_CACHE_HOME="$(pwd)/example/.cache"

cargo run -- \
  -i example/dank-shrek.jpg \
  -P 0 \
  -r -o -p

echo
echo "=== example/output/all-filters.txt ==="
cat example/output/all-filters.txt

echo
echo "=== example/output/conditionals.txt (config mode=dark) ==="
cat example/output/conditionals.txt

echo
echo "=== Running with --mode light (overrides config mode=dark) ==="
cargo run -- \
  -i example/dank-shrek.jpg \
  -P 0 \
  -r -o -p \
  --mode light

echo
echo "=== example/output/conditionals.txt (--mode light) ==="
cat example/output/conditionals.txt
