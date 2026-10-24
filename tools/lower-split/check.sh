#!/usr/bin/env bash
# Proof that the lower.ail split changes no behaviour (ADR 0823): the MIR
# dump of every native test case and perfbench program, and the x86-64 and
# aarch64 executables of the perfbench programs, before (HEAD) and after
# (the working tree). Run from the repository root with the split applied
# and uncommitted. Usage: check.sh AIL_BINARY OUT_DIR
set -uo pipefail
ail="$1"; out="$2"
mkdir -p "$out/before" "$out/after"
progs=$( (ls tests/native/*.ail tests/native/run/*.ail tools/perfbench/programs/*.ail tools/perfbench/agent/*.ail 2>/dev/null) | sort -u)

dump() {
    local dir="$1"
    for p in $progs; do
        k=$(echo "$p" | tr '/' '_')
        "$ail" run compiler mir "$p" > "$dir/$k.mir" 2> "$dir/$k.raw"
        echo "$?" >> "$dir/$k.err"
        # Warnings about the compiler's own source move with its lines.
        grep -E 'error\[' "$dir/$k.raw" >> "$dir/$k.err"
        rm -f "$dir/$k.raw"
    done
    for p in $(ls tools/perfbench/programs/*.ail tools/perfbench/agent/*.ail 2>/dev/null); do
        k=$(echo "$p" | tr '/' '_')
        "$ail" build "$p" -o "$dir/$k.x64" --no-cache > /dev/null 2>&1
        "$ail" build "$p" --target aarch64-linux -o "$dir/$k.a64" --no-cache > /dev/null 2>&1
    done
}

git stash push -u -q -m lower-split-check || exit 2
dump "$out/before"
git stash pop -q || exit 2
dump "$out/after"
if diff -r -q "$out/before" "$out/after"; then
    echo "check.sh: identical: $(ls "$out/after" | wc -l) files"
else
    echo "check.sh: DIFFERENT" >&2
    exit 1
fi
