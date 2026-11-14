#!/usr/bin/env python3
"""Render docs/benchmarks.md from the raw numbers (measurement only, ADR 0070).

    publish.py [--latency LAT.json]... RACE_SMALL.json RACE_BIG.json TOKENS.json TEMPLATE.md OUT.md ENV_FILE
    publish.py --skeleton TEMPLATE.md OUT.md

tools/bench/publish.sh produces the inputs and calls this. The template holds the prose;
the markers {{ENV}}, {{SPEED}}, {{BIG}}, {{TOKENS}}, {{LATENCY}} and {{SUMMARY}} are replaced by
generated text. Every sentence in {{SUMMARY}} is computed from the numbers, so it cannot disagree
with the tables. --skeleton writes the page with every number replaced by PENDING (no inputs),
for the page that ships before the release run.
"""
import json, math, os, sys

PENDING = "PENDING"
args = sys.argv[1:]
lat_files = []
while args and args[0] in ("--latency", "--skeleton"):
    if args[0] == "--latency":
        lat_files.append(args[1]); args = args[2:]
    else:
        break
skeleton = bool(args) and args[0] == "--skeleton"
if skeleton:
    tmpl, out = args[1:3]
else:
    small, big, tokens, tmpl, out, envf = args[:6]
    small, big, tokens = (json.load(open(p)) for p in (small, big, tokens))
    lats = [json.load(open(p)) for p in lat_files]


def cell(c):
    return "%.3f" % c["min"] if c and "min" in c else (c or {}).get("err", "-")


def ratio(a, b):
    if a and b and "min" in a and "min" in b and b["min"] > 0:
        return "%.2f" % (a["min"] / b["min"])
    return "-"


def speed(race, py):
    head = ["program", "size", "AIL", "gcc -O2", "clang -O2"] + (["Python"] if py else []) + ["AIL / gcc"] + (["AIL / Python"] if py else [])
    rows = ["| " + " | ".join(head) + " |", "|---|" + "---:|" * (len(head) - 1)]
    for r in race["rows"]:
        c = r["cells"]
        line = [r["program"], r.get("size") or "-", cell(c.get("native")), cell(c.get("gcc")), cell(c.get("clang"))]
        if py:
            line.append(cell(c.get("python")))
        line.append(ratio(c.get("native"), c.get("gcc")))
        if py:
            line.append(ratio(c.get("native"), c.get("python")))
        rows.append("| " + " | ".join(line) + " |")
    return "\n".join(rows)


def tok(name):
    return next((r["mean"] for r in tokens if os.path.basename(r["file"]) == name), None)



SMALL_NAMES = "collatz nbody spectralnorm mandelbrot fannkuch binarytrees fasta wordfreq matmul sieve".split()
BIG_NAMES = "lzr vm raytrace matmul1400".split()


def num(c):
    return c["min"] if c and "min" in c else None


def geomean(xs):
    return math.exp(sum(math.log(x) for x in xs) / len(xs)) if xs else None


def pairs(race):
    """(program, AIL/gcc) for every row where both columns have a time."""
    got = []
    for r in race["rows"]:
        a, g = num(r["cells"].get("native")), num(r["cells"].get("gcc"))
        if a and g:
            got.append((r["program"], a / g))
    return got


def token_table():
    trows = ["| program | AIL | C | Python | AIL / Python |", "|---|---:|---:|---:|---:|"]
    tot = [0, 0, 0]
    for r in small["rows"]:
        n = r["program"]
        a, c, p = tok(n + ".ail"), tok(n + ".c"), tok(n + ".py")
        if None in (a, c, p):
            continue
        tot = [tot[0] + a, tot[1] + c, tot[2] + p]
        trows.append("| %s | %.0f | %.0f | %.0f | %.2f |" % (n, a, c, p, a / p))
    if tot[2]:
        trows.append("| **total** | %.0f | %.0f | %.0f | %.2f |" % (tot[0], tot[1], tot[2], tot[0] / tot[2]))
    return "\n".join(trows), tot


def ms(c):
    return "%.0f" % c["min"] if c and c.get("ok") else "-"


LAT_ORDER = ["check", "context", "test", "build cold", "build cached", "run cached"]


