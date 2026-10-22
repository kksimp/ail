#!/usr/bin/env bash
# Native vs `ail run` (and NumPy when installed) for the tensor kernels (ADR 0815).
# Run from the repo root on the branch under test:
#   bash tools/perfbench/tensor-bench.sh [RUNS]     # default 5
# Programs: the repo's tools/perfbench/numpy f32_chain and f32_matmul and
# programs/tensor, tensor_ops, tensor_loops; plus sigmoid and softmax_sum below.
# Prints the minimum wall seconds of RUNS per column and whether outputs agree.
set -euo pipefail
runs="${1:-5}"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
cat > "$tmp/sigmoid.ail" <<'AIL'
fn main()
    n := 1024
    x := tensor.from([((i * 7919) % 1009).to_float() * 0.01 - 5.0 for i in 0..n * n], [n, n])
    mut total := 0.0
    for r in 0..20
        total += x.sigmoid().sum()
    print((total * 1000.0).round() / 1000.0)
AIL
cat > "$tmp/softmax_sum.ail" <<'AIL'
fn main()
    n := 512
    x := tensor.from([((i * 7919) % 1009).to_float() * 0.01 for i in 0..n * n], [n, n])
    mut total := 0.0
    for r in 0..20
        total += x.softmax().sum()
    print((total * 1000.0).round() / 1000.0)
AIL
best() { python3 - "$runs" "$@" <<'PY'
import subprocess, sys, time
n = int(sys.argv[1]); cmd = sys.argv[2:]; m = 1e9
for _ in range(n):
    s = time.perf_counter(); subprocess.run(cmd, stdout=subprocess.DEVNULL, check=True); m = min(m, time.perf_counter() - s)
print(f"{m:.3f}")
PY
}
progs="tools/perfbench/numpy/f32_chain.ail tools/perfbench/numpy/f32_matmul.ail tools/perfbench/programs/tensor.ail tools/perfbench/programs/tensor_ops.ail tools/perfbench/programs/tensor_loops.ail $tmp/sigmoid.ail $tmp/softmax_sum.ail"
printf '%-14s %9s %9s %8s %s\n' program native_s ail_run_s ratio same_output
for f in $progs; do
  p="$(basename "$f" .ail)"
  ./ail build "$f" -o "$tmp/$p.exe" >/dev/null 2>&1 || { printf '%-14s %s\n' "$p" "does not build natively"; continue; }
  a=$("$tmp/$p.exe"); b=$(./ail run "$f")
  same=yes; [ "$a" = "$b" ] || same=NO
  tn=$(best "$tmp/$p.exe"); ti=$(best ./ail run "$f")
  printf '%-14s %9s %9s %8s %s\n' "$p" "$tn" "$ti" "$(python3 -c "print(f'{float(\"$ti\")/max(float(\"$tn\"),1e-9):.2f}x')")" "$same"
done
