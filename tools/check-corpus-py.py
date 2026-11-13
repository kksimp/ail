#!/usr/bin/env python3
"""Runs every corpus/NAME/NAME.py and checks its output equals NAME.out (the Python twin is the oracle).
Usage: python3 tools/check-corpus-py.py [--write] [NAME...]   (--write regenerates the .out files)."""
import os, subprocess, sys

root = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "corpus")
args = [a for a in sys.argv[1:] if a != "--write"]
write = "--write" in sys.argv
names = args or sorted(d for d in os.listdir(root) if os.path.isdir(os.path.join(root, d)))
bad = 0
for n in names:
    d = os.path.join(root, n)
    for f in (n + ".ail", n + ".py"):
        if not os.path.exists(os.path.join(d, f)):
            print("MISSING", n + "/" + f); bad += 1
    py = os.path.join(d, n + ".py")
    if not os.path.exists(py):
        continue
    r = subprocess.run([sys.executable, "-I", py], capture_output=True, text=True, timeout=60, cwd=d)
    outp = os.path.join(d, n + ".out")
    if write and r.returncode == 0:
        open(outp, "w", newline="\n").write(r.stdout)
    want = open(outp).read() if os.path.exists(outp) else None
    if r.returncode != 0 or r.stdout != want:
        print("RED", n, "exit", r.returncode, r.stderr[:200]); bad += 1
    if "@test" not in open(os.path.join(d, n + ".ail")).read():
        print("NO @test", n); bad += 1
print("corpus twins:", len(names), "checked,", bad, "red")
sys.exit(1 if bad else 0)
