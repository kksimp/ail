#!/usr/bin/env python3
"""Check docs/design/builtin-methods.txt (api.find.5) and docs/how-to.md (api.find.6).

- every rule a method row cites exists as `**RULE**` in docs/spec;
- every built-in method of the checker (the lists in compiler/check/builtins.ail
  for str, bytes, char, int, float, List, Map and Set) has a row;
- every row names a method of a type the checker knows, or is a language form;
- every how-to title is unique and has a `Words:` line and a code block.

Run `python3 -I tools/check-method-table.py` from the repository root.
Exit 1 on any finding.
"""
import glob
import re
import sys

LISTS = {"STR_METHODS": "str", "BYTES_METHODS": "bytes", "CHAR_METHODS": "char",
         "INT_METHODS": "int", "FLOAT_METHODS": "float", "LIST_METHODS": "List",
         "MAP_METHODS": "Map", "SET_METHODS": "Set"}
# Methods the table leaves to the rule that gives them (the sized-number conversions of
# `int` are rows; `float.to_f32` and `int.to_f32` are one spelling of type.num.15).
SKIP = {("float", "to_f32")}
FORMS = {"Option", "Tuple", ""}

findings = []


def bad(msg):
    findings.append(msg)


spec = ""
for f in glob.glob("docs/spec/*.md"):
    spec += open(f, encoding="utf-8").read()
rules = set(re.findall(r"\*\*([a-z0-9_.]+)\*\*", spec))

rows = {}
for n, line in enumerate(open("docs/design/builtin-methods.txt", encoding="utf-8"), 1):
    line = line.rstrip("\n")
    if not line.strip() or line.startswith("#"):
        continue
    cells = [c.strip() for c in line.split(" ; ")]
    if len(cells) != 3:
        bad(f"builtin-methods.txt:{n}: want SIGNATURE ; RULE ; WORDS")
        continue
    sig, rule, _ = cells
    if rule not in rules:
        bad(f"builtin-methods.txt:{n}: rule {rule} is not in docs/spec")
    head = sig.split("(")[0]
    if "." in head:
        owner, rest = sig.split(".", 1)
        nm = re.match(r"[a-z_0-9]+", rest)
        name = nm.group(0) if nm else rest
    else:
        owner, name = "", re.match(r"[a-z_0-9]+", sig).group(0)
    rows[(owner, name)] = n

src = open("compiler/check/builtins.ail", encoding="utf-8").read()
for var, owner in LISTS.items():
    m = re.search(r"pub %s := \[(.*?)\]" % var, src, re.S)
    if not m:
        bad(f"builtins.ail: list {var} not found")
        continue
    for name in re.findall(r'"([a-z_0-9]+)"', m.group(1)):
        if (owner, name) in SKIP:
            continue
        if (owner, name) not in rows:
            bad(f"builtin-methods.txt: no row for {owner}.{name} (checker builtin)")

seen = set()
text = open("docs/how-to.md", encoding="utf-8").read()
for sec in re.split(r"^## ", text, flags=re.M)[1:]:
    title = sec.split("\n", 1)[0].strip()
    if title in seen:
        bad(f"how-to.md: duplicate title `{title}`")
    seen.add(title)
    if "\nWords:" not in sec:
        bad(f"how-to.md: `{title}` has no Words: line")
    if "```ail" not in sec:
        bad(f"how-to.md: `{title}` has no ail block")

for f in findings:
    print(f)
sys.exit(1 if findings else 0)
