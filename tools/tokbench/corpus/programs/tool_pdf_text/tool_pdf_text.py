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


def ail(*args, stdin=None):
    return subprocess.run([AIL, *args], capture_output=True, text=True, input=stdin)


pdfs = DATA / "pdf"
reader = str(ROOT / "tools/pdf/pdf.ail")
for name in sorted(os.listdir(pdfs)):
    r = ail("run", reader, "text", str(pdfs / name))
    words = len([w for w in r.stdout.replace("\n", " ").split(" ") if len(w) > 0])
    print(f"{name}: status {r.returncode}, {len(lines_of(r.stdout))} lines, {words} words")
