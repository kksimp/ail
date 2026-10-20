import os
import subprocess
import sys
from dataclasses import dataclass, field
@dataclass
class Stats:
    lines: int = 0
    blank: int = 0
    comments: int = 0
    functions: list = field(default_factory=list)
def measure(text):
    stats = Stats()
    for line in text.splitlines():
        stats.lines += 1
        t = line.strip()
        if t == "":
            stats.blank += 1
        elif t.startswith("##"):
            stats.comments += 1
        elif t.startswith("fn "):
            stats.functions.append(t[3:].split("(")[0])
    return stats
def main():
    path = sys.argv[1]
    with open(path) as f:
        stats = measure(f.read())
    name = os.path.basename(path)
    print(f"file: {name}")
    print(f"lines: {stats.lines} ({stats.blank} blank, {stats.comments} comments)")
    print(f"functions: {', '.join(stats.functions)}")
    echo = subprocess.run(["echo", "measured", name], capture_output=True, text=True)
    print(f"echo said: {echo.stdout.strip()} (status {echo.returncode})")
    try:
        open(path + ".missing").read()
        missing = "found"
    except FileNotFoundError:
        missing = "not_found"
    print(f"a missing file is {missing}")
main()
