#!/usr/bin/env bash
# The AIL test runner against the interpreter's `ail test` on tools/ailtest/cases (lane B8): the same
# `ok`/`FAIL` lines, seeds and shrunk inputs, ignoring `at L:C` positions.
set -uo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$root"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
keep() { grep -E '^(ok|FAIL|  input)' | sed -E 's/ at [0-9]+:[0-9]+//g'; }
fail=0
for f in tools/ailtest/cases/*.ail; do
    ./ail test "$f" 2>&1 | keep >"$tmp/want"
    ./ail run tools/ailtest/ailtest.ail "$f" 2>&1 | keep >"$tmp/got"
    if diff "$tmp/got" "$tmp/want" >"$tmp/diff"; then
        echo "same $f"
    else
        echo "DIFF $f"
        head -20 "$tmp/diff"
        fail=1
    fi
done
exit $fail
