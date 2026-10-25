#!/usr/bin/env bash
# Native AIL against plain C (-O2 and -O3; Accelerate for matmul on macOS) on the tensor programs
# in this directory: wall seconds, min of RUNS. ail_1t is the same binary with AIL_DETERMINISTIC=1 (one
# thread); ail_native uses every core. Outputs are compared as numbers first.
#   tools/perfbench/tensor-c/race.sh [RUNS]    # default 5
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
runs="${1:-5}"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
best() { python3 -I -c '
import subprocess,sys,time
b=1e9
for _ in range(int(sys.argv[1])):
    t=time.perf_counter(); subprocess.run(sys.argv[2:],stdout=subprocess.DEVNULL,check=True); b=min(b,time.perf_counter()-t)
print("%.3f"%b)' "$runs" "$@"; }
same() { python3 -I -c '
import sys
a=[float(x) for x in open(sys.argv[1]).read().split()]; b=[float(x) for x in open(sys.argv[2]).read().split()]
sys.exit(0 if len(a)==len(b) and all(abs(x-y)<=1e-9*max(1,abs(x)) for x,y in zip(a,b)) else 1)' "$1" "$2"; }
printf '%-11s %9s %8s %8s %8s %8s\n' program ail_native ail_1t c_O2 c_O3 c_blas
for p in f32_matmul f32_chain softmax; do
  "$root/ail" build "$here/$p.ail" -o "$tmp/$p.ail.bin" >/dev/null
  cc -O2 -o "$tmp/$p.o2" "$here/$p.c" -lm; cc -O3 -o "$tmp/$p.o3" "$here/$p.c" -lm
  "$tmp/$p.ail.bin" > "$tmp/a.out"; "$tmp/$p.o2" > "$tmp/c.out"
  same "$tmp/a.out" "$tmp/c.out" || { echo "$p: outputs differ" >&2; cat "$tmp/a.out" "$tmp/c.out" >&2; exit 1; }
  blas=-
  if [ "$p" = f32_matmul ] && [ "$(uname)" = Darwin ]; then
    cc -O3 -DBLAS -o "$tmp/$p.blas" "$here/$p.c" -framework Accelerate && blas=$(best "$tmp/$p.blas")
  fi
  printf '%-11s %9s %8s %8s %8s %8s\n' "$p" "$(best "$tmp/$p.ail.bin")" "$(best env AIL_DETERMINISTIC=1 "$tmp/$p.ail.bin")" "$(best "$tmp/$p.o2")" "$(best "$tmp/$p.o3")" "$blas"
done
