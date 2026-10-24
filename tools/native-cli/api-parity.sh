#!/usr/bin/env bash
# The native `ail api` against the Rust one (ADR 0855): stdout and exit status for each
# FILE with the flags in $API_FLAGS (default none).
#   tools/native-cli/api-parity.sh [FILE...]    default: examples/*.ail tests/cases/*.ail
set -uo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$root"
export AIL_ROOT="$root"
files=("$@")
[ ${#files[@]} -gt 0 ] || files=(examples/*.ail tests/cases/*.ail)
native="${NATIVE:-$(mktemp -d)/ail-native}"
[ -x "$native" ] || ./ail build compiler/main.ail -o "$native" > /dev/null 2>&1 || { echo "api-parity: the native CLI doesn't build" >&2; exit 2; }
bad=0
for f in "${files[@]}"; do
    want=$(./ail api ${API_FLAGS:-} "$f" 2>/dev/null; echo "rc=$?")
    got=$("$native" api ${API_FLAGS:-} "$f" 2>/dev/null; echo "rc=$?")
    if [ "$want" != "$got" ]; then bad=$((bad + 1)); echo "DIFF $f"; fi
done
echo "$((${#files[@]} - bad))/${#files[@]} equal"
[ "$bad" -eq 0 ]
