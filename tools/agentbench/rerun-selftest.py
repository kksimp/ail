#!/usr/bin/env python3
"""No-model, no-spend, no-AIL check of the rerun's harness parts (plans/rerun.txt): the scoped stage stops of `batch`, `report --reuse-c`
and its refusals, the per-part cut of an unfinished round, and an old-style unscoped plan. Fake trials are written by a stand-in for cmd_run.
usage: python3 tools/agentbench/rerun-selftest.py     prints `rerun selftest OK` or fails with an assertion."""
import argparse, contextlib, io, json, os, random, shutil, sys, tempfile
T = tempfile.mkdtemp(prefix="rerun-selftest-"); H = os.path.join(T, "held"); os.makedirs(H)
with open(H + "/names.txt", "w") as f:
    for i in range(5, 13):
        f.write(f"sealed{i} fake{i}\n"); os.makedirs(f"{H}/fake{i}"); json.dump({"kind": "build", "heldout": True}, open(f"{H}/fake{i}/task.json", "w"))
os.environ["AGENTBENCH_HELDOUT"] = H
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__))); import agentbench as a
OLD = "978793e804b67bc3f2460d2ee6103e78da887a09"; NEW = "f" * 40
a.require_key = lambda: None; a.agent_cli = lambda: "2.1.318 (selftest)"
orig_directives = a.directives
a.directives = lambda n: {k: v for k, v in orig_directives(n).items() if k != "model-sha256"}   # the model pin is the job's check, not this one's

def mk(root, t, arm, tr, tree, vers, usd=0.1):
    d = f"{root}/{t}/{arm}-{tr}"; os.makedirs(d, exist_ok=True)
    json.dump({"task": t, "arm": arm, "trial": str(tr), "tree": tree, "versions_to_green": vers, "correct": True, "cost_usd": usd, "charged_usd": usd, "output": 1,
               "fix_rounds": vers - 1, "debug_rounds": 0, "elapsed_s": 1, "interventions": 0, "docs_bytes": 1000, "turns": 3, "score": {}, "audit": {}}, open(d + "/metrics.json", "w"))

PRACTISED = [("ledger", 24), ("depgraph", 24), ("stock", 24)] + [(x, 10) for x in ("ledger-fix", "stock-locations", "stock-decimal", "stock-bugfix", "stock-refactor", "till-bugfix", "till-rename", "till-returns")]
def old_results(tree=OLD):
    d = os.path.join(T, "old"); shutil.rmtree(d, ignore_errors=True)
    for t, n in PRACTISED:
        for tr in range(1, n + 1): mk(d, t, "c", tr, tree, 1)
    return d

def batch(plan, gap, ignore=(), results=None):
    R = results or os.path.join(T, "res"); a.RESULTS = R; os.makedirs(R, exist_ok=True); rnd = random.Random(1)
    def fake_run(x):
        v = 1 + (1 if x.arm == "ail-plain" and x.task.startswith("fake") and rnd.random() < gap else 0); mk(R, x.task, x.arm, x.trial, NEW, v); return 0
    a.cmd_run = fake_run
    ns = argparse.Namespace(plan=plan, model="m", max_usd=38.0, trial_usd=0.4, docs_policy="read", ignore_stage=list(ignore))
    with contextlib.redirect_stdout(io.StringIO()) as out: rc = a.cmd_batch(ns)
    return rc, out.getvalue(), R, sum(1 for dp, fs in a.live_dirs(R) if "metrics.json" in fs)

def report(R, old, plan="rerun"):
    with contextlib.redirect_stdout(io.StringIO()) as out: a.cmd_report(argparse.Namespace(results=R, plan=plan, reuse_c=old))
    return out.getvalue()

def fresh(): shutil.rmtree(os.path.join(T, "res"), ignore_errors=True)
def refuses(fn, text):
    try: fn()
    except SystemExit as e: assert text in str(e), (text, str(e)); return
    raise AssertionError("expected a refusal: " + text)

# 1. a clean run goes through all 312 rows; the report reuses 152 c rows and prints the held-out line without the validation task
fresh(); rc, out, R, n = batch("rerun", 0.0); assert rc is None and n == 312, (rc, n, out[-300:])
old = old_results(); rep = report(R, old)
assert "Reused c rows: 152 from tree 978793e804b6" in rep and "Held out without the validation task" in rep and "| held out | c | 80 |" in rep and "| change | c | 80 |" in rep and "| from scratch | c | 72 |" in rep, rep[:1500]
# 2. futility after round 5 of the paired part: no spend past it
fresh(); rc, out, R, n = batch("rerun", 0.6); assert rc == 5 and "stage 5 (futility)" in out and n == 80, (rc, n, out[-300:])
# 3. past it, a lost held-out call stops the run before any stage-2 trial
fresh(); rc, out, R, n = batch("rerun", 1.0, ignore=[5]); assert rc == 5 and "stage 10 (lost)" in out and n == 160, (rc, n, out[-300:])
rep = report(R, old); assert "Not run: 11 task(s)" in rep and "| held out | ail-plain | 80 |" in rep
# 4. a half-finished stage 2 drops its open round only, the held-out rounds stay
fresh(); rc, out, R, n = batch("rerun", 0.0); import glob
for dp in glob.glob(R + "/ledger/ail-plain-24"): shutil.rmtree(dp)
rep = report(R, old); assert "Unfinished round: round 24 has missing trials, so rounds 24 and later of 11 task(s) (0 scored" not in rep and "| held out | c | 80 |" in rep and "round 24" in rep, rep[:900]
# 5. refusals: saved c rows of another tree, a missing c row, trials from two trees, no --reuse-c
refuses(lambda: report(R, old_results("0" * 40)), "of another tree")
shutil.rmtree(os.path.join(old, "stock", "c-3")); refuses(lambda: report(R, old), "missing")
old = old_results(); mk(R, "stock", "ail-plain", 3, "e" * 40, 1); refuses(lambda: report(R, old), "more than one tree")
mk(R, "stock", "ail-plain", 3, NEW, 1)
refuses(lambda: report(R, None), "--reuse-c")
# 6. an old-style plan (no scope, every row paired) still stops after its round: stage 2 harness passes, stage 4 futility fires
p = os.path.join(T, "legacy.txt")
open(p, "w").write("@stage 2 harness\n@stage 4 futility 0.1\n" + "".join(f"{t} {arm} {tr}\n" for tr in range(1, 7) for t in ("fake5", "fake6") for arm in ("ail-plain", "c")))
fresh(); rc, out, R, n = batch(p, 0.8); assert rc == 5 and "stage 4 (futility)" in out and n == 16, (rc, n, out[-200:])
print("rerun selftest OK"); shutil.rmtree(T, ignore_errors=True)
