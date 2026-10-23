#!/usr/bin/env bash
# The native `ail api --diff` against the Rust one (ADR 0855): stdout and exit status for each
# pair of neighbouring FILEs, plain and --json, and --rev HEAD~1 on each FILE.
#   tools/native-cli/api-diff-parity.sh [FILE...]    default: examples/*.ail bootstrap/tests/cases/*.ail
set -uo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$root"
export AIL_ROOT="$root"
files=("$@")
[ ${#files[@]} -gt 0 ] || files=(examples/*.ail bootstrap/tests/cases/*.ail)
native="${NATIVE:-$(mktemp -d)/ail-native}"
[ -x "$native" ] || ./ail build compiler/main.ail -o "$native" > /dev/null 2>&1 || { echo "api-diff-parity: the native CLI doesn't build" >&2; exit 2; }
bad=0
total=0
same() {
    total=$((total + 1))
    # The --rev checkout lives in a temporary directory whose name differs per run.
    norm='s#/tmp/[^"/ ]*/#TMP/#; s#/var/folders/[^"]*/ail-diff-[^/]*/#TMP/#'
    want=$(./ail api --diff "$@" 2>&1 | sed "$norm"; echo "rc=${PIPESTATUS[0]}")
    got=$("$native" api --diff "$@" 2>&1 | sed "$norm"; echo "rc=${PIPESTATUS[0]}")
    if [ "$want" != "$got" ]; then bad=$((bad + 1)); echo "DIFF $*"; fi
}
prev=""
for f in "${files[@]}"; do
    if [ -n "$prev" ]; then same "$prev" "$f"; same "$prev" "$f" --json; fi
    same --rev HEAD~1 "$f"
    prev="$f"
done
echo "$((total - bad))/$total equal"
[ "$bad" -eq 0 ]
