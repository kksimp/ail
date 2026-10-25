#!/usr/bin/env bash
# `ail api` against its frozen goldens (the outline modes, --symbols, --tokens, --ast and --diff on examples and cases).
# The Rust binary is not needed: the cases were frozen from it into tests/golden/ (tools/golden/golden.py).
#   tools/native-cli/api-parity.sh [ail]    the `ail` to check (default: ./ail)
set -u
cd "$(git rev-parse --show-toplevel)"
bin="$(cd "$(dirname "${1:-./ail}")" && pwd)/$(basename "${1:-./ail}")"
exec python3 tools/golden/golden.py check api --bin "$bin"
