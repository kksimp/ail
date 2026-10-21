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


def ail(*args, stdin=None):
    return subprocess.run([AIL, *args], capture_output=True, text=True, input=stdin)


rec = DATA / "replay.rec"
program = str(DATA / "replay_fixture.ail")
recorded = ail("run", "--record", str(rec), program)
replayed = ail("run", "--replay", str(rec), program)
print(f"recorded: status {recorded.returncode}, {len(lines_of(recorded.stdout))} lines")
print(f"replayed: status {replayed.returncode}, {len(lines_of(replayed.stdout))} lines")
same = "true" if recorded.stdout == replayed.stdout else "false"
print(f"same output: {same}")
os.remove(rec)
