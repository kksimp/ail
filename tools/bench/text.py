#!/usr/bin/env python3
"""Text and allocation micro-benchmarks, AIL against gcc -O2 (measurement only, ADR 0070).

    text.py --out FILE.json [--runs N] [--ail COMMAND]   (default ./ail, ail.cmd on Windows)

Builds each program of tools/perfbench/text/ (NAME.ail and its C twin NAME.c), times the AIL executable (default
threads, then AIL_THREADS=1) and the C executable (`gcc -O2 -ffp-contract=off`), and checks that both print the
same bytes. Sizes are fixed in the sources (each runs about a second); there is no calibration. `readline` and
`fields` read a generated input of five million lines on stdin. The `floatparse` twins print the same sum in
different float text, so their `total` values are compared as numbers. Writes the JSON that publish.py renders.
tools/bench/publish.sh calls this.
"""
import argparse, json, os, shlex, shutil, statistics, subprocess, sys, tempfile, time

NAMES = ["readline", "fields", "splitline", "floatparse", "charat", "charwalk", "gcchurn", "gcints"]
STDIN_LINES = ("readline", "fields")
WHAT = {"readline": "read 5 million lines with io.read_line", "fields": "read 5 million lines, split each on a space, parse the second field",
        "splitline": "split one 5-word line 3 million times", "floatparse": "parse 50 million float strings with to_float",
        "charat": "20 MB string, 25 passes of s.char_at(i)", "charwalk": "20 MB string, 25 passes of for c in s.chars()",
        "gcchurn": "30 million short-lived strings with 500 thousand live ones", "gcints": "20 million short-lived strings beside a 50-million-element list"}
EXE = ".exe" if os.name == "nt" else ""


def run(argv, inp, env=None, timeout=900):
    t = time.perf_counter()
    f = open(inp, "rb") if inp else subprocess.DEVNULL
    try:
        p = subprocess.run(argv, stdin=f, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, env=env, timeout=timeout)
    except subprocess.TimeoutExpired:
        return None, b""
    finally:
        if inp:
            f.close()
    return (time.perf_counter() - t, p.stdout) if p.returncode == 0 else (None, b"")


def measure(argv, inp, runs, env=None):
    ts, out = [], b""
    for _ in range(runs):
        t, out = run(argv, inp, env)
        if t is None:
            return {"err": "fail"}, b""
        ts.append(t)
    return {"min": round(min(ts), 4), "med": round(statistics.median(ts), 4), "max": round(max(ts), 4), "runs": runs}, out


def same(name, a, b):
    if name != "floatparse":
        # The C twins print through Windows' text-mode stdout (CRLF) and AIL prints LF: compare with line ends normalised.
        return a.replace(b"\r\n", b"\n") == b.replace(b"\r\n", b"\n")
    try:
        return float(a.split()[-1]) == float(b.split()[-1])
    except (ValueError, IndexError):
        return False


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True)
    ap.add_argument("--runs", type=int, default=5)
    # shlex.split below eats backslashes, so the path uses `/`. Python cannot start ./ail (a bash script) on Windows (WinError 193): use the ail.cmd wrapper there.
    root = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
    ap.add_argument("--ail", default=os.path.join(root, "ail.cmd").replace("\\", "/") if os.name == "nt" else "./ail")
    a = ap.parse_args()
    ail = shlex.split(a.ail)
    here = os.path.dirname(os.path.abspath(__file__))
    src = os.path.normpath(os.path.join(here, "..", "perfbench", "text"))
    tmp = tempfile.mkdtemp(prefix="ail-text-")
    cc = "gcc" if shutil.which("gcc") else "cc"
    inp = os.path.join(tmp, "lines.txt")
    with open(inp, "w") as f:
        f.write("".join("line %d of the input file with some words\n" % i for i in range(5000000)))
    rows = []
    try:
        for n in NAMES:
            cells = {}
            exe_a, exe_c = os.path.join(tmp, "ail-" + n + EXE), os.path.join(tmp, "c-" + n + EXE)
            p = subprocess.run(ail + ["build", os.path.join(src, n + ".ail"), "-o", exe_a], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            q = subprocess.run([cc, "-O2", "-ffp-contract=off", "-o", exe_c, os.path.join(src, n + ".c")] + ([] if os.name == "nt" else ["-lm"]),
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            i = inp if n in STDIN_LINES else None
            out_a = out_c = b""
            if p.returncode == 0 and os.path.isfile(exe_a):
                cells["native"], out_a = measure([exe_a], i, a.runs)
                e1 = dict(os.environ, AIL_THREADS="1")
                cells["native1t"], _ = measure([exe_a], i, a.runs, e1)
            else:
                cells["native"] = cells["native1t"] = {"err": "build"}
            if q.returncode == 0 and os.path.isfile(exe_c):
                cells["gcc"], out_c = measure([exe_c], i, a.runs)
            else:
                cells["gcc"] = {"err": "build"}
            if "min" in cells["native"] and "min" in cells["gcc"] and not same(n, out_a, out_c):
                cells["native"] = cells["native1t"] = {"err": "DIFF"}
            rows.append({"program": n, "what": WHAT[n], "cells": cells})
            sys.stderr.write("text %s: %s\n" % (n, {k: v.get("med", v.get("err")) for k, v in cells.items()}))
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    json.dump({"rows": rows}, open(a.out, "w"), indent=1)


main()
