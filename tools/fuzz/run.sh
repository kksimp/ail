#!/usr/bin/env bash
# Fuzz and differential run (ADR 0269-0271). Linux and macOS; Windows: run.ps1.
#
#   tools/fuzz/run.sh                 # 10,000 programs from seed 1
#   tools/fuzz/run.sh 50000           # N programs
#   tools/fuzz/run.sh 10000 777       # N programs starting at seed 777
#
# Sets up its own toolchain through ./ail. A failure is minimised and saved
# in tools/fuzz/failures/ (git-ignored) as gen-SEED.ail or mut-SEED.ail; the
# seed alone reproduces it. Every crash becomes a case (fuzz.policy.1).
# The TLS and X.509 parser fuzzers need a test hook in the runtime: tools/native-tls/fuzz-check.sh.
# Also runs the module fuzzers (tests/fuzz/modules, ADR 0461); `ail selftest --fuzz N [SEED]` is the short form.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
export AIL_FUZZ_N="${1:-10000}"
export AIL_FUZZ_SEED="${2:-1}"
export AIL_FUZZ_OUT="${AIL_FUZZ_OUT:-$root/tools/fuzz/failures}"
rm -rf "$AIL_FUZZ_OUT"
echo "fuzz: $AIL_FUZZ_N programs from seed $AIL_FUZZ_SEED"
"$root/ail" selftest --test fuzz --test fuzz_modules fuzz_ -- --nocapture
