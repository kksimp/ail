#!/usr/bin/env python3
"""Agent end-to-end benchmark harness (ADR 0916): one model, one task, AIL arm vs C arm.

  agentbench.py gen TASK                       # fill cases/{visible,hidden}.jsonl from the task's ref.py
  agentbench.py docs TASK                      # check the two arms' doc bundles are within tolerance
  agentbench.py run TASK ARM TRIAL --model M   # ARM: ail-plain | ail-tools | c (ail = ail-plain); or AGENTBENCH_MODEL
                                               # one trial -> results/TASK/ARM-TRIAL/ (metrics.json, transcript.jsonl)
  agentbench.py validate TASK                  # solution/ passes every hidden case in all three arms; a fix task's seed/ fails some
  agentbench.py score DIR                      # re-score a finished trial's workspace against the hidden cases
  agentbench.py report [RESULTS]               # markdown table over all trials
Run on a Kaleb machine only (ADR 0842): trials call a paid model and build with `ail` / `cc`.
"""
import json, os, shutil, subprocess, sys, tempfile, time, statistics, argparse

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
ARMS = ("ail-plain", "ail-tools", "c")  # ADR 0916, amended 2026-11-10: the AIL arm is split; "ail" in old results means ail-plain
LANG = {"ail-plain": "ail", "ail-tools": "ail", "c": "c"}  # seeds, build lines and docs are per language
ALIAS = {"ail": "ail-plain"}
def arm_name(a):
    a = ALIAS.get(a, a)
    if a not in ARMS: sys.exit(f"unknown arm {a!r}; one of {', '.join(ARMS)}")
    return a
PROMPT = ("You are in a fresh repository. Read SPEC.md and DOCS/, then implement the task in {lang}. "
          "`./build` builds, `./test` runs the visible cases and prints the failures. "
          "Work alone until `./test` passes; hidden cases will also be run. Do not edit build, test or DOCS/.")

def task_dir(t): return os.path.join(HERE, "tasks", t)
def load_task(t):
    with open(os.path.join(task_dir(t), "task.json")) as f: d = json.load(f)
    d.setdefault("exe", t)
    return d  # "base": a task whose spec, oracle and case inputs this one extends (fix tasks)
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

def case_inputs(t):
    task = load_task(t); own = os.path.join(task_dir(t), "cases", "inputs.jsonl")
    base = case_inputs(task["base"]) if "base" in task else []
    return base + (jsonl(own) if os.path.exists(own) else [])

def cmd_gen(a):
    d = task_dir(a.task); ref = os.path.join(task_dir(load_task(a.task).get("base", a.task)), "ref.py")
    vis, hid = [], []
    for c in case_inputs(a.task):
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
    t = load_task(a.task); n = {l: doc_tokens(t["docs"][l]) for l in ("ail", "c")}
    short = (n["ail"] - n["c"]) / n["ail"]  # C may have more docs than AIL, never fewer (ADR 0916)
    # ail-tools adds the arm's agent files on top of the same card; they are the treatment, so they do not count toward parity (ADR 0916, amended 2026-11-10).
    n["ail-tools agent files"] = doc_tokens(arm_files("ail-tools"))
    print(n, f"C shortfall {short:.1%}"); sys.exit(0 if short <= t["doc_token_tolerance"] else 1)

# --- one trial -----------------------------------------------------------
# Wrapper-logged metrics (ADR 0916, amended 2026-11-10): every ./build, ./test, shimmed `ail` call and MCP tool call appends one line to
# .bench/runs.jsonl ({"t", "k": build|test|tool|mcp, "name", "rc", "out"}) and keeps what it printed in .bench/out/, so
# nothing is self-reported. Nested calls (./test running the build) set BENCH_INNER and are not logged twice.
LOG = "echo \"{{\\\"t\\\":$(date +%s),\\\"k\\\":\\\"{k}\\\",\\\"name\\\":\\\"{name}\\\",\\\"rc\\\":$rc,\\\"out\\\":\\\"out/$n.txt\\\"}}\" >> .bench/runs.jsonl"
HEAD = "#!/bin/sh\ncd \"$(dirname \"$0\")\"\nexport BENCH_INNER=1\nn=$(ls .bench/out | wc -l | tr -d ' ')-{k}\n"
SHIM_BUILD = HEAD + "{cmd} > .bench/out/$n.txt 2>&1\nrc=$?\ncat .bench/out/$n.txt\n" + LOG + "\nexit $rc\n"
SHIM_TEST = (HEAD + "{cmd} > .bench/out/$n.b 2>&1 || {{ echo 'build failed; run ./build' > .bench/out/$n.txt; cat .bench/out/$n.txt; rm .bench/out/$n.b; rc=2; " + LOG
             + "; exit 2; }}\nrm -f .bench/out/$n.b\npython3 -I {me} visible . > .bench/out/$n.txt 2>&1\nrc=$?\ncat .bench/out/$n.txt\n" + LOG + "\nexit $rc\n")
