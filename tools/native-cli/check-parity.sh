#!/usr/bin/env bash
# The native `ail check` against the Rust one (ADR 0855): the diagnostics each
# prints on stderr for each FILE, compared as text. Lines `fix: ...` are left
# out (the native checker has no fixes yet; --with-fixes keeps them).
#   tools/native-cli/check-parity.sh [--json] [FILE...]    default: tests/cases/*.ail
set -uo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$root"
export AIL_ROOT="$root"
json=""; [ "${1:-}" = --json ] && { json=--json; shift; }
files=("$@")
[ ${#files[@]} -gt 0 ] || files=(tests/cases/*.ail)
# The native CLI is built once with the Rust-hosted `ail build`.
native="${NATIVE:-$(mktemp -d)/ail-native}"
[ -x "$native" ] || ./ail build compiler/main.ail -o "$native" > /dev/null 2>&1 || { echo "check-parity: the native CLI doesn't build" >&2; exit 2; }
bad=0
for f in "${files[@]}"; do
    want=$(./ail check $json "$f" 2>&1 >/dev/null | grep -v '^fix: ')
    got=$("$native" check-program $json "$f" 2>&1 >/dev/null | grep -v '^fix: ')
    if [ "$want" != "$got" ]; then bad=$((bad + 1)); echo "DIFF $f"; fi
done
echo "$((${#files[@]} - bad))/${#files[@]} equal"
[ "$bad" -eq 0 ]
