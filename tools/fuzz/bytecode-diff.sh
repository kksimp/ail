#!/usr/bin/env bash
# Differential fuzzing of the bytecode tier against the tree walker (ADR 0756).
#
#   tools/fuzz/bytecode-diff.sh               # 2,000 programs from seed 1
#   tools/fuzz/bytecode-diff.sh 20000         # N programs
#   tools/fuzz/bytecode-diff.sh 20000 777     # N programs starting at seed 777
#
# Each program runs with the tier and with `ail run --no-bytecode`; output,
# exit code and the trap text must be the same bytes. A disagreement is
# minimised by line deletion and saved in tools/fuzz/found/bytecode/seed-N.ail
# (git-ignored until reported). Sets up its own toolchain through ./ail.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
export AIL_FUZZ_N="${1:-2000}"
export AIL_FUZZ_SEED="${2:-1}"
export AIL_FUZZ_OUT="${AIL_FUZZ_OUT:-$root/tools/fuzz/found/bytecode}"
rm -rf "$AIL_FUZZ_OUT"
echo "bytecode-diff: $AIL_FUZZ_N programs from seed $AIL_FUZZ_SEED"
"$root/ail" selftest --test fuzz_bytecode -- --nocapture