# `ail` on PATH for the agent: logs one `tool` line per call (not `mcp`, which streams) and passes stdin through.
SHIM_AIL = ("#!/bin/sh\nreal='{real}'\nB=\"$(cd \"$(dirname \"$0\")/..\" && pwd)\"\n"
            "if [ -n \"$BENCH_INNER\" ] || [ \"$1\" = mcp ]; then exec \"$real\" \"$@\"; fi\n"
            "n=$(ls \"$B/out\" | wc -l | tr -d ' ')-ail\n"
            "\"$real\" \"$@\" > \"$B/out/$n.txt\" 2>&1\nrc=$?\ncat \"$B/out/$n.txt\"\n"
            "echo \"{{\\\"t\\\":$(date +%s),\\\"k\\\":\\\"tool\\\",\\\"name\\\":\\\"ail $1\\\",\\\"rc\\\":$rc,\\\"out\\\":\\\"out/$n.txt\\\"}}\" >> \"$B/runs.jsonl\"\nexit $rc\n")

def arm_dir(arm): return os.path.join(HERE, "arms", arm)
def arm_files(arm):  # the arm's instruction file, relative to the repository root (what the doc-size check counts)
    p = os.path.join(arm_dir(arm), "AGENTS.md")
    return [os.path.relpath(p, ROOT)] if os.path.exists(p) else []

def real_ail():
    r = os.environ.get("AIL") or shutil.which("ail") or os.path.join(ROOT, "ail")
    if not os.path.exists(r): sys.exit("no `ail`: put it on PATH or set AIL (ADR 0842: run on a designer machine)")
    return os.path.abspath(r)

def write_exec(path, body):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as f: f.write(body)
    os.chmod(path, 0o755)

def prep(t, task, arm, ws):
    lang = LANG[arm]
    shutil.copytree(os.path.join(task_dir(t), "seed", lang), ws, dirs_exist_ok=True)
    shutil.copy(os.path.join(task_dir(t), "spec.md" if task["kind"] == "build" else "issue.md"), os.path.join(ws, "SPEC.md"))
    os.makedirs(os.path.join(ws, "DOCS"), exist_ok=True); os.makedirs(os.path.join(ws, ".bench", "out"), exist_ok=True)
    if task["kind"] == "fix": shutil.copy(os.path.join(task_dir(task["base"]), "spec.md"), os.path.join(ws, "DOCS", "spec.md"))
    with open(os.path.join(ws, ".bench", "exe"), "w") as f: f.write(task["exe"])
    for p in task["docs"][lang]: shutil.copy(os.path.join(ROOT, p), os.path.join(ws, "DOCS", os.path.basename(p)))
    shutil.copy(os.path.join(task_dir(t), "cases", "visible.jsonl"), os.path.join(ws, ".bench", "visible.jsonl"))
    cmd = " ".join(task["build"][lang])
    write_exec(os.path.join(ws, "build"), SHIM_BUILD.format(k="build", name="./build", cmd=cmd))
    write_exec(os.path.join(ws, "test"), SHIM_TEST.format(k="test", name="./test", cmd=cmd, me=os.path.abspath(__file__)))
    if lang == "ail": write_exec(os.path.join(ws, ".bench", "bin", "ail"), SHIM_AIL.format(real=real_ail()))
    if arm == "ail-tools":  # what E1's `ail new` writes, plus the MCP server through the logging proxy; agent files, never the prompt
        d = arm_dir(arm)
        for f in ("AGENTS.md", "CLAUDE.md"): shutil.copy(os.path.join(d, f), os.path.join(ws, f))
        with open(os.path.join(d, ".mcp.json")) as f: mcp = f.read()
        with open(os.path.join(ws, ".mcp.json"), "w") as f: f.write(mcp.replace("{me}", os.path.abspath(__file__)).replace("{ail}", real_ail()).replace("{bench}", os.path.join(os.path.abspath(ws), ".bench")))
    subprocess.run(["git", "init", "-q"], cwd=ws); subprocess.run(["git", "add", "-A"], cwd=ws)
    subprocess.run(["git", "-c", "user.name=b", "-c", "user.email=b@b", "commit", "-qm", "seed"], cwd=ws)

