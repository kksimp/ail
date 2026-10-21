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


text = (DATA / "rules.txt").read_text()
for rule in lines_of(text):
    r = ail("explain", rule)
    shown = r.stdout if r.returncode == 0 else r.stderr
    first = (lines_of(shown) or [""])[0]
    print(f"{rule} {r.returncode} {first}")