def latency_table(lats):
    """Edit loop: one row per command and program, one column per host (best of the runs, ms)."""
    if not lats:
        return "No latency run was included."
    names = [n for n in ("hello", "json", "ailtest") if n in lats[0]["programs"]]
    head = ["command", "program (lines)"] + [l["host"] for l in lats]
    rows = ["| " + " | ".join(head) + " |", "|---|---|" + "---:|" * len(lats)]
    for cmd in LAT_ORDER:
        for n in names:
            cells = [next((r for r in l["rows"] if r["cmd"] == cmd and r["program"] == n), None) for l in lats]
            if not any(cells):
                continue
            rows.append("| `ail %s` | %s (%d) | %s |" % (cmd.split()[0] + (" --cold" if cmd == "build cold" else ""),
                        n, lats[0]["programs"][n], " | ".join(ms(c) for c in cells)))
    for n in names:
        first = [l["mcp"].get(n) for l in lats]
        rep = first
        rows.append("| `ail mcp` `ail_check`, first call | %s (%d) | %s |" % (n, lats[0]["programs"][n],
                    " | ".join("%.0f" % m["first"] if m and m.get("ok") else "-" for m in first)))
        rows.append("| `ail mcp` `ail_check`, repeat | %s (%d) | %s |" % (n, lats[0]["programs"][n],
                    " | ".join("%.0f" % m["repeat"] if m and m.get("ok") and m.get("repeat") is not None else "-" for m in rep)))
    if any("fresh_clone" in l for l in lats):
        rows.append("| first `./ail run examples/hello.ail` in a fresh clone | hello (2) | %s |" % " | ".join(
            ("%.1f s" % (l["fresh_clone"]["ms"] / 1000.0) if l["fresh_clone"].get("ok") else "-") if "fresh_clone" in l else "n/a" for l in lats))
    return "\n".join(rows)


def skeleton_tables():
    sp = ["| program | size | AIL | gcc -O2 | clang -O2 | Python | AIL / gcc | AIL / Python |", "|---|---|---:|---:|---:|---:|---:|---:|"]
    sp += ["| %s | %s | %s |" % (n, PENDING, " | ".join([PENDING] * 6)) for n in SMALL_NAMES]
    bg = ["| program | size | AIL | gcc -O2 | clang -O2 | AIL / gcc |", "|---|---|---:|---:|---:|---:|"]
    bg += ["| %s | %s | %s |" % (n, PENDING, " | ".join([PENDING] * 4)) for n in BIG_NAMES]
    tk = ["| program | AIL | C | Python | AIL / Python |", "|---|---:|---:|---:|---:|"]
    tk += ["| %s | %s |" % (n, " | ".join([PENDING] * 4)) for n in SMALL_NAMES]
    lat = ["| command | program (lines) | host |", "|---|---|---:|"]
    for cmd in LAT_ORDER:
        for n in ("hello", "json", "ailtest"):
            if cmd == "run cached" and n == "ailtest":
                continue
            lat.append("| `ail %s` | %s | %s |" % (cmd, n, PENDING))
    return "\n".join(sp), "\n".join(bg), "\n".join(tk), "\n".join(lat)


def summary(sm, bg, tot, lats):
    out = []
    sp, bp = pairs(sm), pairs(bg)
    if sp:
        faster = [p for p, r in sp if r < 1]
        slower = [p for p, r in sp if r >= 1]
        out.append("- **Small programs.** AIL is faster than `gcc -O2` on %d of %d (%s) and slower on %d (%s); the geometric mean of AIL / gcc is %.2f."
                   % (len(faster), len(sp), ", ".join(faster) or "none", len(slower), ", ".join(slower) or "none", geomean([r for _, r in sp])))
    if bp:
        out.append("- **Larger programs.** AIL / gcc is " + ", ".join("%.2f on %s" % (r, p) for p, r in bp)
                   + "; the geometric mean is %.2f. Above 1 means AIL is slower." % geomean([r for _, r in bp]))
    if tot[2]:
        out.append("- **Tokens.** Across the ten programs AIL source is %.0f%% of the Python source and %.0f%% of the C source by mean token count (%.0f, %.0f, %.0f)."
                   % (100.0 * tot[0] / tot[2], 100.0 * tot[0] / tot[1], tot[0], tot[1], tot[2]))
    for l in lats:
        d = {(r["cmd"], r["program"]): r for r in l["rows"]}
        c = d.get(("check", "hello"))
        m = l["mcp"].get("hello", {})
        t = d.get(("check", "ailtest"))
        if c and c.get("ok"):
            s = "- **Edit loop on %s.** `ail check` of hello takes %.0f ms" % (l["host"], c["min"])
            if t and t.get("ok"):
                s += " and of the %d-line `ailtest` %.0f ms" % (l["programs"]["ailtest"], t["min"])
            if m.get("ok") and m.get("repeat") is not None:
                s += "; a repeated `ail_check` through a warm `ail mcp` takes %.0f ms" % m["repeat"]
            out.append(s + ".")
    return "\n".join(out) or "No summary: the run produced no comparable rows."


text = open(tmpl).read()
if skeleton:
    sp, bg, tk, lat = skeleton_tables()
    repl = (("ENV", "- date: PENDING\n- AIL commit: PENDING\n- host: PENDING\n- gcc, clang, python: PENDING\n- runs per cell: PENDING"),
            ("SPEED", sp), ("BIG", bg), ("TOKENS", tk), ("LATENCY", lat),
            ("SUMMARY", "PENDING: the release run fills this in from the measured numbers. This page has not been measured yet."))
else:
    tt, tot = token_table()
    repl = (("ENV", open(envf).read().strip()), ("SPEED", speed(small, True)), ("BIG", speed(big, False)),
            ("TOKENS", tt), ("LATENCY", latency_table(lats)), ("SUMMARY", summary(small, big, tot, lats)))
for k, v in repl:
    text = text.replace("{{%s}}" % k, v)
open(out, "w").write(text)
