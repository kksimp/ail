#!/usr/bin/env bash
# AIL against NumPy on the same tensor work: wall time (min of RUNS) and
# tokens (five-tokenizer mean from tools/tokbench). Run from anywhere:
#   tools/perfbench/numpy/run.sh [RUNS]      # default 5
# Needs python3 with numpy. `ail run` includes the check and startup, as in
# tools/perfbench/README.md. The native column says "no" while
# `ail build NAME.ail` still reports native.subset.1 (std.tensor is lane 174's).
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
runs="${1:-5}"
python3 -c 'import numpy' 2>/dev/null || { echo "numpy is missing: pip install numpy" >&2; exit 1; }
# NumPy's import cost, which numpy_s includes, as a baseline for reading the table.
imp=$(best_import=999999; for _ in $(seq "$runs"); do s=$(date +%s.%N); python3 -I -c 'import numpy'; e=$(date +%s.%N); t=$(echo "$e - $s" | bc -l); best_import=$(echo "if ($t < $best_import) $t else $best_import" | bc -l); done; printf '%.3f' "$best_import")
echo "numpy import alone: ${imp} s (included in numpy_s)"
printf '%-11s %8s %8s %8s %7s %7s %7s %-6s\n' program ail_s numpy_s ail/numpy tok_ail tok_py tok_r native
for p in f32_chain f32_matmul softmax; do
  best() { # best seconds of RUNS for the given command
    local m=999999 s e t
    for _ in $(seq "$runs"); do
      s=$(date +%s.%N); "$@" >/dev/null; e=$(date +%s.%N)
      t=$(echo "$e - $s" | bc -l); m=$(echo "if ($t < $m) $t else $m" | bc -l)
    done
    printf '%.3f' "$m"
  }
  a=$("$root/ail" run "$here/$p.ail"); b=$(python3 -I "$here/$p.py")
  [ "$a" = "$b" ] || { echo "$p: outputs differ: ail=[$a] numpy=[$b]" >&2; exit 1; }
  ta=$(best "$root/ail" run "$here/$p.ail"); tp=$(best python3 -I "$here/$p.py")
  tok=$("$root/tools/tokbench/run.sh" "$here/$p.ail" "$here/$p.py" | awk '/\.ail /{a=$NF} /\.py /{b=$NF} END{printf "%s %s", a, b}')
  set -- $tok
  if "$root/ail" build "$here/$p.ail" -o /dev/null >/dev/null 2>&1; then nat=yes; else nat=no; fi
  printf '%-11s %8s %8s %8.2f %7s %7s %7.2f %-6s\n' "$p" "$ta" "$tp" "$(echo "$ta / $tp" | bc -l)" "$1" "$2" "$(echo "$1 / $2" | bc -l)" "$nat"
done
