#!/usr/bin/env bash
# std.sqlite writer against the real SQLite library (std.sqlite.15-22): from the repo root,
#   tests/sqlite/write/run.sh [scenario ...]     scenarios: basic unique autoinc overflow types deep deep3
# Needs python3 (its sqlite3 module is the reference). Builds runner.ail and edit.ail with ./ail.
set -eu
ROOT=$(cd "$(dirname "$0")/../../.." && pwd)
cd "$ROOT"
W=$(mktemp -d "${TMPDIR:-/tmp}/sqlite-write-bin.XXXXXX")
./ail build tests/sqlite/write/runner.ail -o "$W/runner"
./ail build tests/sqlite/write/edit.ail -o "$W/edit"
python3 tests/sqlite/write/verify.py "$W/runner" "$W/edit" "$@"
