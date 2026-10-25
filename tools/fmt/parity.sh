#!/usr/bin/env bash
# `ail fmt` against its frozen goldens (every .ail file in the repository, as written and lengthened).
# The Rust binary is not needed: the cases were frozen from it into tests/golden/ (tools/golden/golden.py).
#   tools/fmt/parity.sh [ail]    the `ail` to check (default: ./ail)
set -u
cd "$(git rev-parse --show-toplevel)"
bin="$(cd "$(dirname "${1:-./ail}")" && pwd)/$(basename "${1:-./ail}")"
exec python3 tools/golden/golden.py check fmt --bin "$bin"
