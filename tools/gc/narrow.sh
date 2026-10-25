#!/usr/bin/env bash
# Find the allocation right after a heap-header corruption (AIL_GC_VERIFY, gc.ail).
#   tools/gc/narrow.sh CMD [ARGS...]     e.g. tools/gc/narrow.sh /tmp/A fmt --check compiler
# Phase A checks at every collection and reads the allocation count T of the first bad header;
# phase B checks every 1000th allocation below T; phase C every allocation in the last 1000.
# The last report's call stack names the function that made the allocation right after the bad store.
# The run must be deterministic (the Mac is; Linux needs ASLR off: setarch -R).
set -u
count() { sed -n 's/.*allocations \([0-9]*\).*/\1/p' | head -n 1; }
a="$(AIL_GC_VERIFY=1 "$@" 2>&1 | tee /tmp/gc-narrow-a.txt | count)"
[ -n "$a" ] || { echo "phase A: no bad header (rc above); see /tmp/gc-narrow-a.txt"; exit 1; }
lo=$((a > 3000000 ? a - 3000000 : 0))
b="$(AIL_GC_VERIFY="$lo,$((a + 1)),1000" "$@" 2>&1 | tee /tmp/gc-narrow-b.txt | count)"
[ -n "$b" ] || { echo "phase B: not reproduced; see /tmp/gc-narrow-b.txt"; exit 1; }
AIL_GC_VERIFY="$((b > 1000 ? b - 1000 : 0)),$((b + 1)),1" "$@" 2>&1 | tee /tmp/gc-narrow-c.txt | grep -v '^$' | head -20
