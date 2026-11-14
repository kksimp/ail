#!/usr/bin/env bash
# Regenerate docs/benchmarks.md (the public speed and token tables) in one command:
#
#   tools/bench/publish.sh                 # 5 timed runs per cell, writes docs/benchmarks.md
#   tools/bench/publish.sh --runs 11       # more runs
#   tools/bench/publish.sh --out FILE      # write somewhere else
#   tools/bench/publish.sh --latency-json F.json   # also show another host's tools/bench/latency.py run (repeatable)
#   tools/bench/publish.sh --fresh-clone   # add the first-run time in a new clone to the edit-loop table
#   tools/bench/publish.sh --keep-dir DIR  # also keep the raw JSON (races, tokens, latency) and the environment block
#   tools/bench/publish.sh --skeleton      # no measuring: the page with every number as PENDING
#   tools/bench/publish.sh --host-dir DIR  # add another host's --keep-dir results (speed tables, latency; repeatable)
#   tools/bench/publish.sh --render-only DIR --host-dir D2 --host-dir D3   # no measuring: the page from kept directories
#
# Needs a quiet machine (no other build or benchmark running), gcc, clang, rustc, go and python3
# (measurement only, ADR 0070). It builds AIL with ./ail, times `ail build` output against
# gcc -O2, clang -O2, rustc -C opt-level=3, go build and CPython, each small program at a size where gcc -O2 takes at least 0.3 s (drag.size.1) (tools/perfbench/race.py), counts tokens with tools/tokbench,
# measures the edit loop (`ail check`, `test`, `context`, `build`, warm `ail mcp`; tools/bench/latency.py),
# and fills tools/bench/benchmarks.tmpl.md. The environment block records host, commit and tool versions.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
runs=5; out="$root/docs/benchmarks.md"; extra=""; fresh=""; skeleton=""; keep=""; hostdirs=""; render=""
while [ $# -gt 0 ]; do
    case "$1" in
        --runs) runs="$2"; shift 2 ;;
        --out) out="$2"; shift 2 ;;
        --latency-json) extra="$extra --latency $2"; shift 2 ;;
        --fresh-clone) fresh="--fresh-clone"; shift ;;
        --keep-dir) keep="$2"; shift 2 ;;
        --skeleton) skeleton=1; shift ;;
        --host-dir) hostdirs="$hostdirs --host $2"; shift 2 ;;
        --render-only) render="$2"; shift 2 ;;
        *) echo "usage: tools/bench/publish.sh [--runs N] [--out FILE] [--latency-json F.json]... [--host-dir DIR]... [--fresh-clone] [--keep-dir DIR] [--render-only DIR] [--skeleton]" >&2; exit 2 ;;
    esac
done
cd "$root"
# Git for Windows' bash: `python3` may be the Store stub, so take the first Python that runs; the host label is Windows.
py=""
for cand in python3 python; do
    if command -v "$cand" >/dev/null 2>&1 && "$cand" -c 'import sys' >/dev/null 2>&1; then py="$cand"; break; fi
done
case "${OSTYPE:-}" in
    msys* | cygwin*) host="Windows $(uname -m)" ;;
    *) host="$(uname -sm)" ;;
esac
if [ -n "$skeleton" ]; then
    "$py" -I tools/bench/publish.py --skeleton tools/bench/benchmarks.tmpl.md "$out"
    echo "wrote $out (skeleton: no numbers)"
    exit 0
fi
if [ -n "$render" ]; then
    # shellcheck disable=SC2086
    "$py" -I tools/bench/publish.py --latency "$render/latency.json"$extra$hostdirs "$render/small.json" "$render/big.json" \
        "$render/tokens.json" tools/bench/benchmarks.tmpl.md "$out" "$render/env.txt"
    echo "wrote $out (from $render)"
    exit 0
fi
missing=""
for tool in gcc clang rustc go; do
    command -v "$tool" >/dev/null 2>&1 || missing="$missing $tool"
done
[ -n "$py" ] || missing="$missing python3"
if [ -n "$missing" ]; then
    echo "error: missing${missing}; install them (macOS: xcode-select --install, rustup for rustc and go from go.dev; Windows: MinGW-w64 gcc, LLVM clang, rustup and Go; Debian/Ubuntu: sudo apt install gcc clang rustc golang-go python3 python3-venv)" >&2
    exit 1
fi
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
./ail run examples/hello.ail >/dev/null            # builds the native CLI from the seed
small=""; for n in collatz nbody spectralnorm mandelbrot fannkuch binarytrees fasta wordfreq matmul sieve; do
    small="$small tools/perfbench/programs/$n.ail"; done
bigp=""; for n in lzr vm raytrace matmul1400; do bigp="$bigp tools/perfbench/big/$n.ail"; done
# shellcheck disable=SC2086
"$py" -I tools/perfbench/race.py --no-run --native --c --rust --go --python --runs "$runs" --timeout 900 --calibrate 0.3 \
    --json "$tmp/small.json" --programs $small >/dev/null
# shellcheck disable=SC2086
"$py" -I tools/perfbench/race.py --no-run --native --c --rust --go --runs "$runs" --timeout 900 \
    --json "$tmp/big.json" --programs $bigp >/dev/null
# shellcheck disable=SC2086
"$py" -I tools/bench/latency.py --runs "$runs" --label "$host" $fresh --out "$tmp/latency.json" >&2
files=""; for n in collatz nbody spectralnorm mandelbrot fannkuch binarytrees fasta wordfreq matmul sieve; do
    for e in ail c rs go py; do files="$files tools/perfbench/programs/$n.$e"; done; done
# shellcheck disable=SC2086
tools/tokbench/run.sh --json $files > "$tmp/tokens.json"
{
    echo "- date: $(date -u +%Y-%m-%d)"
    echo "- AIL commit: $(git rev-parse --short HEAD) ($(./ail --version 2>/dev/null | head -1 || true))"
    echo "- host: $host, $(nproc 2>/dev/null || sysctl -n hw.ncpu) logical cores, $( (grep -m1 'model name' /proc/cpuinfo | cut -d: -f2) 2>/dev/null || sysctl -n machdep.cpu.brand_string)"
    echo "- gcc: $(gcc --version | head -1)"
    echo "- clang: $(clang --version | head -1)"
    echo "- rustc: $(rustc --version)"
    echo "- go: $(go version)"
    echo "- python: $("$py" --version 2>&1)"
    echo "- runs per cell: $runs (best of; wall seconds)"
} > "$tmp/env.txt"
# shellcheck disable=SC2086
"$py" -I tools/bench/publish.py --latency "$tmp/latency.json"$extra$hostdirs "$tmp/small.json" "$tmp/big.json" "$tmp/tokens.json" \
    tools/bench/benchmarks.tmpl.md "$out" "$tmp/env.txt"
if [ -n "$keep" ]; then
    mkdir -p "$keep"; cp "$tmp/small.json" "$tmp/big.json" "$tmp/tokens.json" "$tmp/latency.json" "$tmp/env.txt" "$keep/"
fi
echo "wrote $out"
