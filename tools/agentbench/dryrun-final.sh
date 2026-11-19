#!/usr/bin/env bash
# Dry run of plans/final.txt with the fake agent: no model, no API, no `claude`, no spend (the fake agent bills $0.01 a trial
# on a scratch ledger). Needs `ail` and `cc`, so run it on a designer machine (ADR 0842). Bash 3.2 safe. Takes 15 to 40 minutes
# (150 fake trials, each a build and a test). Usage, from a checkout of the branch: bash tools/agentbench/dryrun-final.sh
# It runs the first five rounds of the plan (150 trials; the sealed tasks stand in as copies of depgraph, ledger and stock)
# in three passes and checks each outcome:
#   1. estimate set to $0.001: the harness stop after round 2 must fire (exit 5, "stage 2 (harness)");
#   2. real estimate, `--ignore-stage 2`, futility limit -1: the futility stop after round 4 must fire (exit 5, "stage 4 (futility)");
#   3. both stops ignored: the batch completes round 5, then `report` must print the pooled line and the target verdict;
#   plus: the plan has 464 trials. Not covered here: the CLI pin (the fake agent reports "fake") and the no-DOCS check (the fake agent reads none).
# Last line: DONE GREEN or DONE RED.
HERE=$(cd "$(dirname "$0")" && pwd); AB="$HERE/agentbench.py"
W=$(mktemp -d); export TMPDIR=$W; trap 'rm -rf "$W"' EXIT
export AGENTBENCH_AGENT="$HERE/fake/agent.sh" AGENTBENCH_RESULTS="$W/results" AGENTBENCH_WS_ROOT="$W" ANTHROPIC_API_KEY=unused
export AGENTBENCH_HELDOUT="$W/heldout"; mkdir -p "$AGENTBENCH_HELDOUT"
red=0; fail() { echo "FAIL: $*"; red=1; }
echo "# dryrun-final: $(uname -sm) $(date -u +%FT%TZ) tree $(git -C "$HERE" rev-parse --short HEAD 2>/dev/null)"
# sealed1..4 as copies of existing from-scratch tasks, marked held out
i=0; for src in depgraph ledger stock depgraph; do
  i=$((i+1)); d="$AGENTBENCH_HELDOUT/sealed$i"; cp -R "$HERE/tasks/$src" "$d"
  python3 -I - "$d/task.json" "sealed$i" <<'PY'
import json, sys
p, n = sys.argv[1:]; t = json.load(open(p)); t.setdefault("exe", t["id"]); t["id"] = n; t["heldout"] = True; json.dump(t, open(p, "w"), indent=1)
PY
done
n=$(grep -vc '^[#@]' "$HERE/plans/final.txt"); [ "$n" = 464 ] || fail "plan has $n trials, not 464"
# the real sealed tasks map through names.txt (AGENTBENCH_HELDOUT/names.txt); the stand-ins keep their own ids
# the first five rounds only
P="$W/p5.txt"; { grep '^@' "$HERE/plans/final.txt" | grep -v '^@stage'; echo "@stage 2 harness"; echo "@stage 4 futility 0.25"
  grep -v '^[#@]' "$HERE/plans/final.txt" | awk '$3 <= 5'; } > "$P"
echo "rows: $(grep -vc '^[@#]' "$P")"
run() { python3 -I "$AB" batch "$@" --model fake --max-usd 500 2>&1; }
echo "== pass 1: harness stop (estimate \$0.001)"
sed 's/^@estimate-usd .*/@estimate-usd 0.001/' "$P" > "$W/p1.txt"
out=$(run "$W/p1.txt"); rc=$?; echo "$out" | tail -4
[ $rc = 5 ] && echo "$out" | grep -q "stage 2 (harness) stop" || fail "pass 1: expected exit 5 with the stage 2 harness stop, got $rc"
echo "trials done: $(find "$W/results" -name metrics.json | wc -l | tr -d ' ') (expect 60)"
echo "== pass 2: futility stop (limit -1)"
sed 's/^@stage 4 futility .*/@stage 4 futility -1/' "$P" > "$W/p2.txt"
out=$(run "$W/p2.txt" --ignore-stage 2); rc=$?; echo "$out" | tail -4
[ $rc = 5 ] && echo "$out" | grep -q "stage 4 (futility) stop" || fail "pass 2: expected exit 5 with the stage 4 futility stop, got $rc"
echo "trials done: $(find "$W/results" -name metrics.json | wc -l | tr -d ' ') (expect 120)"
echo "== pass 3: stops ignored, round 5 completes"
out=$(run "$P" --ignore-stage 2 --ignore-stage 4); rc=$?; echo "$out" | tail -3
[ $rc = 0 ] || fail "pass 3: exit $rc"
echo "trials done: $(find "$W/results" -name metrics.json | wc -l | tr -d ' ') (expect 150)"
# the fake agent bills $0.01 an invocation and never fixes anything, so a trial costs $0.03 (first call plus two nudges): 150 trials, about $4.30.
# What is checked is that the ledger equals the sum of the trials' charged_usd, the accounting the real run's cap relies on.
led=$(python3 -I "$AB" spend); sum=$(find "$W/results" -name metrics.json | python3 -I -c "import sys,json; print('\$%.2f' % sum(json.load(open(l.strip()))['charged_usd'] for l in sys.stdin))")
echo "spent: $led; sum of trials' charged_usd: $sum"; [ "$led" = "$sum" ] || fail "ledger $led differs from the trials' charged_usd $sum"
echo "== report"
python3 -I "$AB" report --plan "$P" > "$W/report.txt" 2>&1; sed -n 1,40p "$W/report.txt" | cut -c1-260
grep -q "^Pooled over all classes" "$W/report.txt" || fail "no pooled line in the report"
grep -q "^Target of the final run" "$W/report.txt" || fail "no target verdict in the report"
grep -q '^| held out | ail-plain' "$W/report.txt" || fail "no held-out class row (the stand-in sealed tasks were not read as held out)"
echo "== correctness rule (by interval): synthetic trials, no harness run"
python3 -I - "$HERE" "$AGENTBENCH_HELDOUT" <<'PY' || fail "correctness rule check"
import sys, io, contextlib, importlib.util, os
here = sys.argv[1]; sys.argv = ["agentbench.py"]
sp = importlib.util.spec_from_file_location("ab", os.path.join(here, "agentbench.py")); ab = importlib.util.module_from_spec(sp); sp.loader.exec_module(ab)
def trials(arm, task, n, wrong, cost):
    return [{"task": task, "arm": arm, "trial": str(i), "correct": i >= wrong, "versions_to_green": 1, "charged_usd": cost, "cost_usd": cost} for i in range(n)]
def verdict(a_wrong):
    ms = trials("ail-plain", "ledger", 24, a_wrong, 0.10) + trials("c", "ledger", 24, 0, 0.14)
    buf = io.StringIO()
    with contextlib.redirect_stdout(buf): ab.target_table(ms, {"from scratch": "tied"})
    return buf.getvalue()
one = verdict(1); half = verdict(12)
print([l for l in one.splitlines() if "Target of" in l][0])
assert "**PASSED**" in one, "AIL one correct trial lower must still pass:\n" + one
print([l for l in half.splitlines() if "Target of" in l][0])
assert "**NOT MET**" in half and "correct share not lower" in half, "half the trials correct must fail:\n" + half
print("ok: one trial lower passes, half the trials correct fails")
PY
[ $red = 0 ] && echo "DONE GREEN" || { echo "DONE RED"; exit 1; }