def agent_env(ws):
    e = dict(os.environ); e["PATH"] = os.path.join(os.path.abspath(ws), ".bench", "bin") + os.pathsep + e["PATH"]
    e["BENCH_DIR"] = os.path.join(os.path.abspath(ws), ".bench")
    return e

def agent(ws, prompt, model, task, cont, log):
    cmd = os.environ.get("AGENTBENCH_AGENT", "claude").split() + ["-p", "--model", model, "--output-format", "stream-json", "--verbose",
           "--dangerously-skip-permissions", "--max-turns", str(task["max_turns"])] + (["--continue"] if cont else [])
    with open(log, "a") as f:
        subprocess.run(cmd, input=prompt.encode(), stdout=f, cwd=ws, timeout=task["timeout_s"], env=agent_env(ws))

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

def count_tokens(paths):
    """Tokens (tokbench mean, ADR 0817) of each file, in order: one tokbench call for the lot."""
    if not paths: return []
    tb = os.path.join(ROOT, "tools", "tokbench", "run.sh")
    out = subprocess.run([tb, "--json"] + list(paths), capture_output=True, text=True, check=True).stdout
    return [r["mean"] for r in json.loads(out)]

CHECKS = ("ail check", "ail_check", "./build")  # a first "check" is whichever of these an agent runs first: the build runs the checker

def toolchain_metrics(ws, tokens=True):
    """The wrapper-logged figures of one trial (ADR 0916, amended 2026-11-10). `ws` is the trial workspace."""
    p = os.path.join(ws, ".bench", "runs.jsonl"); runs = jsonl(p) if os.path.exists(p) else []
    calls = {}
    for r in runs:
        n = r["name"] if r["k"] != "mcp" else "mcp:" + r["name"]
        calls[n] = calls.get(n, 0) + 1
    first = next((r for r in runs if r["name"] in CHECKS), None)
    green = next((i + 1 for i, r in enumerate(runs) if r["k"] == "test" and r["rc"] == 0), None)
    outs = [os.path.join(ws, ".bench", r["out"]) for r in runs if r.get("out") and os.path.exists(os.path.join(ws, ".bench", r["out"]))]
    return {"first_check_clean": None if first is None else first["rc"] == 0, "tool_calls": calls, "calls_total": len(runs),
            "toolchain_tokens": round(sum(count_tokens(outs)), 1) if tokens else None, "turns_to_green": green}

def score_ws(t, arm, ws):
    task = load_task(t); lang = LANG[arm]
    cmd = [real_ail() if (lang == "ail" and x == "ail") else x for x in task["build"][lang]]  # honours AIL=, as the shims do
    r = subprocess.run(cmd, cwd=ws, capture_output=True, text=True)
    if r.returncode: return {"built": False, "passed": 0, "total": len(jsonl(os.path.join(task_dir(t), "cases", "hidden.jsonl"))), "failed": []}
    res = [run_case(os.path.join(ws, task["exe"]), c) for c in jsonl(os.path.join(task_dir(t), "cases", "hidden.jsonl"))]
    return {"built": True, "passed": sum(x["ok"] for x in res), "total": len(res), "failed": [x["name"] for x in res if not x["ok"]]}

