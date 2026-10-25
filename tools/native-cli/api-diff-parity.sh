#!/usr/bin/env bash
# `ail api --diff` against its frozen goldens (neighbouring files, plain and --json).
# The Rust binary is not needed: the cases were frozen from it into tests/golden/ (tools/golden/golden.py).
#   tools/native-cli/api-diff-parity.sh [ail]    the `ail` to check (default: ./ail)
set -u
cd "$(git rev-parse --show-toplevel)"
bin="$(cd "$(dirname "${1:-./ail}")" && pwd)/$(basename "${1:-./ail}")"
exec python3 tools/golden/golden.py check api --bin "$bin" --match "api --diff"
