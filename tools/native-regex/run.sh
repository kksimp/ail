#!/usr/bin/env bash
# Differential tests of native std.regex against `ail run` (lane 232).
#
#   tools/native-regex/run.sh                # the vector file, tests/regex/vectors.txt
#   tools/native-regex/run.sh cases          # the std.regex cases of tests/cases
#   tools/native-regex/run.sh fuzz [N [SEED]]  # N random vectors (default 2,000 from seed 1)
#
# Each mode builds the program with `ail build`, runs it and `ail run` on the
# same input, and compares stdout and the exit code. Until native std.regex
# builds, the build's own problems are printed and the exit status is 3.
# A disagreement leaves the two outputs in $OUT (default: a temporary
# directory that is printed) and exits 1. Measurement only (ADR 0070).
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
mode="${1:-vectors}"
out="${OUT:-$(mktemp -d)}"
mkdir -p "$out"

build() { # SRC EXE
    if ! ./ail build "$1" -o "$2" 2> "$out/build.err"; then
        echo "native build of $1 isn't available yet:"
        head -n 4 "$out/build.err"
        return 3
    fi
}

same() { # SRC EXE ARGS... : the same stdout and exit code from both
    local src="$1" exe="$2"; shift 2
    local a=0 b=0
    ./ail run "$src" "$@" > "$out/run.out" 2> "$out/run.err" || a=$?
    "$exe" "$@" > "$out/nat.out" 2> "$out/nat.err" || b=$?
    if [ "$a" != "$b" ] || ! cmp -s "$out/run.out" "$out/nat.out"; then
        echo "DIFFERS: $src $* (exit $a vs $b); outputs in $out"
        diff "$out/run.out" "$out/nat.out" | head -n 10
        return 1
    fi
}

case "$mode" in
vectors)
    build tools/native-regex/vectors.ail "$out/vectors" || exit $?
    same tools/native-regex/vectors.ail "$out/vectors" tests/regex/vectors.txt || exit 1
    echo "vectors: $(grep -vc '^#\|^$' tests/regex/vectors.txt) lines agree"
    ;;
fuzz)
    n="${2:-2000}"; seed="${3:-1}"
    python3 -I tools/native-regex/gen.py "$n" "$seed" > "$out/fuzz.txt"
    build tools/native-regex/vectors.ail "$out/vectors" || exit $?
    same tools/native-regex/vectors.ail "$out/vectors" "$out/fuzz.txt" || { cp "$out/fuzz.txt" "$out/fuzz-seed-$seed.txt"; exit 1; }
    echo "fuzz: $n vectors from seed $seed agree"
    ;;
cases)
    status=0; n=0
    for src in tests/cases/*regex*.ail; do
        build "$src" "$out/case" > "$out/case.log" 2>&1 || { echo "skip $src ($(sed -n 2p "$out/case.log"))"; status=3; continue; }
        same "$src" "$out/case" || status=1
        n=$((n + 1))
    done
    echo "cases: $n agree"
    exit $status
    ;;
*)
    echo "usage: tools/native-regex/run.sh [vectors | cases | fuzz [N [SEED]]]" >&2
    exit 2
    ;;
esac
