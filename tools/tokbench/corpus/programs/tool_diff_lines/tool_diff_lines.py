import json
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


for name in ["scale.ail", "report.ail"]:
    r = ail("diff", "--json", str(DATA / "v1" / name), str(DATA / "v2" / name))
    for line in lines_of(r.stdout):
        d = parse(line)
        if d is None:
            continue
        if d.get("summary") is not True:
            op = d.get("op") or ""
            ident = d.get("id") or ""
            touched = len(d.get("lines") or [])
            print(f"{op} {ident} lines {touched}")
