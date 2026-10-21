import os
import shutil
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
DATA = ROOT / "tools/tokbench/corpus/agent-r5/data"
AIL = (shutil.which("ail") or str(ROOT / "ail"))


def ail(*args, stdin=None):
    return subprocess.run([AIL, *args], capture_output=True, text=True, input=stdin)


clean = 0
dirty = 0
for name in sorted(os.listdir(DATA)):
    if not name.endswith(".ail"):
        continue
    r = ail("fmt", "--check", str(DATA / name))
    if r.returncode == 0:
        clean += 1
        print(f"{name}: clean")
    else:
        dirty += 1
        print(f"{name}: needs formatting")
print(f"{clean} clean, {dirty} need formatting")
