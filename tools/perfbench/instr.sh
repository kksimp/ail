#!/bin/sh
# Instruction counts (callgrind) for each interpreter benchmark: unaffected by
# machine load, so safe to run next to other timings.
#   tools/perfbench/instr.sh OLD_AIL NEW_AIL [OUT.tsv]
# Prints program, CPython, OLD, NEW in millions of instructions. Sizes come from
# NAME.args (programs/) and are the same as the wall-clock race.
set -eu
old=$1; new=$2; out=${3:-/dev/stdout}
here=$(cd "$(dirname "$0")" && pwd)
tmp=$(mktemp -d)
count() { # count CMD... -> millions of instructions
    valgrind --tool=callgrind --callgrind-out-file="$tmp/cg" "$@" >/dev/null 2>"$tmp/err" || true
    sed -n 's/.*I   refs: *//p' "$tmp/err" | tr -d ',' | awk '{printf "%.1f", $1/1e6}'
}
{
printf 'program\tcpython\told\tnew\n'
for f in ${WALL_FILES:-"$here"/programs/*.ail "$here"/agent/*.ail}; do
    name=$(basename "$f" .ail)
    dir=$(dirname "$f")
    case $name in race_smoke|loops|parsum) continue;; esac
    args=""; [ -f "$dir/$name.args" ] && args=$(cat "$dir/$name.args")
    [ -f "$dir/$name.py" ] || continue
    py=$(count python3 -I "$dir/$name.py" $args)
    o=$(count "$old" run "$f" $args)
    n=$(count "$new" run "$f" $args)
    printf '%s\t%s\t%s\t%s\n' "$name" "$py" "$o" "$n"
done
} > "$out"
rm -rf "$tmp"
