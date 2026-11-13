#!/usr/bin/env python3
"""checkbench: how good is `ail check` on the mistakes people and agents make? (ADR 1111)

Takes every clean program under corpus/ and examples/ that has an expected output, makes copies with one typical
mistake each (mutate.py), and for each copy runs `ail check --json`, then `ail fix`, checks again and, when clean,
builds and runs it against the expected output. It reports per mistake class:

  cascade   errors reported for the one mistake (target: median 1, at most 2 in 95%)
  on_line   the first error is on the mistaken line, or a note names it (target 95%)
  generic   errors whose help is generic or missing (target under 5% of errors)
  fixable   a machine fix is offered (target 80%)
  fix_ok    of the fixes `ail fix` applied, the share after which the file checks clean and prints the expected
            output (target 100%: a fix that leaves the error or changes the output is a bug)
  silent    the mutant checks clean but prints something else (a class marked silent_ok is listed, not counted)

Usage (from the repository root):
  python3 tools/checkbench/checkbench.py [--fast] [--ail CMD] [--jobs N] [--out DIR] [--json FILE] [--floors FILE]
  --fast        about 300 mutants (the gate suite `checkbench`); the default is the full run
  --floors F    exit 1 when a total falls below its floor in F (`metric value` lines)
  --only M,..   only these mutators;  --files N  at most N programs per mutator (full run default 40)
Deterministic (seeded per program and mutator), no network. Needs python3 3.8+.
"""

import argparse
import hashlib
import json
import os
import random
import re
import shutil
import statistics
import subprocess
import sys
import tempfile
import threading
import time
from concurrent.futures import ThreadPoolExecutor

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import mutate  # noqa: E402

GENERIC = [
    r"docs/language-card\.md lists",
    r"declare it first",
    r"choose a different name(?!,? such as `| `)",
    r"see docs/status\.md",
    r"lists the (?:methods|declarations)",
    r"^check the spelling\.?$",
    r"`ail card` shows each form",
    r"give exactly the arguments the signature lists",
]
GENERIC_RX = re.compile("|".join(GENERIC))
TIMEOUT = 60


def run(cmd, cwd, timeout=TIMEOUT, stdin=subprocess.DEVNULL):
    t = time.time()
    try:
        p = subprocess.run(cmd, cwd=cwd, stdin=stdin, capture_output=True, timeout=timeout)
        return p.returncode, p.stdout.decode("utf-8", "replace"), p.stderr.decode("utf-8", "replace"), time.time() - t
    except subprocess.TimeoutExpired:
        return -999, "", "timeout", time.time() - t


def records(out, err):
    recs = []
    for l in (out + "\n" + err).splitlines():
        l = l.strip()
        if l.startswith("{") and '"severity"' in l:
            try:
                recs.append(json.loads(l))
            except ValueError:
                pass
    return recs


def inputs(root):
    """(path, expected output path) for each program with an expected output."""
    out = []
    waiting = set()
    wf = os.path.join(root, "corpus", "waiting.txt")
    if os.path.exists(wf):
        waiting = {l.split()[0] for l in open(wf) if l.strip()}
    cdir = os.path.join(root, "corpus")
    for n in sorted(os.listdir(cdir)) if os.path.isdir(cdir) else []:
        src, want = os.path.join(cdir, n, n + ".ail"), os.path.join(cdir, n, n + ".out")
        if n not in waiting and os.path.exists(src) and os.path.exists(want):
            out.append((src, want))
    edir = os.path.join(root, "examples")
    for f in sorted(os.listdir(edir)) if os.path.isdir(edir) else []:
        if f.endswith(".ail"):
            src, want = os.path.join(edir, f), os.path.join(edir, f[:-4] + ".out")
            if os.path.exists(want) and "use std.app" not in open(src, encoding="utf-8").read():
                out.append((src, want))
    return out


def seed_of(*parts):
    return int(hashlib.sha256("/".join(parts).encode()).hexdigest()[:12], 16)


