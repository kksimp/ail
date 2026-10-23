#!/usr/bin/env bash
# The native `ail effects` against the Rust one (ADR 0855): stdout and stderr compared
# as text, for FILE... (default examples/*.ail), plain and --json.
#   tools/native-cli/effects-parity.sh [FILE...]
set -uo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$root"
export AIL_ROOT="$root" AIL_COMPILER="$root/compiler"
files=("$@")
[ ${#files[@]} -gt 0 ] || files=(examples/*.ail)
native="${NATIVE:-$(mktemp -d)/ail-native}"
[ -x "$native" ] || ./ail build compiler/main.ail -o "$native" > /dev/null 2>&1 || { echo "effects-parity: the native CLI doesn't build" >&2; exit 2; }
bad=0
for f in "${files[@]}"; do
    for j in "" --json; do
        if [ "$(./ail effects $j "$f" 2>&1)" != "$("$native" effects $j "$f" 2>&1)" ]; then bad=$((bad + 1)); echo "DIFF $j $f"; fi
    done
done
echo "$((${#files[@]} * 2 - bad))/$((${#files[@]} * 2)) equal"
[ "$bad" -eq 0 ]
