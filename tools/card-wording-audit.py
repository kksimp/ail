"""Check the language card's calls against the spec (lane 217, ADR 0698).

    python3 -I tools/card-wording-audit.py [ROOT]

Every call the language card writes as `name(args)` or `mod.name(args)` must
be a call the spec also writes with the same number of arguments. A card call
with no spec call of that name is printed as "not in spec"; one whose name
is in the spec only with other arities is printed as "arity". A name the spec writes only in a list or in backticks (not in call form) counts
as mentioned, and its arity is not compared. Exit status 1
when either list is not empty.

Arguments are counted at the top level, so `re, s` is 2 and `f(a, b), c` is 2.
Names the card declares itself (`fn name` in the card) are skipped. Names that
are not functions (`print`, keywords) show as "not in spec" and are listed for
review.
"""
import pathlib
import re
import sys

root = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()


def arity(args):
    args = args.strip()
    if not args:
        return 0
    depth, count = 0, 1
    quote = False
    for ch in args:
        if ch == '"':
            quote = not quote
        elif quote:
            continue
        elif ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
        elif ch == "," and depth == 0:
            count += 1
    return count


def calls(text):
    """(name, arity) for each `name(args)` with the args on one line."""
    found = []
    for m in re.finditer(r"(?<![\w])([A-Za-z_][\w]*(?:\.[A-Za-z_]\w*)?)\(([^()\n]*)\)", text):
        name = m.group(1).split(".")[-1]
        found.append((name, arity(m.group(2))))
    return found


card_text = (root / "docs/language-card.md").read_text(encoding="utf-8")
spec_text = "\n".join(p.read_text(encoding="utf-8") for p in sorted((root / "docs/spec").glob("*.md")))

spec_arities = {}
for name, n in calls(spec_text):
    spec_arities.setdefault(name, set()).add(n)

# Names the card declares itself (its example functions and methods) are not
# std calls, so they are skipped.
declared = set(re.findall(r"\bfn\s+(?:[^(\n]*?\.)?([A-Za-z_]\w*)\s*\(", card_text))
# Variant and struct constructors (`Rgb(...)`) are not std calls either.
declared |= {n for n in re.findall(r"(?<![\w.])([A-Z]\w*)\(", card_text)}
# Spec names a function in a list without its call form (`to_u8`, `need_int`):
# that is a mention; only the arity check is skipped for those.
spec_words = set(re.findall(r"`([A-Za-z_]\w*)`", spec_text)) | set(re.findall(r"(?<![\w.])([a-z_]\w*)\b(?=[,\s]+(?:and|or)\b)", spec_text))

problems = []
for name, n in sorted(set(calls(card_text))):
    if name in declared:
        continue
    if name not in spec_arities and name in spec_words:
        continue
    if name not in spec_arities:
        problems.append(f"not in spec: {name}({n} args)")
    elif n not in spec_arities[name]:
        problems.append(f"arity: {name} has {n} args on the card, spec has {sorted(spec_arities[name])}")
for p in problems:
    print(p)
print(f"card calls checked: {len(set(calls(card_text)))}; problems: {len(problems)}")
sys.exit(1 if problems else 0)
