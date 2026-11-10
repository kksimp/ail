#!/usr/bin/env bash
# The Mac gate in one command, printing only what failed (ADR 0842).
#   tools/gate/mac-gate.sh [SUITE...]    from the repository root, on the branch under test
# Native only (ADR 0865): runs gate.ail's suites, which build and run everything with
# the native compiler, and prints one line per failing test plus the first 3 lines of
# its message. The self-host stage T runs only on request (gate.ail selfhost). Silence means green:
# the last line is `GATE GREEN` or `GATE RED (N failures)`. Full logs stay in
# $TMPDIR/ail-gate/*.log for a look when a failure needs more.
set -u
cd "$(git rev-parse --show-toplevel)"
# A std.app program (examples/app_todo.ail) would open the host's window and wait for ever on this Mac:
# a host path that doesn't exist makes `app.run` fail at once instead.
export AIL_APP_HOST="${AIL_APP_HOST:-/nonexistent/ail-app-host}"
out="${TMPDIR:-/tmp}/ail-gate"; mkdir -p "$out"; fails=0
# Self-host stage 1: the native compiler must build from compiler/main.ail. nativex reuses it
# (NATIVE_COMPILER), so this is not an extra build; a failure is red here and in nativex.
if ./ail build compiler/main.ail -o "$out/native-compiler" >"$out/selfhost1.log" 2>&1; then
  export NATIVE_COMPILER="$out/native-compiler"
else
  fails=$((fails+1)); echo "RED selfhost stage 1: ./ail build compiler/main.ail"; head -8 "$out/selfhost1.log" | cut -c1-200
fi
# The gate in AIL (RR-4): the program suites without cargo. The native run programs are nativex's.
# The parallel suite checked the interpreter's thread-pool log (AIL_PAR_LOG) and is dropped with the interpreter; native `par` is
# nativex's par_basic and par_overflow.
# D3/D5: every suite runs on ./ail; there is no Rust binary and no cargo.
native_suites="docs cases mistakes sched fmt diff build_cache cli git_deps examples fix explain contracts api_card edit net spec pe asm api_queries misc limits native_misc nativex globals snapshot budget replay std agent_tools cert crypto try"
# The suites, GATE_JOBS at a time (default: one per core, at most 4; GATE_JOBS=1 is one after another). Each suite is its
# own gate.ail run with its own TMPDIR and log (gate_NAME.log), so no two share a scratch tree. The first runs alone, so
# gate.ail is built once and the rest start from the build cache. nativex starts one process per core itself, so it runs
# alone after the others. Nothing in the gate edits the compiler's sources, so the `ail` calls the suites make skip the
# scan for sources newer than .native/ail (AIL_NATIVE_FRESH). times.log has the seconds each suite took, slowest first;
# gate_native.log is every suite's log in order. No `wait -n`: the Mac's bash is 3.2.
export AIL_NATIVE_FRESH=1
jobs=${GATE_JOBS:-}
case "$jobs" in '' | *[!0-9]* | 0) jobs=$(getconf _NPROCESSORS_ONLN 2>/dev/null || echo 2); [ "$jobs" -gt 4 ] && jobs=4 ;; esac
base=${TMPDIR:-/tmp}; base=${base%/}
rm -f "$out"/gate_*.log "$out"/gate_*.rc "$out/times.unsorted"
pids=""
start() { # start SUITE: in the background, under the 4800 second alarm
  s=$1; d="$base/g-$s"; rm -rf "$d"; mkdir -p "$d"
  (
    t0=$(date +%s)
    TMPDIR="$d" perl -e 'alarm 4800; exec @ARGV' ./ail run tools/gate/gate.ail "$s" >"$out/gate_$s.log" 2>&1
    echo $? >"$out/gate_$s.rc"
    echo "$(($(date +%s) - t0)) $s" >>"$out/times.unsorted"
    rm -rf "$d"
  ) &
  pids="$pids $!"
}
pool() { # pool WIDTH SUITE...
  width=$1; shift; limit=1; pids=""
  while [ $# -gt 0 ] || [ -n "$pids" ]; do
    live=""; n=0
    for p in $pids; do
      if kill -0 "$p" 2>/dev/null; then live="$live $p"; n=$((n+1)); else limit=$width; fi
    done
    pids=$live
    while [ $# -gt 0 ] && [ "$n" -lt "$limit" ]; do start "$1"; shift; n=$((n+1)); done
    [ -z "$pids" ] || sleep 1
  done
  wait
}
[ $# -gt 0 ] && native_suites="$*"
pooled=""; alone=""
for s in $native_suites; do if [ "$s" = nativex ]; then alone=nativex; else pooled="$pooled $s"; fi; done
# shellcheck disable=SC2086
pool "$jobs" $pooled
[ -z "$alone" ] || pool 1 $alone
: >"$out/gate_native.log"
for s in $native_suites; do
  cat "$out/gate_$s.log" >>"$out/gate_native.log" 2>/dev/null
  [ "$(cat "$out/gate_$s.rc" 2>/dev/null)" = 0 ] && continue
  fails=$((fails+1))
  echo "RED $s"
  grep -E "^test .* FAILED|panicked at|differs from|^error|disagreement|Symbol not found" "$out/gate_$s.log" | head -8 | cut -c1-200
done
sort -rn "$out/times.unsorted" >"$out/times.log" 2>/dev/null; rm -f "$out/times.unsorted"
# Every native binary must start (dyld/kernel32 mix-ups once aborted all of them).
if ./ail build tests/native/run/arith.ail -o "$out/hello" >/dev/null 2>&1; then
  "$out/hello" >/dev/null 2>&1 || { fails=$((fails+1)); echo "RED native binaries do not start"; }
else fails=$((fails+1)); echo "RED native build of arith.ail"; fi
[ "$fails" = 0 ] && echo "GATE GREEN" || echo "GATE RED ($fails failures); logs in $out"
exit $((fails > 0))
