#!/usr/bin/env python3
"""Agent end-to-end benchmark harness (ADR 0916): one model, one task, AIL arm vs C arm.

  agentbench.py gen TASK                       # fill cases/{visible,hidden}.jsonl from the task's ref.py
  agentbench.py docs TASK                      # check the two arms' doc bundles are within tolerance
  agentbench.py run TASK ARM TRIAL [--model M] # one trial -> results/TASK/ARM-TRIAL/ (metrics.json, transcript.jsonl)
  agentbench.py score DIR                      # re-score a finished trial's workspace against the hidden cases
  agentbench.py report [RESULTS]               # markdown table over all trials
Run on a Kaleb machine only (ADR 0842): trials call a paid model and build with `ail` / `cc`.
"""
import json, os, shutil, subprocess, sys, tempfile, time, statistics, argparse

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
ARMS = ("ail", "c")
PROMPT = ("You are in a fresh repository. Read SPEC.md and DOCS/, then implement the task in {lang}. "
          "`./build` builds, `./test` runs the visible cases and prints the failures. "
          "Work alone until `./test` passes; hidden cases will also be run. Do not edit build, test or DOCS/.")

def task_dir(t): return os.path.join(HERE, "tasks", t)
def load_task(t):
    with open(os.path.join(task_dir(t), "task.json")) as f: return json.load(f)
def jsonl(path):
    with open(path) as f: return [json.loads(l) for l in f if l.strip()]

# --- cases ---------------------------------------------------------------
def run_case(exe, case, timeout=10):
    t0 = time.time()
    try:
        p = subprocess.run([exe] + case["args"], input=case["stdin"].encode(), capture_output=True, timeout=timeout)
        return {"name": case["name"], "ok": p.stdout.decode("utf-8", "replace") == case["stdout"] and p.returncode == case["code"],
                "code": p.returncode, "stdout": p.stdout.decode("utf-8", "replace"), "err": p.stderr.decode("utf-8", "replace")[-400:], "s": round(time.time() - t0, 3)}
    except subprocess.TimeoutExpired:
        return {"name": case["name"], "ok": False, "code": None, "stdout": "", "err": "timeout", "s": timeout}

def cmd_gen(a):
    d = task_dir(a.task); ref = os.path.join(d, "ref.py")
    vis, hid = [], []
    for c in jsonl(os.path.join(d, "cases", "inputs.jsonl")):
        p = subprocess.run([sys.executable, "-I", ref] + c["args"], input=c["stdin"].encode(), capture_output=True)
        full = {"name": c["name"], "args": c["args"], "stdin": c["stdin"], "stdout": p.stdout.decode(), "code": p.returncode}
        hid.append(full)
        if c.get("visible"): vis.append(full)
    for n, rows in (("visible", vis), ("hidden", hid)):
        with open(os.path.join(d, "cases", n + ".jsonl"), "w") as f:
            f.write("".join(json.dumps(r) + "\n" for r in rows))
    print(f"gen: {len(vis)} visible, {len(hid)} hidden")

# --- doc parity ----------------------------------------------------------
def doc_tokens(paths):
    tb = os.path.join(ROOT, "tools", "tokbench", "run.sh")
    out = subprocess.run([tb, "--json"] + [os.path.join(ROOT, p) for p in paths], capture_output=True, text=True, check=True).stdout
    return sum(r["mean"] for r in json.loads(out))  # mean of the offline tokenizers (ADR 0817)

def cmd_docs(a):
    t = load_task(a.task); n = {arm: doc_tokens(t["docs"][arm]) for arm in ARMS}
    short = (n["ail"] - n["c"]) / n["ail"]  # C may have more docs than AIL, never fewer (ADR 0916)
    print(n, f"C shortfall {short:.1%}"); sys.exit(0 if short <= t["doc_token_tolerance"] else 1)

