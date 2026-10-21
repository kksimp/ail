# Python twin of examples/std_args.ail: the same spec, parse and output.
import json
import sys

USAGE = """usage: count [--json] [--out] INPUT [MORE...]
count lines

  -j, --json            print JSON
  -o, --out V           output file (default -)
  INPUT                 input file
  MORE                  extra files
  -h, --help            show this help
"""

words = [
    "--json",
    "-o",
    "report.txt",
    "data.csv",
    "a.txt",
    "b.txt",
]

flags = {"json": False}
opts = {"out": "-"}
positional = []
rest = []
i = 0
while i < len(words):
    w = words[i]
    if w in ("--json", "-j"):
        flags["json"] = True
    elif w in ("--out", "-o"):
        i += 1
        opts["out"] = words[i]
    elif w.startswith("-"):
        sys.exit(f"unknown flag {w}")
    else:
        (positional if len(positional) < 1 else rest).append(w)
    i += 1

print(f"json={str(flags['json']).lower()} out={opts['out']} input={positional[0]}")
print(f"more={json.dumps(rest)}")
print(USAGE)
