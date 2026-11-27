#!/usr/bin/env python3
"""Outside-agent kit (ADR 0916 harness, no paid API): prepares agentbench workspaces for an agent that runs OUTSIDE the
harness (a chat assistant with a shell on the designer's Mac, a CLI, an IDE agent), then scores what it left behind.

  outside.py make KIT [--tasks A,B] [--arms ail-plain,c] [--trial N]   # KIT/work/TASK-ARM/ (agent sees only this), KIT/prompts/, KIT/score/
  outside.py score KIT [--json]                                        # hidden cases + the wrapper-logged tries, one table, KIT/results.json

The workspace is the same one agentbench gives a trial: SPEC.md, DOCS/, src/ seed, ./build, ./test (visible cases), and the
shims that log every build, test and `ail`/`cc` call into .bench/runs.jsonl. The hidden cases are NOT in the workspace: they are
copied to KIT/score/, which the agent must not be able to read. Keep KIT outside the repository, and start the agent in one
work/TASK-ARM folder only. Run on a designer machine (ADR 0842)."""
import argparse, json, os, shutil, sys, time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "agentbench"))
import agentbench as ab  # noqa: E402

BUILD = ["ledger", "depgraph"]
MAINTAIN = ["stock-bugfix", "stock-locations", "stock-refactor", "till-bugfix", "till-returns", "till-rename"]
DEFAULT_TASKS = BUILD + MAINTAIN
DEFAULT_ARMS = ["ail-plain", "c"]
LANGNAME = {"ail": "AIL (the `ail` language; its compiler is the `ail` command already on PATH in this folder)", "c": "C"}

PROMPT = """You are in a fresh repository (this folder). Read SPEC.md, then read every file in DOCS/ in full before you write any code:
a file longer than 24 KB is split into numbered parts, and every part has to be read.
Then do the task in {lang}. `./build` builds, `./test` runs the visible cases and prints the failures.
Work alone until `./test` passes; hidden cases will also be run afterwards. Do not edit `build`, `test`, `.bench/` or `DOCS/`.
Run `export PATH="$PWD/.bench/bin:$PATH"` first: it puts `ail`, `cc` and `sed` on PATH through wrappers that log your build and test calls (that is how tries are counted). Use only this folder and those tools; do not search the machine for other copies of the task or of the compiler's source.
Do not ask me questions and do not wait for me: keep going until `./test` passes or you are truly stuck. When you stop, say in
one line whether `./test` passes.
"""

def cmd_make(a):
    kit = os.path.abspath(a.kit)
    if kit.startswith(ab.ROOT + os.sep) or kit == ab.ROOT: sys.exit("KIT must be outside the repository (the hidden cases are in tools/agentbench/tasks)")
    if os.path.exists(os.path.join(kit, "work")): sys.exit(f"{kit}/work exists; pick a new KIT or remove it")
    tasks = a.tasks.split(",") if a.tasks else DEFAULT_TASKS
    arms = [ab.arm_name(x) for x in a.arms.split(",")]
    ab.real_ail()  # fails early when there is no ail
    index = []
    for t in tasks:
        task = ab.load_task(t)
        sc = os.path.join(kit, "score", t); os.makedirs(sc, exist_ok=True)
        shutil.copy(os.path.join(ab.task_dir(t), "task.json"), os.path.join(sc, "task.json"))
        shutil.copy(os.path.join(ab.task_dir(t), "cases", "hidden.jsonl"), os.path.join(sc, "hidden.jsonl"))
        for arm in arms:
            name = f"{t}-{arm}-{a.trial}"
            ws = os.path.join(kit, "work", name); ab.prep(t, task, arm, ws)
            for x in ("build", "test"):  # outside the harness nothing puts the logging shims on PATH, so the scripts do it themselves
                fp = os.path.join(ws, x); body = open(fp).read().replace('cd "$(dirname "$0")"\n', 'cd "$(dirname "$0")"\nexport PATH="$PWD/.bench/bin:$PATH"\n', 1)
                open(fp, "w").write(body)
            prompt = PROMPT.format(lang=LANGNAME[ab.LANG[arm]])
            os.makedirs(os.path.join(kit, "prompts"), exist_ok=True)
            with open(os.path.join(kit, "prompts", name + ".txt"), "w") as f: f.write(prompt)
            index.append({"name": name, "task": t, "arm": arm, "kind": task["kind"], "trial": a.trial})
    with open(os.path.join(kit, "index.json"), "w") as f: json.dump(index, f, indent=1)
    with open(os.path.join(kit, "README.txt"), "w") as f: f.write(open(os.path.join(HERE, "README.md")).read())
    print(f"{len(index)} workspaces in {kit}/work; prompts in {kit}/prompts. Next: tools/outside-test/README.md, 'Running it'.")