def cmd_run(a):
    arm = arm_name(a.arm); task = load_task(a.task); out = os.path.join(HERE, "results", a.task, f"{arm}-{a.trial}"); shutil.rmtree(out, ignore_errors=True); os.makedirs(out)
    ws = os.path.join(out, "ws"); prep(a.task, task, arm, ws)
    log = os.path.join(out, "transcript.jsonl"); prompt = PROMPT.format(lang={"ail": "AIL (the `ail` language)", "c": "C"}[LANG[arm]])
    t0 = time.time(); nudges = 0
    agent(ws, prompt, a.model, task, False, log)
    while nudges < task["max_nudges"] and subprocess.run([os.path.join(ws, "test")], capture_output=True, env=agent_env(ws)).returncode != 0:
        nudges += 1  # a nudge stands in for a human saying "keep going"; counted as an intervention
        agent(ws, "`./test` still fails. Keep going.", a.model, task, True, log)
    elapsed = round(time.time() - t0, 1)
    runs = jsonl(os.path.join(ws, ".bench", "runs.jsonl")) if os.path.exists(os.path.join(ws, ".bench", "runs.jsonl")) else []
    builds = [r for r in runs if r["k"] == "build"]; tests = [r for r in runs if r["k"] == "test"]
    m = {"task": a.task, "arm": arm, "trial": a.trial, "model": a.model, "elapsed_s": elapsed, "interventions": nudges,
         "build_failures": sum(r["rc"] != 0 for r in builds), "test_failures": sum(r["rc"] != 0 for r in tests),
         "debug_rounds": sum(r["rc"] != 0 for r in builds + tests), **usage(log), **toolchain_metrics(ws), "score": score_ws(a.task, arm, ws)}
    m["correct"] = m["score"]["built"] and m["score"]["passed"] == m["score"]["total"]
    with open(os.path.join(out, "metrics.json"), "w") as f: json.dump(m, f, indent=1)
    print(json.dumps({k: m[k] for k in ("arm", "trial", "elapsed_s", "output", "debug_rounds", "interventions", "first_check_clean", "turns_to_green", "toolchain_tokens", "correct")}))

