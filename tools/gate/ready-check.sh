#!/bin/bash
# What a lane runs on a Kaleb machine before it posts READY: the cheap checks batches kept failing on
# (item 251; fmt broke CC, CD, CF and CG). Only the files the branch changes against BASE, so it takes seconds.
#   tools/gate/ready-check.sh [BASE]      from the repository root; BASE defaults to origin/main
# Prints only what failed; the last line is `READY-CHECK GREEN` or `READY-CHECK RED (N)`.
set -u
cd "$(git rev-parse --show-toplevel)"
# ADR 1130: `ail check` here must not apply its certain fixes to wrong-on-purpose fixtures.
export AIL_FIX=0
base=${1:-origin/main}
git fetch -q origin "${base#origin/}" 2>/dev/null
files=$(git diff --name-only --diff-filter=AM "$(git merge-base "$base" HEAD)" HEAD)
fails=0
red() { fails=$((fails+1)); echo "RED $1"; }
ail=$(echo "$files" | grep '\.ail$' | grep -v '^tests/' || true)
if [ -n "$ail" ]; then
  # shellcheck disable=SC2086
  ./ail fmt --check $ail >"${TMPDIR:-/tmp}/ready-fmt.$$" 2>&1 || { red "fmt: run ./ail fmt on"; grep '^--- ' "${TMPDIR:-/tmp}/ready-fmt.$$" | cut -c5-; }
  rm -f "${TMPDIR:-/tmp}/ready-fmt.$$"
  case "$ail" in *compiler/*) ./ail check compiler >/dev/null 2>&1 || red "./ail check compiler";; esac
fi
for f in $(echo "$files" | grep -E '\.sh$|^ail$' || true); do bash -n "$f" 2>/dev/null || red "bash -n $f"; done
# Conflict markers a merge of main left behind.
[ -n "$files" ] && git grep -n "^<<<<<<< \|^>>>>>>> " -- $files 2>/dev/null | head -3 | grep . && red "conflict markers"
[ $fails = 0 ] && echo "READY-CHECK GREEN" || echo "READY-CHECK RED ($fails)"
[ $fails = 0 ]
