#!/usr/bin/env bash
# Interpreter timing (ADR 0175): min and median wall time per program.
#
#   tools/perfbench/run.sh            # 5 runs per program
#   tools/perfbench/run.sh 10         # 10 runs per program
#
# Builds the bootstrap with ./ail, then times `ail run` on each program. If
# python3 is present it also times the matching tokbench corpus program, for
# reference only.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
runs="${1:-5}"
"$root/ail" run "$root/examples/hello.ail" >/dev/null   # builds the bootstrap

programs=("$root/examples/interp.ail" "$root/examples/json.ail" "$root/examples/sheet.ail" "$here/programs/loops.ail")
corpus="$root/tools/tokbench/corpus/programs"

now() { date +%s.%N; }
# stats FILE...: prints "min median" in seconds of the numbers on stdin
stats() { sort -n | awk '{a[NR]=$1} END {printf "%.3f %.3f", a[1], (NR%2 ? a[(NR+1)/2] : (a[NR/2]+a[NR/2+1])/2)}'; }

printf '%-12s %9s %9s %12s\n' program min_s median_s python_s
for p in "${programs[@]}"; do
    name="$(basename "$p" .ail)"
    times=""
    for _ in $(seq "$runs"); do
        t0="$(now)"
        "$root/bootstrap/target/release/ail" run "$p" >/dev/null
        t1="$(now)"
        times+="$(echo "$t1 - $t0" | bc -l)"$'\n'
    done
    read -r mn md <<<"$(printf '%s' "$times" | stats)"
    py="-"
    if command -v python3 >/dev/null 2>&1 && [ -f "$corpus/$name/$name.py" ]; then
        t0="$(now)"; python3 "$corpus/$name/$name.py" >/dev/null 2>&1 || true; t1="$(now)"
        py="$(echo "$t1 - $t0" | bc -l | awk '{printf "%.3f", $1}')"
    fi
    printf '%-12s %9s %9s %12s\n' "$name" "$mn" "$md" "$py"
done
