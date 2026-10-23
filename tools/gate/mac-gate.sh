#!/usr/bin/env bash
# The Mac gate in one command, printing only what failed (ADR 0842).
#   tools/gate/mac-gate.sh [--full]    from the repository root, on the branch under test
# Native only (ADR 0865): runs gate.ail's suites, which build and run everything with
# the native compiler, and prints one line per failing test plus the first 3 lines of
# its message. The cargo selftest (the interpreter and Rust-internal tests) runs only
# with --full, and the self-host stage T only on request (gate.ail selfhost). Silence means green:
# the last line is `GATE GREEN` or `GATE RED (N failures)`. Full logs stay in
# $TMPDIR/ail-gate/*.log for a look when a failure needs more.
set -u
full=0; [ "${1:-}" = "--full" ] && full=1
cd "$(git rev-parse --show-toplevel)"
# A std.app program (examples/app_todo.ail) would open the host's window and wait for ever on this Mac:
# a host path that doesn't exist makes `app.run` fail at once instead.
export AIL_APP_HOST="${AIL_APP_HOST:-/nonexistent/ail-app-host}"
out="${TMPDIR:-/tmp}/ail-gate"; mkdir -p "$out"; fails=0
run() { # run NAME CMD...
  name=$1; shift
  if "$@" >"$out/$name.log" 2>&1; then return 0; fi
  fails=$((fails+1))
  echo "RED $name"
  grep -E "^test .* FAILED|panicked at|differs from|^error|disagreement|Symbol not found" "$out/$name.log" | head -8 | cut -c1-200
}
# The cargo copies of the ported suites are deleted (B6's list); --full runs what is left.
[ "$full" = 1 ] && run selftest ./ail selftest --no-fail-fast
# Self-host stage 1: the native compiler must build from compiler/main.ail. nativex reuses it
# (NATIVE_COMPILER), so this is not an extra build; a failure is red here and in nativex.
if ./ail build compiler/main.ail -o "$out/native-compiler" >"$out/selfhost1.log" 2>&1; then
  export NATIVE_COMPILER="$out/native-compiler"
else
  fails=$((fails+1)); echo "RED selfhost stage 1: ./ail build compiler/main.ail"; head -8 "$out/selfhost1.log" | cut -c1-200
fi
# The gate in AIL (RR-4): the program suites without cargo. The native run programs are nativex's.
run gate_ail ./ail run tools/gate/gate.ail examples tools cases explain fix cli mistakes contracts snapshot misc docs fmt budget limits diff build_cache parallel api_queries replay api_card edit git_deps net bytecode spec native_misc nativex globals pe asm
# Every native binary must start (dyld/kernel32 mix-ups once aborted all of them).
if ./ail build bootstrap/tests/native/run/arith.ail -o "$out/hello" >/dev/null 2>&1; then
  "$out/hello" >/dev/null 2>&1 || { fails=$((fails+1)); echo "RED native binaries do not start"; }
else fails=$((fails+1)); echo "RED native build of arith.ail"; fi
[ "$fails" = 0 ] && echo "GATE GREEN" || echo "GATE RED ($fails failures); logs in $out"
exit $((fails > 0))
