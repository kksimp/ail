#!/usr/bin/env bash
# The Mac gate in one command, printing only what failed (ADR 0842).
#   tools/gate/mac-gate.sh [SUITE...]    from the repository root, on the branch under test
#   tools/gate/mac-gate.sh --failed      only the suites that were red in the last run (their gate_*.rc files); for a
#                                        fix that touches nothing but what those suites read. The release gate is a full run.
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
# Self-host stage 1: the native compiler must build from compiler/main.ail. nativex reuses it (NATIVE_COMPILER), so this is
# not an extra build; a failure is red here and in nativex. It runs as one more job of the pool below (`selfhost1`), next
# to the suites, and nativex starts only after it.
# The gate in AIL (RR-4): the program suites without cargo. The native run programs are nativex's.
# The parallel suite checked the interpreter's thread-pool log (AIL_PAR_LOG) and is dropped with the interpreter; native `par` is
# nativex's par_basic and par_overflow.
# D3/D5: every suite runs on ./ail; there is no Rust binary and no cargo.
native_suites=$(grep -v '^#' tools/gate/suites.txt | tr '\n' ' ')
# The suites, GATE_JOBS at a time (default: one per core, at most 4; GATE_JOBS=1 is one after another). Each suite is its
# own gate.ail run with its own TMPDIR and log (gate_NAME.log), so no two share a scratch tree. The first runs alone, so
# gate.ail is built once and the rest start from the build cache. nativex starts one process per core itself, so it runs
# alone after the others. Nothing in the gate edits the compiler's sources, so the `ail` calls the suites make skip the
# scan for sources newer than .native/ail (AIL_NATIVE_FRESH). times.log has the seconds each suite took, slowest first;
# gate_native.log is every suite's log in order. No `wait -n`: the Mac's bash is 3.2.
# A first build, before AIL_NATIVE_FRESH is set, so `ail` brings .native/ail up to date with the sources (it does that scan
# on the first call): the build of arith.ail is also the "native binaries start" check at the end.
arith_ok=1
./ail build tests/native/run/arith.ail -o "$out/hello" >/dev/null 2>&1 || arith_ok=0
export AIL_NATIVE_FRESH=1
jobs=${GATE_JOBS:-}
case "$jobs" in '' | *[!0-9]* | 0) jobs=$(getconf _NPROCESSORS_ONLN 2>/dev/null || echo 2); [ "$jobs" -gt "${GATE_JOBS_MAX:-4}" ] && jobs=${GATE_JOBS_MAX:-4} ;; esac
base=${TMPDIR:-/tmp}; base=${base%/}
# `--failed`: the suites whose last run was red (a missing or non-zero .rc), before the old logs are cleared.
rerun=""
if [ "${1:-}" = --failed ]; then
  shift
  for rc in "$out"/gate_*.rc; do
    [ -f "$rc" ] || continue
    [ "$(cat "$rc")" = 0 ] && continue
    n=${rc##*/gate_}; n=${n%.rc}; rerun="$rerun $(cat "$out/gate_$n.name" 2>/dev/null || echo "$n")"
  done
  [ -n "$rerun" ] || { echo "no red suites in the last run (logs in $out)"; exit 0; }
fi
rm -f "$out"/gate_*.log "$out"/gate_*.rc "$out"/gate_*.name "$out/times.unsorted"
pids=""
tag() { printf '%s' "$1" | tr '@/' '__'; } # a suite name as a file name: `cases@1/4` is `cases_1_4`
start() { # start SUITE: in the background, under the 4800 second alarm
  s=$1; t=$(tag "$s"); d="$base/g-$t"; rm -rf "$d"; mkdir -p "$d"
  printf '%s' "$s" >"$out/gate_$t.name"
  (
    t0=$(date +%s)
    if [ "$s" = selfhost1 ]; then
      TMPDIR="$d" perl -e 'alarm 4800; exec @ARGV' ./ail build compiler/main.ail -o "$out/native-compiler" >"$out/gate_$t.log" 2>&1
    else
      TMPDIR="$d" perl -e 'alarm 4800; exec @ARGV' ./ail run tools/gate/gate.ail "$s" >"$out/gate_$t.log" 2>&1
    fi
    echo $? >"$out/gate_$t.rc"
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
fullrun=1; [ $# -gt 0 ] && fullrun=0
[ $# -gt 0 ] && native_suites="$*"
[ -n "$rerun" ] && fullrun=0
[ -n "$rerun" ] && native_suites="$rerun"
pooled=""; alone=""
for s in $native_suites; do if [ "$s" = nativex ]; then alone=nativex; else pooled="$pooled $s"; fi; done
# selfhost1 goes in right after the first (lone) suite, so it overlaps the long suites; nativex needs it only when it starts.
# A full run, and a `--failed` rerun of nativex, build it; a named suite list that includes nativex does too.
set -- $pooled
first=${1:-}; [ $# -gt 0 ] && shift
rest="$*"
sel=""; [ -n "$alone" ] && sel=selfhost1
case " $native_suites " in *" selfhost1 "*) sel="" ;; esac
export NATIVE_COMPILER="$out/native-compiler"
# shellcheck disable=SC2086
pool "$jobs" $first $sel $rest
if [ -n "$alone" ]; then
  if [ "$(cat "$out/gate_selfhost1.rc" 2>/dev/null)" != 0 ]; then unset NATIVE_COMPILER; fi
  pool 1 $alone
fi
: >"$out/gate_native.log"
shown=${GATE_SHOW:-60} # RED lines printed per red suite: all of a suite's failures in one pass, not the first 8
for s in $sel $native_suites; do
  t=$(tag "$s")
  cat "$out/gate_$t.log" >>"$out/gate_native.log" 2>/dev/null
  [ "$(cat "$out/gate_$t.rc" 2>/dev/null)" = 0 ] && continue
  fails=$((fails+1))
  if [ "$s" = selfhost1 ]; then echo "RED selfhost stage 1: ./ail build compiler/main.ail"; head -8 "$out/gate_$t.log" | cut -c1-200; continue; fi
  echo "RED $s"
  grep -E "^RED |^test .* FAILED|panicked at|differs from|^error|disagreement|Symbol not found" "$out/gate_$t.log" | head -"$shown" | cut -c1-200
done
# The shards of a suite must have run every item exactly once: all N shards reported, one shared total, and the ran counts add up.
if [ "$fullrun" = 1 ] && [ "$fails" = 0 ]; then
  bad=$(grep -h '^SHARD ' "$out"/gate_*.log | awk '{ split($3, a, "/"); k = $2; n[k] = a[2]; seen[k, a[1]]++; cnt[k]++; sum[k] += $7; if ($7 == 0) zero[k] = zero[k] " " a[1]; if (!(k in tot)) tot[k] = $5; else if (tot[k] != $5) mism[k] = 1 }
    END { for (k in n) { if (zero[k] != "") print k ": shard" zero[k] " ran 0 items"; if (cnt[k] != n[k] || sum[k] != tot[k] || mism[k]) print k ": " cnt[k] " of " n[k] " shards, ran " sum[k] " of " tot[k]; for (j = 0; j < n[k]; j++) if (seen[k, j] != 1) print k ": shard " j " ran " seen[k, j] + 0 " times" } }')
  if [ -n "$bad" ]; then fails=$((fails+1)); echo "RED shards do not cover their suite exactly once"; echo "$bad"; fi
fi
sort -rn "$out/times.unsorted" >"$out/times.log" 2>/dev/null; rm -f "$out/times.unsorted"
# Every native binary must start (dyld/kernel32 mix-ups once aborted all of them).
if [ "$arith_ok" = 1 ]; then
  "$out/hello" >/dev/null 2>&1 || { fails=$((fails+1)); echo "RED native binaries do not start"; }
else fails=$((fails+1)); echo "RED native build of arith.ail"; fi
[ "$fails" = 0 ] && echo "GATE GREEN" || echo "GATE RED ($fails failures); logs in $out"
exit $((fails > 0))