class Bench:
    def __init__(self, a):
        self.a = a
        self.root = os.getcwd()
        self.ail = a.ail.split()
        self.tmp = tempfile.mkdtemp(prefix="checkbench-")
        self.oracle = {}
        self.lock = threading.Lock()

    def check(self, path):
        rc, out, err, dt = run(self.ail + ["check", "--json", path], self.root)
        return rc, records(out, err), dt, err

    def build_run(self, path, want_path):
        exe = path[:-4] + ".exe"
        rc, out, err, _ = run(self.ail + ["build", path, "-o", exe], self.root, 300)
        if rc != 0 or not os.path.exists(exe):
            return "nobuild", (err or out)[-600:]
        rc, out, err, _ = run([exe], self.root)
        want = open(want_path, encoding="utf-8").read().replace("\r\n", "\n")
        got = out.replace("\r\n", "\n")
        if rc != 0 or got != want:
            return "differs", f"exit {rc}\n{got[-400:]}"
        return "same", ""

    def siblings(self, src, d):
        """Link the program's sibling directories (examples/configs, data) next to its copy, as it reads them."""
        here = os.path.dirname(src)
        for n in os.listdir(here):
            p = os.path.join(here, n)
            if os.path.isdir(p) and not os.path.exists(os.path.join(d, n)):
                try:
                    os.symlink(os.path.abspath(p), os.path.join(d, n))
                except OSError:
                    pass

    def oracle_ok(self, src, want):
        """Whether the unchanged copy builds and prints the expected output from a temp directory (else a fix
        can't be judged by output, and is counted unverified, not wrong)."""
        with self.lock:
            if src in self.oracle:
                return self.oracle[src]
        d = os.path.join(self.tmp, "oracle", os.path.basename(src)[:-4])
        os.makedirs(d, exist_ok=True)
        p = os.path.join(d, os.path.basename(src))
        shutil.copy(src, p)
        self.siblings(src, d)
        ok = self.build_run(p, want)[0] == "same"
        with self.lock:
            self.oracle[src] = ok
        return ok

    def clean_inputs(self):
        """The programs that check clean as copies (a copy may lose sibling files)."""
        progs = inputs(self.root)

        def one(sw):
            src, want = sw
            d = os.path.join(self.tmp, "base", os.path.basename(src)[:-4])
            os.makedirs(d, exist_ok=True)
            p = os.path.join(d, os.path.basename(src))
            shutil.copy(src, p)
            rc, recs, dt, err = self.check(p)
            errs = [r for r in recs if r.get("severity") == "error"]
            return (src, want, dt) if rc == 0 and not errs else None

        with ThreadPoolExecutor(self.a.jobs) as ex:
            got = list(ex.map(one, progs))
        ok = [g for g in got if g]
        self.check_times = sorted(g[2] for g in ok)
        return [(s, w) for s, w, _ in ok], len(progs)

    def plan(self, progs):
        """One mutant per (program, mutator) where the mutator applies, then a seeded sample."""
        names = list(mutate.MUTATORS)
        if self.a.only:
            names = [n for n in names if n in self.a.only.split(",")]
        per = self.a.files if self.a.files else (3 if self.a.fast else 40)
        jobs = []
        for name in names:
            fn = mutate.MUTATORS[name]
            order = sorted(progs, key=lambda sw: seed_of("order", name, os.path.basename(sw[0])))
            got = 0
            for src, want in order:
                if got >= per:
                    break
                text = open(src, encoding="utf-8").read()
                try:
                    prog = mutate.Prog(src, text)
                    m = fn(prog, random.Random(seed_of(name, os.path.basename(src))))
                except Exception as e:  # a mutator bug is reported, not fatal
                    print(f"mutator {name} failed on {src}: {e}", file=sys.stderr)
                    m = None
                if m and m.text != text:
                    jobs.append((src, want, m))
                    got += 1
        return jobs

    def one(self, job):
        k, (src, want, m) = job
        stem = os.path.basename(src)[:-4]
        d = os.path.join(self.tmp, f"m{k:05d}")
        os.makedirs(d, exist_ok=True)
        p = os.path.join(d, stem + ".ail")
        with open(p, "w", encoding="utf-8") as f:
            f.write(m.text)
        self.siblings(src, d)
        r = {"id": k, "mutator": m.mutator, "group": m.group, "program": os.path.relpath(src, self.root), "what": m.what,
             "lo": m.lo, "hi": m.hi, "silent_ok": m.silent_ok}
        rc, recs, dt, err = self.check(p)
        r["check_ms"] = int(dt * 1000)
        if rc == -999:
            r["outcome"] = "hang"
            return r
        if rc not in (0, 1, 2) and not recs:
            r["outcome"] = "crash"
            r["detail"] = err[-800:]
            return r
        errs = [x for x in recs if x.get("severity") == "error"]
        r["errors"] = len(errs)
        r["rules"] = [x.get("rule") for x in errs]
        if not errs:
            res, detail = self.build_run(p, want)
            r["outcome"] = "benign" if res == "same" else "silent"
            if res != "same" and not self.oracle_ok(src, want):
                r["outcome"] = "unverified"
            r["detail"] = detail
            return r
        first = errs[0]
        lines = [first.get("line", 0)] + [n.get("line", 0) for n in first.get("notes", [])]
        r["on_line"] = any(m.lo <= ln <= m.hi for ln in lines)
        r["first"] = {"rule": first.get("rule"), "line": first.get("line"), "message": first.get("message"),
                      "help": first.get("help")}
        r["generic"] = sum(1 for x in errs if not x.get("help") or GENERIC_RX.search(x.get("help", "")))
        r["fixable"] = any(x.get("fixes") for x in errs)
        r["outcome"] = "reported"
        if r["fixable"]:
            frc, fout, ferr, _ = run(self.ail + ["fix", p], self.root)
            fixed = open(p, encoding="utf-8").read()
            r["fix_changed"] = fixed != m.text
            if r["fix_changed"]:
                rc2, recs2, _, _ = self.check(p)
                left = [x for x in recs2 if x.get("severity") == "error"]
                if left:
                    r["fix"] = "leaves_error"
                    r["detail"] = left[0].get("rule", "") + ": " + left[0].get("message", "")
                else:
                    res, detail = self.build_run(p, want)
                    r["fix"] = {"same": "right", "differs": "changes_output", "nobuild": "does_not_build"}[res]
                    if res == "differs" and not m.restores:
                        r["fix"] = "right"  # fills in what the mistake deleted: clean and builds is right
                    elif res != "same" and not self.oracle_ok(src, want):
                        r["fix"] = "unverified"
                    r["detail"] = detail
                if r["fix"] != "right":
                    with open(p + ".fixed", "w", encoding="utf-8") as f:
                        f.write(fixed)
        return r


