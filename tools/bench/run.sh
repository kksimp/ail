#!/usr/bin/env bash
# Before/after benchmark table (ADR 0492): tokens and speed for the examples,
# the agent-tool corpus and tools/perfbench programs.
#
#   tools/bench/run.sh                    # before = origin/main, 5 runs
#   tools/bench/run.sh --before REV       # any git revision
#   tools/bench/run.sh --runs 10          # more timing runs per program
#   tools/bench/run.sh --json             # machine-readable output
#
# Builds the bootstrap at REV (in a temporary git worktree, removed on exit) and
# at the working tree, then prints one table. Python and C twins are timed when
# present; the C twins need a C compiler (cc). Rust comes from the private
# toolchain in .toolchain/ (ADR 0087). Uses the Python from tools/tokbench.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

before=""
runs=5
format="table"
while [ $# -gt 0 ]; do
    case "$1" in
        --before) [ $# -ge 2 ] || { echo "bench: --before needs a revision" >&2; exit 2; }; before="$2"; shift 2 ;;
        --runs) [ $# -ge 2 ] || { echo "bench: --runs needs a number" >&2; exit 2; }; runs="$2"; shift 2 ;;
        --json) format="json"; shift ;;
        -h | --help) sed -n '2,13p' "$0"; exit 0 ;;
        *) echo "usage: tools/bench/run.sh [--before REV] [--runs N] [--json]" >&2; exit 2 ;;
    esac
done

if [ -z "$before" ]; then
    # origin/main, not a local main (which can be far behind); refresh it first.
    git -C "$root" fetch -q origin main 2>/dev/null || echo "bench: could not fetch origin/main; using the last fetched copy" >&2
    before=origin/main
fi
git -C "$root" rev-parse --verify -q "$before^{commit}" >/dev/null || {
    echo "bench: unknown revision: $before" >&2; exit 2; }

export AIL_TOOLCHAIN_DIR="${AIL_TOOLCHAIN_DIR:-$root/.toolchain}"
scratch="$(mktemp -d)"
wt="$scratch/before"
cleanup() {
    git -C "$root" worktree remove --force "$wt" >/dev/null 2>&1 || true
    git -C "$root" worktree prune >/dev/null 2>&1 || true
    rm -rf "$scratch"
}
trap cleanup EXIT

echo "bench: building $before ..." >&2
git -C "$root" worktree add --quiet --detach "$wt" "$before"
"$wt/ail" run "$wt/examples/hello.ail" >/dev/null

echo "bench: building the working tree ..." >&2
"$root/ail" run "$root/examples/hello.ail" >/dev/null

after_label="$(git -C "$root" rev-parse --short HEAD)"
git -C "$root" diff --quiet HEAD -- || after_label="$after_label+uncommitted"
before_label="$(git -C "$wt" rev-parse --short HEAD) ($before, $(git -C "$wt" log -1 --format=%s | cut -c1-60))"

python3 "$root/tools/bench/bench.py" \
    --root "$root" \
    --before-root "$wt" \
    --before-label "$before_label" \
    --after-label "$after_label" \
    --tokbench "$root/tools/tokbench/run.sh" \
    --runs "$runs" \
    --format "$format" \
    --card-ail "$root/docs/language-card.md" \
    --card-py "$root/tools/tokbench/reference/python-reference.md"