def score_one(kit, e):
    t, arm = e["task"], e["arm"]; ws = os.path.join(kit, "work", e["name"])
    os.environ["AGENTBENCH_HELDOUT"] = os.path.join(kit, "score")  # the copy made by `make`; the repo's own cases are identical
    sc = ab.score_ws(t, arm, ws); m = ab.toolchain_metrics(ws, tokens=False)
    runs = ab.jsonl(os.path.join(ws, ".bench", "runs.jsonl")) if os.path.exists(os.path.join(ws, ".bench", "runs.jsonl")) else []
    ts = [r["t"] for r in runs]
    return {**e, "built": sc["built"], "passed": sc["passed"], "total": sc["total"], "structure": sc.get("structure"), "correct": ab.is_correct(sc),
            "failed_cases": sc["failed"][:5], "touched": bool(runs), "versions": m["versions"], "versions_to_green": m["versions_to_green"],
            "first_check_clean": m["first_check_clean"], "fix_rounds": m["fix_rounds"], "build_rewrites": m["build_rewrites"],
            "calls": m["calls_total"], "minutes": round((max(ts) - min(ts)) / 60, 1) if ts else None}

def cmd_score(a):
    kit = os.path.abspath(a.kit); index = json.load(open(os.path.join(kit, "index.json")))
    rows = [score_one(kit, e) for e in index]
    with open(os.path.join(kit, "results.json"), "w") as f: json.dump(rows, f, indent=1)
    if a.json: print(json.dumps(rows, indent=1)); return 0
    print(f"{'workspace':34} {'kind':8} {'hidden':>8} {'ok':>3} {'vers':>4} {'to-green':>8} {'1st-clean':>9} {'min':>5}")
    for r in rows:
        print(f"{r['name']:34} {r['kind']:8} {str(r['passed'])+'/'+str(r['total']):>8} {'yes' if r['correct'] else ('-' if not r['touched'] else 'NO'):>3} "
              f"{r['versions']:>4} {str(r['versions_to_green']):>8} {str(r['first_check_clean']):>9} {str(r['minutes']):>5}")
    print()
    for arm in sorted({r["arm"] for r in rows}):
        rs = [r for r in rows if r["arm"] == arm and r["touched"]]
        if not rs: continue
        g = [r["versions_to_green"] for r in rs if r["versions_to_green"]]
        print(f"{arm:10} correct {sum(r['correct'] for r in rs)}/{len(rs)}  first ./test-green version {sum(1 for x in g if x == 1)}/{len(rs)}  "
              f"mean versions to green {sum(g)/len(g):.2f} (over {len(g)} that reached green)" if g else f"{arm:10} correct {sum(r['correct'] for r in rs)}/{len(rs)}  none reached green")
    print("\nversions = distinct source states the agent built or tested (the seed excluded); fewer is better. Same tasks, same prompt, both arms.")
    return 0

def main():
    p = argparse.ArgumentParser(); s = p.add_subparsers(dest="cmd", required=True)
    m = s.add_parser("make"); m.add_argument("kit"); m.add_argument("--tasks"); m.add_argument("--arms", default=",".join(DEFAULT_ARMS)); m.add_argument("--trial", type=int, default=1)
    c = s.add_parser("score"); c.add_argument("kit"); c.add_argument("--json", action="store_true")
    a = p.parse_args(); sys.exit(cmd_make(a) if a.cmd == "make" else cmd_score(a))

if __name__ == "__main__": main()
