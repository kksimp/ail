#!/usr/bin/env bash
# Std-module benchmark (AIL std against Python, Go, C/zlib and Rust std). Runs only on the designer's
# machines, on a quiet host (load below 3, no gate running). Needs gcc, rustc, go and python3 >= 3.10
# (on the Mac use Homebrew python). Measurement only (ADR 0070).
#
#   tools/bench/std/run.sh                       # every program, 5 runs, writes std-results.md and .json here
#   tools/bench/std/run.sh --runs 3 json_parse   # chosen programs
#   tools/bench/std/run.sh --scale 0.1           # a smoke run at a tenth of the sizes
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$root"
out="${STD_BENCH_OUT:-$root/tools/bench/std/out}"
mkdir -p "$out"
py=python3
for cand in /opt/homebrew/bin/python3 python3; do
    if command -v "$cand" >/dev/null 2>&1 && "$cand" -c 'import sys; sys.exit(sys.version_info < (3, 10))' 2>/dev/null; then py="$cand"; break; fi
done
"$root/ail" build examples/hello.ail -o "$out/hello.bin" >/dev/null
"$py" -I tools/bench/std/std.py --md "$out/std-results.md" --json "$out/std-results.json" "$@"
echo "wrote $out/std-results.md"
