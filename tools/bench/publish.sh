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
#
# Needs a quiet machine (no other build or benchmark running), gcc and clang, and python3
# (measurement only, ADR 0070). It builds AIL with ./ail, times `ail build` output against
# gcc -O2, clang -O2 and CPython, each small program at a size where gcc -O2 takes at least 0.3 s (drag.size.1) (tools/perfbench/race.py), counts tokens with tools/tokbench,
# measures the edit loop (`ail check`, `test`, `context`, `build`, warm `ail mcp`; tools/bench/latency.py),
# and fills tools/bench/benchmarks.tmpl.md. The environment block records host, commit and tool versions.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
runs=5; out="$root/docs/benchmarks.md"; extra=""; fresh=""; skeleton=""; keep=""
while [ $# -gt 0 ]; do
    case "$1" in
        --runs) runs="$2"; shift 2 ;;
        --out) out="$2"; shift 2 ;;
        --latency-json) extra="$extra --latency $2"; shift 2 ;;
        --fresh-clone) fresh="--fresh-clone"; shift ;;
        --keep-dir) keep="$2"; shift 2 ;;
        --skeleton) skeleton=1; shift ;;
        *) echo "usage: tools/bench/publish.sh [--runs N] [--out FILE] [--latency-json F.json]... [--fresh-clone] [--keep-dir DIR] [--skeleton]" >&2; exit 2 ;;
    esac
done
cd "$root"
if [ -n "$skeleton" ]; then
    python3 -I tools/bench/publish.py --skeleton tools/bench/benchmarks.tmpl.md "$out"
    echo "wrote $out (skeleton: no numbers)"
    exit 0
fi
missing=""
for tool in gcc clang python3; do
    command -v "$tool" >/dev/null 2>&1 || missing="$missing $tool"
done
if [ -n "$missing" ]; then
    echo "error: missing${missing}; install them (macOS: xcode-select --install; Debian/Ubuntu: sudo apt install gcc clang python3 python3-venv)" >&2
    exit 1
fi
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
./ail run examples/hello.ail >/dev/null            # builds the native CLI from the seed
small=""; for n in collatz nbody spectralnorm mandelbrot fannkuch binarytrees fasta wordfreq matmul sieve; do
    small="$small tools/perfbench/programs/$n.ail"; done
bigp=""; for n in lzr vm raytrace matmul1400; do bigp="$bigp tools/perfbench/big/$n.ail"; done
# shellcheck disable=SC2086
python3 -I tools/perfbench/race.py --no-run --native --c --python --runs "$runs" --timeout 900 --calibrate 0.3 \
    --json "$tmp/small.json" --programs $small >/dev/null
# shellcheck disable=SC2086
python3 -I tools/perfbench/race.py --no-run --native --c --runs "$runs" --timeout 900 \
    --json "$tmp/big.json" --programs $bigp >/dev/null
# shellcheck disable=SC2086
python3 -I tools/bench/latency.py --runs "$runs" --label "$(uname -sm)" $fresh --out "$tmp/latency.json" >&2
files=""; for n in collatz nbody spectralnorm mandelbrot fannkuch binarytrees fasta wordfreq matmul sieve; do
    for e in ail c py; do files="$files tools/perfbench/programs/$n.$e"; done; done
# shellcheck disable=SC2086
tools/tokbench/run.sh --json $files > "$tmp/tokens.json"
{
    echo "- date: $(date -u +%Y-%m-%d)"
    echo "- AIL commit: $(git rev-parse --short HEAD) ($(./ail --version 2>/dev/null | head -1 || true))"
    echo "- host: $(uname -sm), $(nproc 2>/dev/null || sysctl -n hw.ncpu) logical cores, $( (grep -m1 'model name' /proc/cpuinfo | cut -d: -f2) 2>/dev/null || sysctl -n machdep.cpu.brand_string)"
    echo "- gcc: $(gcc --version | head -1)"
    echo "- clang: $(clang --version | head -1)"
    echo "- python: $(python3 --version)"
    echo "- runs per cell: $runs (best of; wall seconds)"
} > "$tmp/env.txt"
# shellcheck disable=SC2086
python3 -I tools/bench/publish.py --latency "$tmp/latency.json"$extra "$tmp/small.json" "$tmp/big.json" "$tmp/tokens.json" \
    tools/bench/benchmarks.tmpl.md "$out" "$tmp/env.txt"
if [ -n "$keep" ]; then
    mkdir -p "$keep"; cp "$tmp/small.json" "$tmp/big.json" "$tmp/tokens.json" "$tmp/latency.json" "$tmp/env.txt" "$keep/"
fi
echo "wrote $out"
