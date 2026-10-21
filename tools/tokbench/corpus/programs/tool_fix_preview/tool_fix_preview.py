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


for name in ["bad_type.ail", "bad_name.ail"]:
    r = ail("fix", "--dry-run", str(DATA / name))
    errors = 0
    for line in lines_of(r.stdout + r.stderr):
        if "error[" in line:
            errors += 1
    out_lines = lines_of(r.stdout)
    last = out_lines[-1] if out_lines else ""
    print(f"{name}: {errors} error lines; {last}")
