#!/usr/bin/env bash
# The Mac gate in one command, printing only what failed (ADR 0842).
#   tools/gate/mac-gate.sh [BASE_REF]    from the repository root, on the branch under test
# Runs the full selftest with --no-fail-fast, then the native suites, and prints one
# line per failing test plus the first 3 lines of its message. Silence means green:
# the last line is `GATE GREEN` or `GATE RED (N failures)`. Full logs stay in
# $TMPDIR/ail-gate/*.log for a look when a failure needs more.
set -u
cd "$(git rev-parse --show-toplevel)"
out="${TMPDIR:-/tmp}/ail-gate"; mkdir -p "$out"; fails=0
run() { # run NAME CMD...
  name=$1; shift
  if "$@" >"$out/$name.log" 2>&1; then return 0; fi
  fails=$((fails+1))
  echo "RED $name"
  grep -E "^test .* FAILED|panicked at|differs from|^error|disagreement|Symbol not found" "$out/$name.log" | head -8 | cut -c1-200
}
run selftest ./ail selftest --no-fail-fast
run native_suites bash -c 'cd bootstrap && cargo test --release --no-fail-fast --test native --test native_subset_table --test native_asm --test native_globals --test native_test'
# Every native binary must start (dyld/kernel32 mix-ups once aborted all of them).
if ./ail build bootstrap/tests/native/run/arith.ail -o "$out/hello" >/dev/null 2>&1; then
  "$out/hello" >/dev/null 2>&1 || { fails=$((fails+1)); echo "RED native binaries do not start"; }
else fails=$((fails+1)); echo "RED native build of arith.ail"; fi
[ "$fails" = 0 ] && echo "GATE GREEN" || echo "GATE RED ($fails failures); logs in $out"
exit $((fails > 0))
