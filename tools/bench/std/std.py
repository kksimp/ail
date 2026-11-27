#!/usr/bin/env python3
"""Std-module benchmark: AIL's standard library against the usual libraries of C, Rust, Go and Python.

Measurement only (ADR 0070). Reuses the drag-race harness (tools/perfbench/race.py) for timing and
builds. Each program in tools/bench/std/programs/ exists once per language where that language has the
library in its standard distribution (Python and Go stdlib, zlib for C, Rust std for sort/maps), and
every twin prints the same bytes: a column whose output differs from the first reference is DIFF and
left out of the ratios. A cell with no twin reads "no twin" (Rust has no JSON, hash, regex, ... in std).

  python3 tools/bench/std/std.py                    # all programs, 5 runs
  python3 tools/bench/std/std.py --runs 3 json_parse sort_ints
  python3 tools/bench/std/std.py --scale 0.1        # smaller sizes, for a smoke run
  python3 tools/bench/std/std.py --md OUT.md --json OUT.json

Columns: ail (default threads), ail1t (AIL_THREADS=1), c (gcc -O2, extra flags in NAME.cflags), rustc
(-C opt-level=3), go (go build), python (python3 -I). Ratios are ail1t / X on the best run, below 1 means
AIL is faster. Run it on a quiet machine (load below 3).
"""
import argparse, importlib.util, json, os, platform, shutil, subprocess, sys, tempfile, time

HERE = os.path.dirname(os.path.abspath(__file__))
PROGS = os.path.join(HERE, "programs")
spec = importlib.util.spec_from_file_location("race", os.path.join(HERE, "..", "..", "perfbench", "race.py"))
race = importlib.util.module_from_spec(spec)
spec.loader.exec_module(race)


def build_c(src, tmp):
    flags = []
    f = src[:-2] + ".cflags"
    if os.path.isfile(f):
        flags = open(f).read().split()
    out = os.path.join(tmp, "c-" + os.path.basename(src)[:-2])
    cc = shutil.which("gcc") or shutil.which("cc")
    p = subprocess.run([cc, "-O2", "-ffp-contract=off", "-o", out, src] + flags,
                       stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    return out if p.returncode == 0 else None


def sized(name, scale):
    a = race.size_args(os.path.join(PROGS, name + ".ail"))
    if a and a[0].isdigit():
        return [str(max(1, int(int(a[0]) * scale)))]
    return a


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("names", nargs="*")
    ap.add_argument("--runs", type=int, default=5)
    ap.add_argument("--scale", type=float, default=1.0, help="multiply every size in NAME.args")
    ap.add_argument("--timeout", type=int, default=300)
    ap.add_argument("--md")
    ap.add_argument("--json")
    a = ap.parse_args()
    names = a.names or sorted(f[:-4] for f in os.listdir(PROGS) if f.endswith(".ail"))
    tmp = tempfile.mkdtemp(prefix="stdbench-")
    cols = ["ail", "ail1t", "c", "rustc", "go", "python"]
    rows = []
    try:
        for name in names:
            ail = os.path.join(PROGS, name + ".ail")
            extra = sized(name, a.scale)
            res = {}
            exe = race.build_native(ail, tmp, os.environ.get("AIL_BUILD_ARGS", ""))
            if exe:
                res["ail"] = race.measure([exe] + extra, a.runs, a.timeout)
                res["ail1t"] = race.measure([exe] + extra, a.runs, a.timeout, {"AIL_THREADS": "1"})
            else:
                res["ail"] = res["ail1t"] = {"err": "build"}
            for col, ext, build in (("c", ".c", lambda s: build_c(s, tmp)),
                                    ("rustc", ".rs", lambda s: race.build_rust(s, tmp)),
                                    ("go", ".go", lambda s: race.build_go(s, tmp))):
                src = os.path.join(PROGS, name + ext)
                if not os.path.isfile(src) or (col == "rustc" and not shutil.which("rustc")) \
                        or (col == "go" and not shutil.which("go")):
                    res[col] = {"err": "no twin"}
                    continue
                e = build(src)
                res[col] = race.measure([e] + extra, a.runs, a.timeout) if e else {"err": "build"}
            src = os.path.join(PROGS, name + ".py")
            res["python"] = race.measure([sys.executable, "-I", src] + extra, a.runs, a.timeout) \
                if os.path.isfile(src) else {"err": "no twin"}
            # The reference is the first column that ran, preferring the mature twins (Python, Go, C).
            ref = next((res[c]["out"] for c in ("python", "go", "c", "rustc", "ail") if "out" in res.get(c, {})), None)
            for c, r in res.items():
                if "out" in r and ref is not None and r["out"].replace(b"\r\n", b"\n") != ref.replace(b"\r\n", b"\n"):
                    res[c] = {"err": "DIFF"}
            rows.append((name, " ".join(extra), res))
            sys.stderr.write("  %s done\n" % name)
    finally:
        shutil.rmtree(tmp, ignore_errors=True)

    head = ["program", "size"] + ["%s med" % c for c in cols] + ["ail1t/%s" % c for c in cols[2:]]
    table = [head]
    for name, size, res in rows:
        line = [name, size]
        for c in cols:
            r = res[c]
            line.append(race.fmt(r["med"]) if "med" in r else r.get("err", "-"))
        for c in cols[2:]:
            line.append(race.ratio(res["ail1t"].get("min"), res[c].get("min")))
        table.append(line)
    widths = [max(len(r[i]) for r in table) for i in range(len(head))]
    for r in table:
        print("  ".join(v.ljust(widths[0]) if i == 0 else v.rjust(widths[i]) for i, v in enumerate(r)))

    if a.json:
        with open(a.json, "w") as f:
            json.dump({"runs": a.runs, "scale": a.scale, "columns": cols, "rows": [
                {"program": n, "size": s, "cells": {c: {k: v for k, v in r.items() if k != "out"} for c, r in res.items()}}
                for n, s, res in rows]}, f, indent=1)
    if a.md:
        ail_rev = subprocess.run(["git", "-C", race.ROOT, "rev-parse", "--short", "HEAD"],
                                 stdout=subprocess.PIPE, text=True).stdout.strip()
        env = [("date", time.strftime("%Y-%m-%d", time.gmtime())), ("AIL commit", ail_rev),
               ("os", "%s %s" % (platform.system(), platform.machine())),
               ("cpu", "%s, %d logical cores" % (race.cpu_name(), os.cpu_count() or 0)),
               ("load at start", " ".join("%.2f" % x for x in os.getloadavg())),
               ("runs per cell", "%d (median shown; ratios use the best run)" % a.runs),
               ("gcc", race.tool_version(["gcc", "--version"]) or "-"),
               ("rustc", race.tool_version(["rustc", "--version"]) or "-"),
               ("go", race.tool_version(["go", "version"]) or "-"),
               ("python", platform.python_version())]
        lines = ["# Std module benchmarks", "",
                 "Generated by `tools/bench/std/std.py`. Seconds of wall time for the whole process.",
                 "`ail1t/X` is the single-threaded AIL run over X's best run: below 1 means AIL is faster.", "",
                 "## Environment", ""] + ["- %s: %s" % kv for kv in env]
        lines += ["", "## Results", "", "| " + " | ".join(head) + " |",
                  "|" + "|".join(["---", "---:"] + ["---:"] * (len(head) - 2)) + "|"]
        lines += ["| " + " | ".join(r) + " |" for r in table[1:]]
        lines += [""]
        open(a.md, "w").write("\n".join(lines))


if __name__ == "__main__":
    main()
