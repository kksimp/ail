#!/usr/bin/env python3
"""Scripted agent sessions with no model (ADR 0916, amended 2026-11-10).

  replay/run.sh [--only TASK] [--json] [--save FILE] [--compare FILE]

Each tasks/NAME/steps*.txt is one fixed session: the commands an agent would issue for one small task, run for real with today's
`ail`. The script totals tokens WRITTEN (the files and commands the agent sends) and READ (what the tools print back), tokbench
mean (ADR 0817), plus calls (one per write or run). It is the free gate every tooling slice reports against: run it before and
after, `--save` the first table and `--compare` the second. Agent reasoning is not counted. Run it on a designer machine (ADR 0842).

Step lines (blank lines and `#` lines are ignored):
  put FILE FROM        place tasks/NAME/FROM as FILE; the agent did not write it, so it costs nothing (existing code)
  write FILE FROM      place tasks/NAME/FROM as FILE; the agent wrote it, so its tokens count as written
  run COMMAND [<<TAG]  run COMMAND with sh in the scratch directory; a `<<TAG` line starts stdin, which runs to a line `TAG`
Any exit code is tolerated (a red first test is the point), except that the last `run` must exit 0: a session that ends red fails.
`steps.txt` is the default flow; `steps.NAME.txt` is a variant (a flow with a newer tool), reported as `task/NAME`.
"""
import argparse, glob, json, os, re, shutil, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))

def parse(path):
    steps, lines, i = [], open(path).read().split("\n"), 0
    while i < len(lines):
        l = lines[i]; i += 1
        if not l.strip() or l.lstrip().startswith("#"): continue
        kind, _, rest = l.partition(" ")
        if kind in ("put", "write"):
            f, _, src = rest.partition(" "); steps.append({"k": kind, "file": f, "src": src})
        elif kind == "run":
            m = re.match(r"^(.*?)\s*<<(\w+)$", rest); body = ""
            if m:
                rest, tag, buf = m.group(1), m.group(2), []
                while i < len(lines) and lines[i] != tag: buf.append(lines[i]); i += 1
                i += 1; body = "\n".join(buf) + "\n"
            steps.append({"k": "run", "cmd": rest, "stdin": body})
        else: sys.exit(f"{path}: bad step {l!r}")
    return steps

def session(name, d, steps, ail):
    """Run one session; return its snippets ('w' written / 'r' read) and whether it ended green."""
    ws = tempfile.mkdtemp(prefix="replay-"); bindir = os.path.join(ws, ".bin"); os.makedirs(bindir)
    with open(os.path.join(bindir, "ail"), "w") as f: f.write(f"#!/bin/sh\nexec '{ail}' \"$@\"\n")
    os.chmod(os.path.join(bindir, "ail"), 0o755)
    env = dict(os.environ, PATH=bindir + os.pathsep + os.environ["PATH"]); out, rc = [], None
    for s in steps:
        if s["k"] in ("put", "write"):
            shutil.copy(os.path.join(d, s["src"]), os.path.join(ws, s["file"]))
            if s["k"] == "write": out.append(("w", open(os.path.join(ws, s["file"])).read()))
        else:
            p = subprocess.run(["sh", "-c", s["cmd"]], cwd=ws, env=env, input=s["stdin"].encode(), stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=600)
            rc = p.returncode; out.append(("w", "$ " + s["cmd"] + "\n" + s["stdin"])); out.append(("r", p.stdout.decode("utf-8", "replace")))
    shutil.rmtree(ws, ignore_errors=True)
    return out, rc == 0

def tokens(texts):
    """tokbench mean of each text, one tokbench call for the lot."""
    tmp = tempfile.mkdtemp(prefix="replay-tok-"); paths = []
    for i, t in enumerate(texts):
        p = os.path.join(tmp, f"{i}.txt"); open(p, "w").write(t if t.strip() else " "); paths.append(p)
    r = subprocess.run([os.path.join(ROOT, "tools", "tokbench", "run.sh"), "--json"] + paths, capture_output=True, text=True)
    shutil.rmtree(tmp, ignore_errors=True)
    if r.returncode: sys.exit("tokbench failed:\n" + r.stderr)
    return [x["mean"] for x in json.loads(r.stdout)]

def main():
    ap = argparse.ArgumentParser(); ap.add_argument("--only"); ap.add_argument("--json", action="store_true")
    ap.add_argument("--save"); ap.add_argument("--compare"); a = ap.parse_args()
    ail = os.environ.get("AIL") or os.path.join(ROOT, "ail")
    runs = []
    for d in sorted(glob.glob(os.path.join(HERE, "tasks", "*"))):
        task = os.path.basename(d)
        if a.only and a.only != task: continue
        for f in sorted(glob.glob(os.path.join(d, "steps*.txt"))):
            variant = os.path.basename(f)[len("steps"):-len(".txt")].lstrip(".")
            runs.append((task + ("/" + variant if variant else ""), d, parse(f)))
    if not runs: sys.exit("no sessions")
    results, flat = [], []
    for name, d, steps in runs:
        out, green = session(name, d, steps, ail); results.append((name, out, green)); flat += [t for _, t in out]
    card = tokens([open(os.path.join(ROOT, "docs", "language-card.md")).read()])[0]
    tok = iter(tokens(flat)); rows = []
    for name, out, green in results:
        w = r = 0.0
        for kind, _ in out:
            n = next(tok)
            if kind == "w": w += n
            else: r += n
        rows.append({"task": name, "calls": 0, "written": round(w, 1), "read": round(r, 1), "total": round(w + r, 1), "green": green})
    # a call is one write or run step; `put` is existing code, not a call
    for row, (name, d, steps) in zip(rows, runs): row["calls"] = sum(1 for s in steps if s["k"] != "put")
    tot = {"task": "total", "calls": sum(r["calls"] for r in rows), "written": round(sum(r["written"] for r in rows), 1),
           "read": round(sum(r["read"] for r in rows), 1), "total": round(sum(r["total"] for r in rows), 1), "green": all(r["green"] for r in rows)}
    rows.append(tot)
    if a.save: json.dump({"rows": rows, "card": card}, open(a.save, "w"), indent=1)
    base = {r["task"]: r for r in json.load(open(a.compare))["rows"]} if a.compare else {}
    if a.json: print(json.dumps({"rows": rows, "card": card}, indent=1)); return sys.exit(0 if tot["green"] else 1)
    dl = lambda r, k: (f" ({r[k] - base[r['task']][k]:+.1f})" if r["task"] in base else "")
    print("| session | calls | written | read | total | ended green |\n|---|--:|--:|--:|--:|:-:|")
    for r in rows:
        print(f"| {r['task']} | {r['calls']}{dl(r, 'calls')} | {r['written']}{dl(r, 'written')} | {r['read']}{dl(r, 'read')} | {r['total']}{dl(r, 'total')} | {'yes' if r['green'] else 'NO'} |")
    print(f"\nTokens are tokbench means (ADR 0817). Not counted: the language card ({card:.0f} tokens, paid once per session) and agent reasoning.")
    sys.exit(0 if tot["green"] else 1)

if __name__ == "__main__": main()
