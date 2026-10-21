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


edits = (DATA / "edit.txt").read_text()
r = ail("edit", str(DATA / "ok.ail"), "--dry-run", "--json", stdin=edits)
for line in lines_of(r.stdout):
    d = parse(line)
    if d is not None and d.get("summary") is True:
        written = "true" if d.get("written", True) else "false"
        print(f"edits {d.get('edits', 0)}, errors {d.get('errors', 0)}, warnings {d.get('warnings', 0)}, written {written}")
