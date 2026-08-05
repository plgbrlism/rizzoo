#!/usr/bin/env bash
# Demonstrates rizzoo's behavior on a broken template.
#
# Uses a SEPARATE XDG_CONFIG_HOME (example/bad) so the good run (run.sh)
# and this error run never share template files.
#
# Known limitation this exercises: `-r` aborts on the first bad template and
# there is no --continue-on-error (matugen has one; rizzoo does not yet).
set -uo pipefail

cd "$(dirname "$0")/.."

export XDG_CONFIG_HOME="$(pwd)/example/bad"
export XDG_CACHE_HOME="$(pwd)/example/.cache"

echo "Running with example/bad/rizzoo/templates/error-cases.tpl ..."
echo "(this run is expected to FAIL)"
echo

cargo run -- -c "#7c3aed" -P 0 -r
code=$?
echo
echo "exit code: $code"
