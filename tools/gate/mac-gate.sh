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
# suites.log is every suite's log in order. No `wait -n`: the Mac's bash is 3.2.
# A first build, before AIL_NATIVE_FRESH is set, so `ail` brings .native/ail up to date with the sources (it does that scan
# on the first call): the build of arith.ail is also the "native binaries start" check at the end.
# The gate's own build cache, kept between runs but apart from ~/.cache/ail/build: another job's `ail` (a lane, a bench,
# a second gate) trims the shared cache by recency and could drop an entry a suite is about to run (the flaky `pe`).
export AIL_CACHE_DIR="${AIL_CACHE_DIR:-$out/cache}"
arith_ok=1
./ail build tests/native/run/arith.ail -o "$out/hello" >/dev/null 2>&1 || arith_ok=0
export AIL_NATIVE_FRESH=1
# fix.auto.3 (ADR 1130): the suites check programs that are wrong on purpose, so no command writes a certain fix into them.
export AIL_FIX=0
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
      case "$s" in nativex*) [ "$(cat "$out/gate_selfhost1.rc" 2>/dev/null)" = 0 ] || unset NATIVE_COMPILER ;; esac # no stage 1: nativex builds its own
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
    while [ $# -gt 0 ] && [ "$n" -lt "$limit" ]; do
      # A nativex shard waits for stage 1 (selfhost1) to finish, pass or fail; the other suites go on filling the pool.
      case "$1" in nativex*) if [ -n "$sel" ] && [ -f "$out/gate_selfhost1.name" ] && [ ! -f "$out/gate_selfhost1.rc" ]; then break; fi ;; esac
      start "$1"; shift; n=$((n+1))
    done
    [ -z "$pids" ] || sleep 1
  done
  wait
}
fullrun=1; [ $# -gt 0 ] && fullrun=0
[ $# -gt 0 ] && native_suites="$*"
[ -n "$rerun" ] && fullrun=0
[ -n "$rerun" ] && native_suites="$rerun"
# GATE_SKIP_HOSTFREE=1: leave out the suites of hostfree.txt (see there); the verdict line says so.
skipped=""
if [ "${GATE_SKIP_HOSTFREE:-}" = 1 ]; then
  kept=""
  for s in $native_suites; do
    if grep -qx "$s" tools/gate/hostfree.txt; then skipped="$skipped $s"; else kept="$kept $s"; fi
  done
  native_suites=${kept# }
  [ -z "$skipped" ] || echo "SKIPPED host-free suites (the Mac gate must have been green on this head):$skipped"
fi
# selfhost1 goes in right after the first (lone) suite, so it overlaps the long suites; the nativex shards need it, and wait for it.
# nativex is sharded into the pool (`nativex@K/N`, one process each): its parts ran serially while three cores idled.
sel=""
for s in $native_suites; do case "$s" in nativex*) sel=selfhost1 ;; esac; done
case " $native_suites " in *" selfhost1 "*) sel="" ;; esac
# checkfuzz times 300 generated programs under one alarm: it runs alone after the pool, so the other suites' load can't
# push it past the alarm (a red under load on 2026-11-15).
late=""; early=""
for s in $native_suites; do if [ "$s" = checkfuzz ]; then late="$late $s"; else early="$early $s"; fi; done
set -- $early
first=${1:-}; [ $# -gt 0 ] && shift
rest="$*"
export NATIVE_COMPILER="$out/native-compiler"
# shellcheck disable=SC2086
pool "$jobs" $first $sel $rest
# shellcheck disable=SC2086
[ -z "$late" ] || pool 1 $late
: >"$out/suites.log"
shown=${GATE_SHOW:-60} # RED lines printed per red suite: all of a suite's failures in one pass, not the first 8
for s in $sel $native_suites; do
  t=$(tag "$s")
  cat "$out/gate_$t.log" >>"$out/suites.log" 2>/dev/null
  [ "$(cat "$out/gate_$t.rc" 2>/dev/null)" = 0 ] && continue
  fails=$((fails+1))
  if [ "$s" = selfhost1 ]; then echo "RED selfhost stage 1: ./ail build compiler/main.ail"; head -8 "$out/gate_$t.log" | cut -c1-200; continue; fi
  echo "RED $s"
  grep -E "^RED |^test .* FAILED|panicked at|differs from|^error|disagreement|Symbol not found" "$out/gate_$t.log" | head -"$shown" | cut -c1-200
done
# The shards of a suite must have run every item exactly once: all N shards reported, one shared total, and the ran counts add up.
if [ "$fullrun" = 1 ] && [ "$fails" = 0 ]; then
  # Only this run's suite logs (each has a .name): suites.log repeats their lines, and a stray older log must not count.
  logs=""
  for nf in "$out"/gate_*.name; do [ -f "$nf" ] || continue; t=${nf##*/gate_}; logs="$logs $out/gate_${t%.name}.log"; done
  # shellcheck disable=SC2086 # $logs is a list of paths without spaces ($TMPDIR has none on the gate hosts)
  bad=$(cat $logs 2>/dev/null | grep -h '^SHARD ' | awk '{ split($3, a, "/"); k = $2; n[k] = a[2]; seen[k, a[1]]++; cnt[k]++; sum[k] += $7; tots[k] = tots[k] " " a[1] ":" $5; if ($7 == 0) zero[k] = zero[k] " " a[1]; if (!(k in tot)) tot[k] = $5; else if (tot[k] != $5) mism[k] = 1 }
    END { for (k in n) { if (zero[k] != "") print k ": shard" zero[k] " ran 0 items"; if (cnt[k] != n[k] || sum[k] != tot[k] || mism[k]) print k ": " cnt[k] " of " n[k] " shards, ran " sum[k] " of " tot[k] " (totals, shard:total:" tots[k] ")"; for (j = 0; j < n[k]; j++) if (seen[k, j] != 1) print k ": shard " j " ran " seen[k, j] + 0 " times" } }')
  if [ -n "$bad" ]; then fails=$((fails+1)); echo "RED shards do not cover their suite exactly once"; echo "$bad"; fi
fi
sort -rn "$out/times.unsorted" >"$out/times.log" 2>/dev/null; rm -f "$out/times.unsorted"
# Every native binary must start (dyld/kernel32 mix-ups once aborted all of them).
if [ "$arith_ok" = 1 ]; then
  "$out/hello" >/dev/null 2>&1 || { fails=$((fails+1)); echo "RED native binaries do not start"; }
else fails=$((fails+1)); echo "RED native build of arith.ail"; fi
[ "$fails" = 0 ] && echo "GATE GREEN${skipped:+ (not run:$skipped)}" || echo "GATE RED ($fails failures); logs in $out"
exit $((fails > 0))
