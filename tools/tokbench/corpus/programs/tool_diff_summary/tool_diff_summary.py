import json
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


for name in ["scale.ail", "report.ail"]:
    r = ail("diff", "--json", str(DATA / "v1" / name), str(DATA / "v2" / name))
    for line in lines_of(r.stdout):
        d = parse(line)
        if d is not None and d.get("summary") is True:
            added = d.get("added", 0)
            removed = d.get("removed", 0)
            changed = d.get("changed", 0)
            tests = d.get("tests_changed", 0)
            print(f"{name}: added {added}, removed {removed}, changed {changed}, tests changed {tests}")
