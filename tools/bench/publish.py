#!/usr/bin/env python3
"""Render docs/benchmarks.md from the raw numbers (measurement only, ADR 0070).

    publish.py RACE_SMALL.json RACE_BIG.json TOKENS.json TEMPLATE.md OUT.md ENV_FILE

tools/bench/publish.sh produces the inputs and calls this. The template holds the prose;
the markers {{ENV}}, {{SPEED}}, {{BIG}} and {{TOKENS}} are replaced by generated tables.
"""
import json, os, sys

small, big, tokens, tmpl, out, envf = sys.argv[1:7]
small, big, tokens = (json.load(open(p)) for p in (small, big, tokens))


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

text = open(tmpl).read()
for k, v in (("ENV", open(envf).read().strip()), ("SPEED", speed(small, True)),
             ("BIG", speed(big, False)), ("TOKENS", "\n".join(trows))):
    text = text.replace("{{%s}}" % k, v)
open(out, "w").write(text)
