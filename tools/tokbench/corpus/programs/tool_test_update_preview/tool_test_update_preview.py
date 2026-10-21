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


r = ail("test", "--update", "--dry-run", str(DATA / "tests.ail"))
added = 0
removed = 0
for line in lines_of(r.stdout):
    if line.startswith("+") and not line.startswith("+++"):
        added += 1
        print(line)
    elif line.startswith("-") and not line.startswith("---"):
        removed += 1
        print(line)
print(f"{added} lines added, {removed} removed")
