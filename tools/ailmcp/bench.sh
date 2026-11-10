#!/usr/bin/env bash
# Wall time of `ail mcp` calls, cold (AIL_MCP_COLD=1, every call through the launcher) against
# warm (mcp.warm.1-4). From the repository root:
#
#   tools/ailmcp/bench.sh [PROGRAM.ail] [CALLS]
#
# Three sessions of CALLS `ail_check` calls each: cold; warm with a different file per call
# (the direct path); warm with the same file every call (the memo after the first).
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
prog="${1:-$root/examples/hello.ail}"
n="${2:-10}"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
for i in $(seq 1 "$n"); do cp "$prog" "$work/p$i.ail"; done
now() { perl -MTime::HiRes=time -e 'printf "%d\n", time * 1000'; }
calls() {
    for i in $(seq 1 "$n"); do
        f="p1.ail"; [ "$1" = many ] && f="p$i.ail"
        printf '{"jsonrpc":"2.0","id":%d,"method":"tools/call","params":{"name":"ail_check","arguments":{"program":"%s"}}}\n' "$i" "$f"
    done
}
session() {
    # $1: cold or warm; $2: one or many files.
    local t0 t1
    calls "$2" > "$work/in.txt"
    t0="$(now)"
    if [ "$1" = cold ]; then
        (cd "$work" && AIL_MCP_COLD=1 "$root/ail" mcp < in.txt > out.txt)
    else
        (cd "$work" && "$root/ail" mcp < in.txt > out.txt)
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
