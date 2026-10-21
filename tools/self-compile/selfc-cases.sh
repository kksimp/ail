#!/usr/bin/env bash
# Self-compile repro cases (lane 226, ADR 0715): each bootstrap/tests/native/selfc/NAME.ail is one
# construct the self-compile survey counts as blocked, with NAME.out as the `ail run` output.
# This builds each natively, runs it and compares with NAME.out. Measurement only: not a
# merge gate (the native harness reads tests/native/run, not this directory).
#
#   tools/self-compile/selfc-cases.sh [OUT_DIR]     # prints PASS/FAIL per case and the count
set -uo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$root" || exit 2
[ -x ./ail ] || { echo "selfc-cases.sh: no ./ail launcher" >&2; exit 2; }
out="${1:-${TMPDIR:-/tmp}/selfc-$$}"
mkdir -p "$out"
pass=0; total=0
for src in bootstrap/tests/native/selfc/*.ail; do
    name="$(basename "$src" .ail)"
    total=$((total + 1))
    if ./ail build "$src" -o "$out/$name" > "$out/$name.build" 2>&1 \
        && "$out/$name" > "$out/$name.txt" 2>/dev/null \
        && cmp -s "$out/$name.txt" "bootstrap/tests/native/selfc/$name.out"; then
        pass=$((pass + 1)); echo "PASS $name"
    else
        echo "FAIL $name: $(grep -m1 -o 'error\[[^]]*\]: .*' "$out/$name.build" 2>/dev/null | cut -c1-120)"
    fi
done
echo "selfc cases: $pass of $total pass"
