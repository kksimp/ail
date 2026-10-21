import json
import os
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
DATA = ROOT / "tools/tokbench/corpus/agent-r5/data"
AIL = str(ROOT / "ail")


def lines_of(text):
    """Split on newlines only, as AIL's `lines()` does (str.splitlines also splits on form feeds)."""
    parts = text.split("\n")
    if parts and parts[-1] == "":
        parts.pop()
    return parts


def parse(line):
    try:
        return json.loads(line)
    except ValueError:
        return None


def ail(*args, stdin=None):
    return subprocess.run([AIL, *args], capture_output=True, text=True, input=stdin)


counts = {}
for name in sorted(os.listdir(DATA)):
    if not name.endswith(".ail"):
        continue
    r = ail("check", "--json", str(DATA / name))
    for line in lines_of(r.stderr):
        d = parse(line)
        if d is None:
            continue
        rule = d.get("rule")
        if not isinstance(rule, str):
            continue
        counts[rule] = counts.get(rule, 0) + 1
for rule in sorted(counts, key=lambda m: -counts[m]):
    print(f"{counts[rule]} {rule}")
print(f"{len(counts)} distinct rules")
