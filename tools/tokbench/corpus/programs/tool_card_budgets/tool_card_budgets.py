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


pkg = DATA / "pkg"
for budget in [60, 120, 240, 480]:
    r = ail("api", "--card", "--budget", str(budget), str(pkg))
    print(f"budget {budget}: {len(lines_of(r.stdout))} lines, {len(r.stdout)} chars")
