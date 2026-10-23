#!/usr/bin/env bash
# The AIL `ail explain` against the Rust one (RR-5). Builds compiler/main.ail natively, then
# runs `explain` both ways on the rule list, on every rule in the specification (text and
# --json), on `bootstrap.unsupported` and on unknown rules, and compares stdout, stderr and
# exit codes.
#   tools/explain/parity.sh [N]     check the first N rules (default: all)
set -u
cd "$(git rev-parse --show-toplevel)"
out="${TMPDIR:-/tmp}/explain-parity"; mkdir -p "$out"
rust=./ail; native="$out/ail-native"
./ail build compiler/main.ail -o "$native" >/dev/null 2>&1 || { echo "RED native build"; exit 1; }
export AIL_ROOT="$PWD"
rules=$(./ail explain | awk 'NF>=2 && $1 ~ /\./ {print $1}')
[ -n "${1:-}" ] && rules=$(echo "$rules" | head -"$1")
rules="$rules bootstrap.unsupported no.such.rule expr.bind.99 zzz.yy.1 --foo"
fail=0; n=0
cmp_run() {
  "$rust" explain "$@" >"$out/r.out" 2>"$out/r.err"; rc=$?
  "$native" explain "$@" >"$out/n.out" 2>"$out/n.err"; nc=$?
  n=$((n+1))
  if [ $rc != $nc ] || ! cmp -s "$out/r.out" "$out/n.out" || ! cmp -s "$out/r.err" "$out/n.err"; then
    fail=$((fail+1)); echo "DIFF explain $* (exit $rc/$nc)"
  fi
}
cmp_run; cmp_run --json
for r in $rules; do cmp_run "$r"; cmp_run "$r" --json; done
[ $fail = 0 ] && echo "EXPLAIN PARITY GREEN ($n runs)" || echo "EXPLAIN PARITY RED ($fail of $n)"
exit $((fail > 0))
