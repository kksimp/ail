import json
import os
import shutil
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
DATA = ROOT / "tools/tokbench/corpus/agent-r5/data"
AIL = (shutil.which("ail") or str(ROOT / "ail"))


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


pdfs = DATA / "pdf"
reader = str(ROOT / "tools/pdf/pdf.ail")
for name in sorted(os.listdir(pdfs)):
    r = ail("run", reader, "grep", "--json", "ledger", str(pdfs / name))
    hits = 0
    for line in lines_of(r.stdout):
        d = parse(line)
        if d is None:
            continue
        hits += 1
        print(f"{name} page {float(d.get('page', 0.0))} : {d.get('text') or ''}")
    print(f"{name}: {hits} hits")