# --- one trial -----------------------------------------------------------
SHIM_BUILD = "#!/bin/sh\ncd \"$(dirname \"$0\")\"\n{cmd} 2>&1\nrc=$?\necho \"{{\\\"t\\\":$(date +%s),\\\"k\\\":\\\"build\\\",\\\"rc\\\":$rc}}\" >> .bench/runs.jsonl\nexit $rc\n"
SHIM_TEST = "#!/bin/sh\ncd \"$(dirname \"$0\")\"\n./build >/dev/null 2>&1 || {{ echo 'build failed; run ./build'; echo \"{{\\\"t\\\":$(date +%s),\\\"k\\\":\\\"test\\\",\\\"rc\\\":2}}\" >> .bench/runs.jsonl; exit 2; }}\npython3 -I {me} visible .\nrc=$?\necho \"{{\\\"t\\\":$(date +%s),\\\"k\\\":\\\"test\\\",\\\"rc\\\":$rc}}\" >> .bench/runs.jsonl\nexit $rc\n"

def prep(t, task, arm, ws):
    shutil.copytree(os.path.join(task_dir(t), "seed", arm), ws, dirs_exist_ok=True)
    shutil.copy(os.path.join(task_dir(t), "spec.md" if task["kind"] == "build" else "issue.md"), os.path.join(ws, "SPEC.md"))
    os.makedirs(os.path.join(ws, "DOCS"), exist_ok=True); os.makedirs(os.path.join(ws, ".bench"), exist_ok=True)
    for p in task["docs"][arm]: shutil.copy(os.path.join(ROOT, p), os.path.join(ws, "DOCS", os.path.basename(p)))
    shutil.copy(os.path.join(task_dir(t), "cases", "visible.jsonl"), os.path.join(ws, ".bench", "visible.jsonl"))
    for name, body in (("build", SHIM_BUILD.format(cmd=" ".join(task["build"][arm]))), ("test", SHIM_TEST.format(me=os.path.abspath(__file__)))):
        p = os.path.join(ws, name)
        with open(p, "w") as f: f.write(body)
        os.chmod(p, 0o755)
    subprocess.run(["git", "init", "-q"], cwd=ws); subprocess.run(["git", "add", "-A"], cwd=ws)
    subprocess.run(["git", "-c", "user.name=b", "-c", "user.email=b@b", "commit", "-qm", "seed"], cwd=ws)

def agent(ws, prompt, model, task, cont, log):
    cmd = os.environ.get("AGENTBENCH_AGENT", "claude").split() + ["-p", "--model", model, "--output-format", "stream-json", "--verbose",
           "--dangerously-skip-permissions", "--max-turns", str(task["max_turns"])] + (["--continue"] if cont else [])
    with open(log, "a") as f:
        subprocess.run(cmd, input=prompt.encode(), stdout=f, cwd=ws, timeout=task["timeout_s"])

def usage(log):
    u = {"input": 0, "output": 0, "cache_read": 0, "cache_create": 0, "turns": 0, "cost_usd": 0.0}
    for l in open(log):
        try: e = json.loads(l)
        except ValueError: continue
        if e.get("type") == "result":  # one per agent invocation; cumulative for that call
            x = e.get("usage", {}); u["turns"] += e.get("num_turns", 0); u["cost_usd"] += e.get("total_cost_usd", 0.0)
            u["input"] += x.get("input_tokens", 0); u["output"] += x.get("output_tokens", 0)
            u["cache_read"] += x.get("cache_read_input_tokens", 0); u["cache_create"] += x.get("cache_creation_input_tokens", 0)
    return u

def score_ws(t, arm, ws):
    task = load_task(t)
    r = subprocess.run(task["build"][arm], cwd=ws, capture_output=True, text=True)
    if r.returncode: return {"built": False, "passed": 0, "total": len(jsonl(os.path.join(task_dir(t), "cases", "hidden.jsonl"))), "failed": []}
    res = [run_case(os.path.join(ws, "ledger" if t == "ledger" else task.get("exe", t)), c) for c in jsonl(os.path.join(task_dir(t), "cases", "hidden.jsonl"))]
    return {"built": True, "passed": sum(x["ok"] for x in res), "total": len(res), "failed": [x["name"] for x in res if not x["ok"]]}

