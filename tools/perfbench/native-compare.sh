#!/bin/bash
# Native wall-clock comparison of two commits on the ten native-vs-C programs.
#   bash tools/perfbench/native-compare.sh OLD_REF NEW_REF [RUNS]   
# For PR #482 (allocator moves): bash native-compare.sh 9e33d2a2 7cbadb47
# Builds each program with `./ail build` in a temporary worktree per ref and
# prints the median of RUNS runs (default 7), in milliseconds. Run on a quiet
# machine from the repository root; cleans up after itself. Works on macOS
# ARM64 and Linux (no valgrind needed). Sizes: the perfbench .args, with
# fannkuch 10 and matmul 400 so the runs are long enough to time.
set -eu
old=$1; new=$2; n=${3:-7}
root=$(git rev-parse --show-toplevel)
tmp=$(mktemp -d)
trap 'git -C "$root" worktree remove --force "$tmp/old" 2>/dev/null; git -C "$root" worktree remove --force "$tmp/new" 2>/dev/null; rm -rf "$tmp"' EXIT
git -C "$root" worktree add -q "$tmp/old" "$old"
git -C "$root" worktree add -q "$tmp/new" "$new"
progs="mandelbrot collatz spectralnorm nbody sieve matmul fannkuch binarytrees fasta wordfreq"
now_ms() { python3 -c 'import time; print(int(time.time()*1000))'; }
med() { # med BIN ARGS... -> median ms
    ts=""; i=0
    while [ $i -lt "$n" ]; do
        s=$(now_ms); "$@" >/dev/null 2>&1 || true; e=$(now_ms)
        ts="$ts $((e - s))"; i=$((i+1))
    done
    echo $ts | tr ' ' '\n' | sort -n | awk '{a[NR]=$1} END{print a[int((NR+1)/2)]}'
}
for t in old new; do
    for p in $progs; do
        (cd "$tmp/$t" && ./ail build tools/perfbench/programs/$p.ail -o "$tmp/$t.$p" >/dev/null 2>&1) || echo "build $t $p failed" >&2
    done
done
printf '%-13s %8s %8s %8s\n' program "old ms" "new ms" change
for p in $progs; do
    a=$(cat "$root/tools/perfbench/programs/$p.args" 2>/dev/null || true)
    [ "$p" = fannkuch ] && a=10
    [ "$p" = matmul ] && a=400
    o=$(med "$tmp/old.$p" $a); w=$(med "$tmp/new.$p" $a)
    printf '%-13s %8s %8s %7s%%\n' "$p" "$o" "$w" "$(awk -v o="$o" -v w="$w" 'BEGIN{printf "%+.1f", (o>0)?(w-o)*100/o:0}')"
done
