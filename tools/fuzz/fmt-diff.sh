#!/usr/bin/env bash
# Fuzzing `ail fmt`: idempotence and meaning preserved.
#
#   tools/fuzz/fmt-diff.sh            # 2,000 programs from seed 1
#   tools/fuzz/fmt-diff.sh 4000 777   # N programs starting at seed 777
#
# Each program has its layout messed up, then goes through `ail fmt` twice; the
# text must not change the second time, and `ail check` and `ail run` (walker and
# bytecode tier) must say the same before and after. A disagreement is
# minimised by line deletion into tools/fuzz/found/fmt/seed-N.ail.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
export AIL_FUZZ_N="${1:-2000}"
export AIL_FUZZ_SEED="${2:-1}"
export AIL_FUZZ_OUT="${AIL_FUZZ_OUT:-$root/tools/fuzz/found/fmt}"
echo "fmt-diff: $AIL_FUZZ_N programs from seed $AIL_FUZZ_SEED"
"$root/ail" selftest --test fuzz_fmt -- --nocapture
