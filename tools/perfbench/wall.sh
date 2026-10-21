#!/bin/sh
# Median wall-clock seconds of N runs (default 5) per interpreter benchmark.
#   tools/perfbench/wall.sh OLD_AIL NEW_AIL [N]
# Prints program, CPython, OLD, NEW. Run on a quiet machine.
set -eu
old=$1; new=$2; n=${3:-5}
here=$(cd "$(dirname "$0")" && pwd)
med() { # med CMD... -> median seconds
    i=0; ts=""
    while [ $i -lt "$n" ]; do
        s=$(date +%s.%N); "$@" >/dev/null 2>&1 || true; e=$(date +%s.%N)
        ts="$ts $(echo "$e - $s" | bc -l)"; i=$((i+1))
    done
    echo $ts | tr ' ' '\n' | sort -n | awk '{a[NR]=$1} END{printf "%.3f", a[int((NR+1)/2)]}'
}
printf 'program\tcpython\told\tnew\n'
for f in ${WALL_FILES:-"$here"/programs/*.ail "$here"/agent/*.ail}; do
    name=$(basename "$f" .ail); dir=$(dirname "$f")
    case $name in race_smoke|loops|parsum) continue;; esac
    args=""; [ -f "$dir/$name.args" ] && args=$(cat "$dir/$name.args")
    [ -f "$dir/$name.py" ] || continue
    printf '%s\t%s\t%s\t%s\n' "$name" "$(med python3 -I "$dir/$name.py" $args)" "$(med "$old" run "$f" $args)" "$(med "$new" run "$f" $args)"
done
