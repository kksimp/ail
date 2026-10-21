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


def ail(*args, stdin=None):
    return subprocess.run([AIL, *args], capture_output=True, text=True, input=stdin)


file = DATA / "effects.ail"
r = ail("effects", str(file))
users = {}
for line in lines_of(r.stdout):
    parts = line.split(" {")
    name = parts[0].replace("fn:", "")
    inner = parts[-1].replace("}", "")
    names = []
    if len(inner) > 0:
        names = inner.split(", ")
    for effect in names:
        users[effect] = users.get(effect, 0) + 1
    label = "pure" if len(names) == 0 else ", ".join(names)
    print(f"{name}: {label}")
for effect in sorted(users):
    print(f"{effect} used by {users[effect]}")
