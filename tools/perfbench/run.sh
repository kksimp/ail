#!/usr/bin/env bash
# Interpreter timing (ADR 0175): min and median wall time per program.
#
#   tools/perfbench/run.sh            # 5 runs per program
#   tools/perfbench/run.sh 10         # 10 runs per program
#
# Builds the bootstrap with ./ail, then times `ail run` on each program. If
# python3 is present it also times the matching Python twin (tokbench corpus or
# programs/NAME.py) and prints ratio = AIL min / Python min (below 1 beats CPython),
# for reference.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
runs="${1:-5}"
"$root/ail" run "$root/examples/hello.ail" >/dev/null   # builds the bootstrap

programs=("$root/examples/interp.ail" "$root/examples/json.ail" "$root/examples/sheet.ail" "$here/programs/loops.ail" "$root/examples/logscan.ail" "$root/examples/tasks.ail" "$root/examples/build.ail" "$here/programs/parsum.ail" "$here/programs/tensor.ail" "$here/programs/tensor_loops.ail" "$here/programs/tensor_ops.ail" "$here/programs/tensor_ops_loops.ail")
corpus="$root/tools/tokbench/corpus/programs"

now() { date +%s.%N; }
# stats FILE...: prints "min median" in seconds of the numbers on stdin
stats() { sort -n | awk '{a[NR]=$1} END {printf "%.3f %.3f", a[1], (NR%2 ? a[(NR+1)/2] : (a[NR/2]+a[NR/2+1])/2)}'; }

printf '%-12s %9s %9s %9s %7s\n' program min_s median_s python_s ratio
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
    py="-"; ratio="-"
    twin="$corpus/$name/$name.py"
    [ -f "$twin" ] || twin="$(dirname "$p")/$name.py"
    if command -v python3 >/dev/null 2>&1 && [ -f "$twin" ]; then
        ptimes=""
        for _ in $(seq "$runs"); do
            t0="$(now)"; python3 "$twin" >/dev/null 2>&1 || true; t1="$(now)"
            ptimes+="$(echo "$t1 - $t0" | bc -l)"$'\n'
        done
        read -r py _ <<<"$(printf '%s' "$ptimes" | stats)"
        ratio="$(echo "$mn / $py" | bc -l | awk '{printf "%.2f", $1}')"
    fi
    printf '%-12s %9s %9s %9s %7s\n' "$name" "$mn" "$md" "$py" "$ratio"
done

# conc.par.8-9: the thread pool on parsum, one thread against the default pool.
one="$(for _ in $(seq "$runs"); do t0="$(now)"; AIL_THREADS=1 "$root/bootstrap/target/release/ail" run "$here/programs/parsum.ail" >/dev/null; t1="$(now)"; echo "$t1 - $t0" | bc -l; done | stats)"
pool="$(for _ in $(seq "$runs"); do t0="$(now)"; "$root/bootstrap/target/release/ail" run "$here/programs/parsum.ail" >/dev/null; t1="$(now)"; echo "$t1 - $t0" | bc -l; done | stats)"
printf 'parsum min_s: 1 thread %s, pool %s\n' "${one%% *}" "${pool%% *}"
