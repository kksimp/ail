#!/usr/bin/env bash
# Wall time of `ail mcp` calls, cold (AIL_MCP_COLD=1, every call through the launcher) against
# warm (mcp.warm.1-4). From the repository root:
#
#   tools/ailmcp/bench.sh [PROGRAM.ail] [CALLS]
#
# Three sessions of CALLS `ail_check` calls each, run in the program's own directory so its
# modules are found: cold; warm with the file spelt differently per call (`./` prefixes, so
# the memo misses and each call takes the direct path); warm with the same spelling every
# call (the memo after the first).
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
prog="${1:-$root/examples/hello.ail}"
n="${2:-10}"
prog="$(cd "$(dirname "$prog")" && pwd)/$(basename "$prog")"
dir="$(dirname "$prog")"
base="$(basename "$prog")"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
now() { perl -MTime::HiRes=time -e 'printf "%d\n", time * 1000'; }
calls() {
    for i in $(seq 1 "$n"); do
        f="$base"
        if [ "$1" = many ]; then for _ in $(seq 1 "$i"); do f="./$f"; done; fi
        printf '{"jsonrpc":"2.0","id":%d,"method":"tools/call","params":{"name":"ail_check","arguments":{"program":"%s"}}}\n' "$i" "$f"
    done
}
session() {
    # $1: cold or warm; $2: one or many files.
    local t0 t1
    calls "$2" > "$work/in.txt"
    t0="$(now)"
    if [ "$1" = cold ]; then
        (cd "$dir" && AIL_MCP_COLD=1 "$root/ail" mcp < "$work/in.txt" > "$work/out.txt")
    else
        (cd "$dir" && "$root/ail" mcp < "$work/in.txt" > "$work/out.txt")
    fi
    t1="$(now)"
    grep -c '"isError":false' "$work/out.txt" > "$work/ok.txt" || true
    echo "$1 $2: $((t1 - t0)) ms for $n calls ($(cat "$work/ok.txt") clean), $(grep -o '"ail/warm":"[a-z]*"' "$work/out.txt" | sort | uniq -c | tr -s ' \n' ' ')"
}
# One start-up first, so the native CLI and the server's build are cached for every session.
echo '{"jsonrpc":"2.0","id":1,"method":"ping"}' | "$root/ail" mcp > /dev/null
echo "program: $prog"
session cold many
session warm many
session warm one
