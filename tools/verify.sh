#!/usr/bin/env bash
# Test a pending branch on this machine without touching your checkout (ADR 0085).
#
#   tools/verify.sh BRANCH
#
# Checks out BRANCH into .verify/BRANCH (a temporary git worktree), runs the
# bootstrap's self-tests and the examples there, and prints PASS or FAIL.
# Reuses the private Rust toolchain in .toolchain/ (ADR 0087).
set -euo pipefail

if [ $# -ne 1 ]; then
    echo "usage: tools/verify.sh BRANCH" >&2
    exit 2
fi
branch="$1"
root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
wt="$root/.verify/$branch"

echo "fetching $branch ..."
git -C "$root" fetch --quiet origin "$branch"
if [ -d "$wt" ]; then
    git -C "$root" worktree remove --force "$wt" >/dev/null 2>&1 || rm -rf "$wt"
fi
git -C "$root" worktree prune
mkdir -p "$(dirname "$wt")"
git -C "$root" worktree add --quiet --detach "$wt" "origin/$branch"
echo "testing $(git -C "$wt" log -1 --format='%h %s')"

export AIL_TOOLCHAIN_DIR="$root/.toolchain"
status=0
"$wt/ail" selftest || status=1
for example in "$wt"/examples/*.ail; do
    expected="${example%.ail}.out"
    [ -f "$expected" ] || continue
    if actual="$("$wt/ail" run "$example" 2>&1)" && [ "$actual" = "$(cat "$expected")" ]; then
        echo "ok    examples/$(basename "$example")"
    else
        echo "FAIL  examples/$(basename "$example")"
        echo "      expected: $(cat "$expected")"
        echo "      actual:   $actual"
        status=1
    fi
done

echo
if [ "$status" -eq 0 ]; then
    echo "PASS: $branch on $(uname -sm)"
else
    echo "FAIL: $branch on $(uname -sm). Paste this output back to the AI."
fi
exit "$status"