def summarize(rs):
    out = {}
    for name in sorted({r["mutator"] for r in rs}):
        out[name] = stats([r for r in rs if r["mutator"] == name])
    out["TOTAL"] = stats(rs)
    return out


def stats(rs):
    rep = [r for r in rs if r["outcome"] == "reported"]
    errs = [r["errors"] for r in rep]
    fixes = [r for r in rep if r.get("fix_changed")]
    nerr = sum(errs)
    s = {
        "mutants": len(rs),
        "reported": len(rep),
        "median_errors": statistics.median(errs) if errs else 0,
        "le2_pct": pct(sum(1 for e in errs if e <= 2), len(errs)),
        "on_line_pct": pct(sum(1 for r in rep if r["on_line"]), len(rep)),
        "generic_pct": pct(sum(r["generic"] for r in rep), nerr),
        "fixable_pct": pct(sum(1 for r in rep if r["fixable"]), len(rep)),
        "fix_ok_pct": pct(sum(1 for r in fixes if r["fix"] == "right"),
                          sum(1 for r in fixes if r["fix"] != "unverified")),
        "fixes_applied": len(fixes),
        "unverified": sum(1 for r in rs if r.get("fix") == "unverified" or r["outcome"] == "unverified"),
        "silent": sum(1 for r in rs if r["outcome"] == "silent" and not r["silent_ok"]),
        "silent_ok": sum(1 for r in rs if r["outcome"] == "silent" and r["silent_ok"]),
        "benign": sum(1 for r in rs if r["outcome"] == "benign"),
        "crash": sum(1 for r in rs if r["outcome"] in ("crash", "hang")),
    }
    return s


def pct(a, b):
    return round(100.0 * a / b, 1) if b else None


COLS = ["mutants", "reported", "median_errors", "le2_pct", "on_line_pct", "generic_pct", "fixable_pct", "fix_ok_pct",
        "fixes_applied", "unverified", "silent", "silent_ok", "benign", "crash"]


def table(summary):
    head = ["mutator"] + COLS
    rows = [[k] + ["-" if v[c] is None else str(v[c]) for c in COLS] for k, v in summary.items()]
    w = [max(len(r[i]) for r in rows + [head]) for i in range(len(head))]
    fmt = lambda r: "  ".join(x.ljust(w[i]) if i == 0 else x.rjust(w[i]) for i, x in enumerate(r))
    return "\n".join([fmt(head)] + [fmt(r) for r in rows])


