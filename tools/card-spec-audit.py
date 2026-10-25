"""Audit the std functions against the spec (lane 217, ADR 0698).

    python3 -I tools/card-spec-audit.py [ROOT]

One list, printed one name per line, then a count:

1. std functions with no spec mention: every function in the `MODULES`
   table of compiler/check/stdlib.ail whose name appears as a word nowhere
   in docs/spec/ (the spec writes `find_all(re, text)` without the module).

Exit status 1 when either list is not empty. The table is read from the
checker's source, so the checker and the audit cannot drift apart.
"""
import pathlib
import re
import sys

root = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
src = (root / "compiler/check/stdlib.ail").read_text(encoding="utf-8")
table = src.split("STD_FUNCTIONS := [", 1)[1].split("\n]\n", 1)[0]
modules = {}
for m in re.finditer(r"\[([^\[\]]*)\]", table):
    names = re.findall(r'"(\w+)"', m.group(1))
    if names:
        modules.setdefault(names[0], set()).update(names[1:])

spec_text = "\n".join(p.read_text(encoding="utf-8") for p in sorted((root / "docs/spec").glob("*.md")))
words = set(re.findall(r"(?<!\w)(\w+)\(", spec_text)) | set(re.findall(r"`(\w+)`", spec_text))
missing = sorted((mod, fn) for mod, fns in modules.items() for fn in fns if fn not in words)
for mod, fn in missing:
    print(f"no spec mention: {mod}.{fn}")
total = sum(len(f) for f in modules.values())
print(f"functions: {total}; with no spec mention: {len(missing)}")
sys.exit(1 if missing else 0)
