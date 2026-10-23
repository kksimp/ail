#!/usr/bin/env bash
# The AIL formatter against the Rust one (RR-5). Builds compiler/main.ail natively,
# then formats every .ail file in the repository (as written, and with every
# identifier lengthened so lines wrap) both ways and compares stdout and exit codes.
#   tools/fmt/parity.sh [N]     check the first N files (default: all)
set -u
cd "$(git rev-parse --show-toplevel)"
out="${TMPDIR:-/tmp}/fmt-parity"; mkdir -p "$out"
rust=./ail; native="$out/ail-native"
./ail build compiler/main.ail -o "$native" >/dev/null 2>&1 || { echo "RED native build"; exit 1; }
files=$(git ls-files '*.ail'); [ -n "${1:-}" ] && files=$(echo "$files" | head -"$1")
fail=0; n=0
for f in $files; do
  for mode in plain long; do
    in=$f
    if [ $mode = long ]; then python3 tools/fmt/lengthen.py "$f" "$out/long.ail"; in="$out/long.ail"; fi
    $rust fmt --stdout "$in" >"$out/r.out" 2>/dev/null; rc=$?
    "$native" fmt --stdout "$in" >"$out/n.out" 2>/dev/null; nc=$?
    n=$((n+1))
    if [ $rc != $nc ] || ! cmp -s "$out/r.out" "$out/n.out"; then fail=$((fail+1)); echo "DIFF $mode $f (exit $rc/$nc)"; fi
  done
done
[ $fail = 0 ] && echo "FMT PARITY GREEN ($n runs)" || echo "FMT PARITY RED ($fail of $n)"
exit $((fail > 0))