def cmd_run(a):
    task = load_task(a.task); out = os.path.join(HERE, "results", a.task, f"{a.arm}-{a.trial}"); shutil.rmtree(out, ignore_errors=True); os.makedirs(out)
    ws = os.path.join(out, "ws"); prep(a.task, task, a.arm, ws)
    log = os.path.join(out, "transcript.jsonl"); prompt = PROMPT.format(lang={"ail": "AIL (the `ail` language)", "c": "C"}[a.arm])
    t0 = time.time(); nudges = 0
    agent(ws, prompt, a.model, task, False, log)
    while nudges < task["max_nudges"] and subprocess.run([os.path.join(ws, "test")], capture_output=True).returncode != 0:
        nudges += 1  # a nudge stands in for a human saying "keep going"; counted as an intervention
        agent(ws, "`./test` still fails. Keep going.", a.model, task, True, log)
    elapsed = round(time.time() - t0, 1)
    runs = jsonl(os.path.join(ws, ".bench", "runs.jsonl")) if os.path.exists(os.path.join(ws, ".bench", "runs.jsonl")) else []
    builds = [r for r in runs if r["k"] == "build"]; tests = [r for r in runs if r["k"] == "test"]
    m = {"task": a.task, "arm": a.arm, "trial": a.trial, "model": a.model, "elapsed_s": elapsed, "interventions": nudges,
         "build_failures": sum(r["rc"] != 0 for r in builds), "test_failures": sum(r["rc"] != 0 for r in tests),
         "debug_rounds": sum(r["rc"] != 0 for r in builds + tests), **usage(log), "score": score_ws(a.task, a.arm, ws)}
    m["correct"] = m["score"]["built"] and m["score"]["passed"] == m["score"]["total"]
    with open(os.path.join(out, "metrics.json"), "w") as f: json.dump(m, f, indent=1)
    print(json.dumps({k: m[k] for k in ("arm", "trial", "elapsed_s", "output", "debug_rounds", "interventions", "correct")}))

def cmd_score(a):
    m = json.load(open(os.path.join(a.dir, "metrics.json")))
    m["score"] = score_ws(m["task"], m["arm"], os.path.join(a.dir, "ws")); m["correct"] = m["score"]["built"] and m["score"]["passed"] == m["score"]["total"]
    json.dump(m, open(os.path.join(a.dir, "metrics.json"), "w"), indent=1); print(m["score"])

# --- `visible` is the in-workspace test runner (called by ./test) --------
def cmd_visible(a):
    cases = jsonl(os.path.join(a.dir, ".bench", "visible.jsonl")); bad = 0
    for c in cases:
        r = run_case(os.path.join(a.dir, "ledger"), c)
        if not r["ok"]:
            bad += 1; print(f"FAIL {c['name']}: args={c['args']} want code {c['code']} got {r['code']}\n  want stdout {c['stdout']!r}\n  got  stdout {r['stdout']!r}\n  stderr {r['err']!r}")
    print(f"{len(cases) - bad}/{len(cases)} visible cases pass"); sys.exit(1 if bad else 0)

def cmd_report(a):
    ms = [json.load(open(os.path.join(dp, "metrics.json"))) for dp, _, fs in os.walk(a.results) if "metrics.json" in fs]
    print("| task | arm | trials | correct | median out tokens | median debug rounds | median min | interventions |\n|---|---|--:|--:|--:|--:|--:|--:|")
    for t in sorted({m["task"] for m in ms}):
        for arm in ARMS:
            g = [m for m in ms if m["task"] == t and m["arm"] == arm]
            if g: print(f"| {t} | {arm} | {len(g)} | {sum(m['correct'] for m in g)}/{len(g)} | {statistics.median(m['output'] for m in g):.0f} | "
                        f"{statistics.median(m['debug_rounds'] for m in g):.1f} | {statistics.median(m['elapsed_s'] for m in g)/60:.1f} | {sum(m['interventions'] for m in g)} |")

def main():
    p = argparse.ArgumentParser(); s = p.add_subparsers(dest="c", required=True)
    for n, args in (("gen", ["task"]), ("docs", ["task"]), ("run", ["task", "arm", "trial"]), ("score", ["dir"]), ("visible", ["dir"]), ("report", [])):
        q = s.add_parser(n)
        for x in args: q.add_argument(x)
        if n == "run": q.add_argument("--model", default="claude-sonnet-5-5")
        if n == "report": q.add_argument("results", nargs="?", default=os.path.join(HERE, "results"))
    a = p.parse_args(); globals()["cmd_" + a.c](a)

if __name__ == "__main__": main()
