#!/usr/bin/env bash
# `ail explain` against its frozen goldens (every rule, text and --json).
# The Rust binary is not needed: the cases were frozen from it into tests/golden/ (tools/golden/golden.py).
#   tools/explain/parity.sh [ail]    the `ail` to check (default: ./ail)
set -u
cd "$(git rev-parse --show-toplevel)"
bin="$(cd "$(dirname "${1:-./ail}")" && pwd)/$(basename "${1:-./ail}")"
exec python3 tools/golden/golden.py check explain --bin "$bin"
