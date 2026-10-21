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


total_errors = 0
total_warnings = 0
for name in sorted(os.listdir(DATA)):
    if not name.endswith(".ail"):
        continue
    r = ail("check", "--json", str(DATA / name))
    errors = 0
    warnings = 0
    for line in lines_of(r.stderr):
        d = parse(line)
        if d is None:
            continue
        sev = d.get("severity") or ""
        if sev == "error":
            errors += 1
        elif sev == "warning":
            warnings += 1
    total_errors += errors
    total_warnings += warnings
    print(f"{name}: {errors} errors, {warnings} warnings")
print(f"total: {total_errors} errors, {total_warnings} warnings")
