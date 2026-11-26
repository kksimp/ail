#!/usr/bin/env bash
# The demo: an agent resolves a real issue (GitHub #163) on its own, with `ail task`, and the run is recorded.
#   tools/agentbench/demo/run.sh [MODEL]      from the repository root, on a designer machine
# CALLS A PAID MODEL (`claude -p`, or $AGENTBENCH_AGENT): run it only with the designer's go-ahead (ADR 0916).
# The agent works in a scratch git worktree of HEAD, so nothing in this checkout changes, and it may only use
# `./ail`, `git diff|status|log` and the file tools (no other shell commands, no permission bypass). Output,
# untracked, in tools/agentbench/demo/runs/STAMP/: transcript.jsonl (everything the agent did), summary.md (its
# commands, turns, tokens, cost), changes.diff (what it changed) and verdict.txt (a check the agent doesn't control).
set -u
root=$(git rev-parse --show-toplevel); cd "$root" || exit 1
model=${1:-${AGENTBENCH_MODEL:-}}
[ -n "$model" ] || { echo "usage: tools/agentbench/demo/run.sh MODEL (or set AGENTBENCH_MODEL)" >&2; exit 2; }
out=${DEMO_OUT:-$root/tools/agentbench/demo/runs/$(date -u +%Y%m%dT%H%M%SZ)}
mkdir -p "$out"
tmp=$(mktemp -d); wt=$tmp/ail
git worktree add -q --detach "$wt" HEAD || exit 1
trap 'git worktree remove --force "$wt" 2>/dev/null; rm -rf "$tmp"' EXIT
cp tools/agentbench/demo/issue163.md "$wt/ISSUE.md"
prompt='ISSUE.md is a bug report from before the interpreter was removed. Decide whether it still reproduces with the native compiler. Work with `./ail task` (see docs/quick-reference.md): `task start`, edit, `task verify` until green, `task finish`. If it does not reproduce, add a regression test that would have caught it: tests/native/run/NAME.ail and its expected output tests/native/golden/NAME.out. If it does, fix it in compiler/ and add the same test. Change nothing else. End with one sentence: what you found and what you changed.'
agent=${AGENTBENCH_AGENT:-claude}
# shellcheck disable=SC2086
( cd "$wt" && printf '%s' "$prompt" | $agent -p --model "$model" --output-format stream-json --verbose \
    --permission-mode acceptEdits --allowedTools "Bash(./ail:*)" "Bash(git diff:*)" "Bash(git status:*)" "Bash(git log:*)" Read Edit Write Grep Glob \
    --max-turns 40 ) >"$out/transcript.jsonl" 2>"$out/agent.err"
python3 -I tools/agentbench/demo/summarize.py "$out/transcript.jsonl" >"$out/summary.md"
( cd "$wt" && git add -A && git diff --cached ) >"$out/changes.diff"
# The check the agent doesn't control: every new native test runs to exactly its golden output.
{
  new=$(cd "$wt" && git diff --cached --name-only --diff-filter=A | grep '^tests/native/run/.*\.ail$')
  if [ -z "$new" ]; then echo "FAIL: no new test under tests/native/run/"; fi
  for t in $new; do
    g=tests/native/golden/$(basename "$t" .ail).out
    if [ ! -f "$wt/$g" ]; then echo "FAIL: $t has no golden $g"; continue; fi
    if (cd "$wt" && ./ail run "$t" 2>&1 | cmp -s - "$g"); then echo "PASS: $t matches $g"; else echo "FAIL: $t differs from $g"; fi
  done
} >"$out/verdict.txt"
cat "$out/summary.md" "$out/verdict.txt"
echo "recorded in $out"