def badness(r):
    if r["outcome"] in ("crash", "hang"):
        return 1000
    if r["outcome"] == "silent" and not r["silent_ok"]:
        return 500
    if r.get("fix") and r["fix"] not in ("right", "unverified"):
        return 400
    if r["outcome"] != "reported":
        return 0
    return (r["errors"] - 1) * 10 + (0 if r["on_line"] else 30) + r["generic"] * 5 + (0 if r["fixable"] else 3)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--fast", action="store_true")
    # Windows: `ail` is a shell script, which Git's `sh` runs (the gate's own shell)
    ap.add_argument("--ail", default="sh ./ail" if os.name == "nt" else "./ail")
    ap.add_argument("--jobs", type=int, default=4)
    ap.add_argument("--out", default="")
    ap.add_argument("--json", default="")
    ap.add_argument("--floors", default="")
    ap.add_argument("--only", default="")
    ap.add_argument("--files", type=int, default=0)
    ap.add_argument("--keep", action="store_true")
    ap.add_argument("--report", action="store_true", help="one line per mutant that misses a target")
    ap.add_argument("--list", action="store_true", help="print the planned mutants and stop (needs no ail)")
    a = ap.parse_args()
    b = Bench(a)
    t0 = time.time()
    if a.list:
        progs = inputs(b.root)
        for src, _, m in b.plan(progs):
            print(f"{m.mutator:18} {os.path.relpath(src)}:{m.lo}-{m.hi}  {m.what}")
        return 0
    progs, total = b.clean_inputs()
    print(f"checkbench: {len(progs)} of {total} programs check clean", flush=True)
    jobs = b.plan(progs)
    print(f"checkbench: {len(jobs)} mutants over {len({j[2].mutator for j in jobs})} mutators", flush=True)
    with ThreadPoolExecutor(a.jobs) as ex:
        rs = list(ex.map(b.one, enumerate(jobs)))
    summary = summarize(rs)
    if a.report:
        for r in sorted(rs, key=lambda r: (r["mutator"], -badness(r))):
            if badness(r) <= 0:
                continue
            f = r.get("first") or {}
            print(f"MISS {r['mutator']} {r['program']}:{r['lo']}-{r['hi']} [{r['what']}] {r['outcome']}"
                  f" errors={r.get('errors')} on_line={r.get('on_line')} generic={r.get('generic')}"
                  f" fix={r.get('fix', 'offered' if r.get('fixable') else 'none')}"
                  f" | {f.get('rule')}@{f.get('line')}: {(f.get('message') or '')[:140]}"
                  f" | help: {(f.get('help') or '')[:140]} | rules={','.join(r.get('rules') or [])[:120]}"
                  f" | {(r.get('detail') or '')[:200].replace(chr(10), ' / ')}")
    print(table(summary))
    ct = b.check_times
    if ct:
        print(f"check time over the clean programs: median {int(statistics.median(ct) * 1000)} ms, "
              f"max {int(ct[-1] * 1000)} ms")
    print(f"checkbench: {int(time.time() - t0)} s")
    if a.out:
        os.makedirs(a.out, exist_ok=True)
        worst = sorted(rs, key=badness, reverse=True)[:20]
        for r in worst:
            if badness(r) <= 0:
                break
            d = os.path.join(b.tmp, f"m{r['id']:05d}")
            stem = os.path.basename(r["program"])[:-4]
            base = os.path.join(a.out, f"{r['mutator']}-{stem}")
            shutil.copy(os.path.join(d, stem + ".ail"), base + (".fixed.ail" if r.get("fix_changed") else ".ail"))
            with open(base + ".txt", "w", encoding="utf-8") as f:
                f.write(json.dumps(r, indent=1) + "\n")
                rc, out, err, _ = run(b.ail + ["check", os.path.join(d, stem + ".ail")], b.root)
                f.write(err + out)
    if a.json:
        with open(a.json, "w", encoding="utf-8") as f:
            json.dump({"summary": summary, "mutants": rs}, f, indent=1)
    rc = 0
    if a.floors:
        tot = summary["TOTAL"]
        for l in open(a.floors, encoding="utf-8"):
            l = l.split("#")[0].split()
            if len(l) != 2:
                continue
            k, v = l[0], float(l[1])
            got = tot.get(k)
            # a `max_` floor is a ceiling: crash, silent and generic_pct may not rise
            if k.startswith("max_"):
                got = tot.get(k[4:])
                bad = got is None or got > v
            else:
                bad = got is None or got < v
            if bad:
                print(f"RED checkbench {k}: {got} (floor {v})")
                rc = 1
    if not a.keep:
        shutil.rmtree(b.tmp, ignore_errors=True)
    return rc


if __name__ == "__main__":
    sys.exit(main())
