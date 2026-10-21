#!/usr/bin/env bash
# Differential fuzzing of `ail build` output against `ail run`.
#
#   tools/fuzz/native-wide.sh            # 2,000 programs from seed 1
#   tools/fuzz/native-wide.sh 4000 777   # N programs starting at seed 777
#
# Each program that the native backend accepts is built, run, and compared with
# the tree walker: stdout, exit code and the first line of a trap must match.
# Programs outside the native subset are counted as skips. A disagreement is
# minimised by line deletion into tools/fuzz/found/native/seed-N.ail.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
export AIL_FUZZ_N="${1:-2000}"
export AIL_FUZZ_SEED="${2:-1}"
export AIL_FUZZ_OUT="${AIL_FUZZ_OUT:-$root/tools/fuzz/found/native}"
echo "native-wide: $AIL_FUZZ_N programs from seed $AIL_FUZZ_SEED"
"$root/ail" selftest --test fuzz_native_wide -- --nocapture
