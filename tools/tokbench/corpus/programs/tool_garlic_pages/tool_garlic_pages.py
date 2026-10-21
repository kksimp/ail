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


pages = DATA / "pages"
garlic = str(ROOT / "tools/garlic/garlic.ail")
all_words = 0
for name in sorted(os.listdir(pages)):
    r = ail("run", garlic, "--print", "--width", "60", str(pages / name))
    text = r.stdout.replace("\n", " ")
    words = len([w for w in text.split(" ") if len(w) > 0])
    all_words += words
    print(f"{name}: {len(lines_of(r.stdout))} lines, {words} words")
print(f"total words: {all_words}")