# --- the MCP logging proxy (ail-tools arm): .mcp.json starts `agentbench.py mcpproxy`, which runs the real `ail mcp` ---
def cmd_mcpproxy(a):
    import threading
    bench = os.environ["BENCH_DIR"]; lock = threading.Lock(); pending = {}
    child = subprocess.Popen([a.ail, "mcp"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, env=dict(os.environ, BENCH_INNER="1"))
    def note(line, fn):
        try: e = json.loads(line)
        except ValueError: return
        fn(e)
    def up(e):  # agent -> server
        if e.get("method") == "tools/call":
            with lock: pending[e.get("id")] = e.get("params", {}).get("name", "?")
    def down(e):  # server -> agent
        with lock: name = pending.pop(e.get("id"), None)
        if name is None or "id" not in e: return
        text = json.dumps(e.get("result", e.get("error")), ensure_ascii=False)
        content = (e.get("result") or {}).get("content") or []
        text = "".join(c.get("text", "") for c in content) or text
        with lock:
            n = f"{len(os.listdir(os.path.join(bench, 'out')))}-mcp"
            with open(os.path.join(bench, "out", n + ".txt"), "w") as f: f.write(text)
            bad = "error" in e or (e.get("result") or {}).get("isError", False)
            with open(os.path.join(bench, "runs.jsonl"), "a") as f:
                f.write(json.dumps({"t": int(time.time()), "k": "mcp", "name": name, "rc": 1 if bad else 0, "out": f"out/{n}.txt"}) + "\n")
    def pump_up():
        for line in sys.stdin.buffer:
            note(line, up); child.stdin.write(line); child.stdin.flush()
        child.stdin.close()
    th = threading.Thread(target=pump_up, daemon=True); th.start()
    for line in child.stdout:
        note(line, down); sys.stdout.buffer.write(line); sys.stdout.buffer.flush()
    sys.exit(child.wait())

def cmd_score(a):
    m = json.load(open(os.path.join(a.dir, "metrics.json")))
    m["score"] = score_ws(m["task"], arm_name(m["arm"]), os.path.join(a.dir, "ws")); m["correct"] = m["score"]["built"] and m["score"]["passed"] == m["score"]["total"]
    json.dump(m, open(os.path.join(a.dir, "metrics.json"), "w"), indent=1); print(m["score"])

def cmd_validate(a):
    """Every arm's solution/ must pass all hidden cases; a fix task's seed/ must fail some, then be fixed by solution/.
    Then each arm's workspace is built with the solution in place and driven through its shims, so a trial's logging is known to work."""
    task = load_task(a.task); sol_task = task.get("base", a.task); ok = True
    for arm in ARMS:
        lang = LANG[arm]
        for what, src in (("solution", os.path.join(task_dir(sol_task), "solution", lang)), ("seed", os.path.join(task_dir(a.task), "seed", lang))):
            if what == "seed" and task["kind"] != "fix": continue
            with tempfile.TemporaryDirectory() as ws:
                shutil.copytree(src, ws, dirs_exist_ok=True); sc = score_ws(a.task, arm, ws)
            good = sc["built"] and (sc["passed"] == sc["total"]) == (what == "solution")
            ok &= good; print(("ok  " if good else "FAIL"), arm, what, f"{sc['passed']}/{sc['total']}", "fails:", sc["failed"])
        with tempfile.TemporaryDirectory() as tmp:
            ws = os.path.join(tmp, "ws"); prep(a.task, task, arm, ws)
            shutil.copytree(os.path.join(task_dir(sol_task), "solution", lang), ws, dirs_exist_ok=True)
            env = agent_env(ws); rcs = []
            if lang == "ail": rcs.append(subprocess.run([os.path.join(ws, ".bench", "bin", "ail"), "check", "src/main.ail"], cwd=ws, env=env, capture_output=True).returncode)
            rcs += [subprocess.run([os.path.join(ws, x)], cwd=ws, env=env, capture_output=True).returncode for x in ("build", "test")]
            m = toolchain_metrics(ws); names = set(m["tool_calls"])
            want = {"./build", "./test"} | ({"ail check"} if lang == "ail" else set())
            files = {"AGENTS.md", "CLAUDE.md", ".mcp.json"}
            good = (not any(rcs) and want <= names and m["first_check_clean"] is True and m["turns_to_green"] is not None
                    and m["toolchain_tokens"] is not None and all(os.path.exists(os.path.join(ws, f)) == (arm == "ail-tools") for f in files))
            ok &= good; print(("ok  " if good else "FAIL"), arm, "shims", "rc", rcs, "calls", m["tool_calls"], "first_check_clean", m["first_check_clean"],
                              "turns_to_green", m["turns_to_green"], "toolchain_tokens", m["toolchain_tokens"])
    sys.exit(0 if ok else 1)

# --- `visible` is the in-workspace test runner (called by ./test) --------
def cmd_visible(a):
    cases = jsonl(os.path.join(a.dir, ".bench", "visible.jsonl")); bad = 0
    for c in cases:
        r = run_case(os.path.join(a.dir, open(os.path.join(a.dir, ".bench", "exe")).read()), c)
        if not r["ok"]:
            bad += 1; print(f"FAIL {c['name']}: args={c['args']} want code {c['code']} got {r['code']}\n  want stdout {c['stdout']!r}\n  got  stdout {r['stdout']!r}\n  stderr {r['err']!r}")
    print(f"{len(cases) - bad}/{len(cases)} visible cases pass"); sys.exit(1 if bad else 0)

def med(xs):
    xs = [x for x in xs if x is not None]
    return f"{statistics.median(xs):.1f}" if xs else "-"

def cmd_report(a):
    ms = [json.load(open(os.path.join(dp, "metrics.json"))) for dp, _, fs in os.walk(a.results) if "metrics.json" in fs]
    for m in ms: m["arm"] = arm_name(m["arm"])
    print("| task | arm | trials | correct | median out tokens | median debug rounds | median min | interventions | first check clean | median toolchain tokens | median turns to green |\n|---|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|")
    for t in sorted({m["task"] for m in ms}):
        for arm in ARMS:
            g = [m for m in ms if m["task"] == t and m["arm"] == arm]
            if g:
                fc = [m["first_check_clean"] for m in g if m.get("first_check_clean") is not None]
                print(f"| {t} | {arm} | {len(g)} | {sum(m['correct'] for m in g)}/{len(g)} | {statistics.median(m['output'] for m in g):.0f} | "
                      f"{statistics.median(m['debug_rounds'] for m in g):.1f} | {statistics.median(m['elapsed_s'] for m in g)/60:.1f} | {sum(m['interventions'] for m in g)} | "
                      f"{sum(fc)}/{len(fc)} | {med(m.get('toolchain_tokens') for m in g)} | {med(m.get('turns_to_green') for m in g)} |")

def main():
    p = argparse.ArgumentParser(); s = p.add_subparsers(dest="c", required=True)
    for n, args in (("gen", ["task"]), ("docs", ["task"]), ("run", ["task", "arm", "trial"]), ("score", ["dir"]), ("validate", ["task"]), ("visible", ["dir"]), ("report", []), ("mcpproxy", [])):
        q = s.add_parser(n)
        for x in args: q.add_argument(x)
        if n == "run": q.add_argument("--model", default=os.environ.get("AGENTBENCH_MODEL"), required="AGENTBENCH_MODEL" not in os.environ)
        if n == "mcpproxy": q.add_argument("--ail", default=os.environ.get("AIL", "ail"))
        if n == "report": q.add_argument("results", nargs="?", default=os.path.join(HERE, "results"))
    a = p.parse_args(); globals()["cmd_" + a.c](a)

if __name__ == "__main__": main()
