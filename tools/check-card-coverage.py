#!/usr/bin/env python3
"""Check that the agent card covers the language (ADR 1110).

Fails when
  * a built-in method row of docs/design/builtin-methods.txt has its method name on no line of
    docs/language-card.md or docs/language-card-core.md, or
  * a rule family (the first two parts of a rule ID, `type.str`) of a language chapter has no rule
    cited in the cards, in docs/how-to.md or in the method table's RULE column,
unless docs/design/card-coverage-exceptions.txt names it with a reason. One exception per line:
`method Type.name ; reason` or `family a.b ; reason`. An exception that is no longer needed is an
error too, so the list can only shrink. `--write-baseline` rewrites the list from the current state.
Measurement-style tool in Python, like tokbench (ADR 0070); no arguments needed.
"""
import glob, re, sys, pathlib

ROOT = pathlib.Path(__file__).resolve().parents[1]
# Chapters whose families are the language the card teaches. Tool chapters are covered by `ail help`.
LANG = {"lex", "gram", "type", "expr", "own", "trait", "err", "conc", "mod", "ct", "eff", "test", "std",
        "contract", "gen", "json", "unsafe", "ffi", "mem", "refl", "http"}
EXC = ROOT / "docs/design/card-coverage-exceptions.txt"


def read(p):
    return (ROOT / p).read_text(encoding="utf-8")


def main():
    cards = read("docs/language-card.md") + read("docs/language-card-core.md")
    how = read("docs/how-to.md")
    table = [l for l in read("docs/design/builtin-methods.txt").splitlines() if l.strip() and not l.startswith("#")]
    rule_col = " ".join(l.split(";")[1].strip() for l in table if l.count(";") >= 2)
    seen_text = cards + "\n" + how + "\n" + rule_col
    need = {}
    for l in table:
        sig = l.split(";")[0].strip()
        m = re.match(r"([\w\[\], ]+?)\.(\w+)\(", sig)
        if m and not re.search(r"\b" + m.group(2) + r"\b", cards):
            need[f"method {m.group(1)}.{m.group(2)}"] = True
    fams = set()
    for f in glob.glob(str(ROOT / "docs/spec/*.md")):
        for m in re.finditer(r"^\*\*([a-z0-9_]+(?:\.[a-z0-9_]+)+)\.\d+\*\*", pathlib.Path(f).read_text(encoding="utf-8"), re.M):
            fam = m.group(1)
            if fam.split(".")[0] in LANG:
                fams.add(fam)
    for fam in sorted(fams):
        if fam + "." not in seen_text:
            need[f"family {fam}"] = True
    if "--write-baseline" in sys.argv:
        old = {}
        if EXC.exists():
            for l in EXC.read_text(encoding="utf-8").splitlines():
                if ";" in l and not l.startswith("#"):
                    k, r = l.split(";", 1)
                    old[k.strip()] = r.strip()
        out = ["# Cards' coverage exceptions (ADR 1110): `method Type.name ; reason` or `family a.b ; reason`.",
               "# tools/check-card-coverage.py fails on a gap listed nowhere here, and on a line here that is no longer a gap.",
               "# Give each entry a reason; remove it when a card line, a how-to or a method-table row covers it."]
        for k in sorted(need):
            out.append(f"{k} ; {old.get(k, 'not on a card line yet; `ail ask` finds it')}")
        EXC.write_text("\n".join(out) + "\n", encoding="utf-8")
        print(f"wrote {len(need)} exceptions")
        return 0
    exc = {}
    for n, l in enumerate(EXC.read_text(encoding="utf-8").splitlines(), 1) if EXC.exists() else []:
        if not l.strip() or l.startswith("#"):
            continue
        if ";" not in l or not l.split(";", 1)[1].strip():
            print(f"card-coverage-exceptions.txt:{n}: every exception needs a reason")
            return 1
        exc[l.split(";")[0].strip()] = n
    bad = 0
    for k in sorted(need):
        if k not in exc:
            print(f"card coverage: {k} is on no card line, in no how-to and in no method-table row; add it or list it in {EXC.relative_to(ROOT)}")
            bad += 1
    for k, n in exc.items():
        if k not in need:
            print(f"card-coverage-exceptions.txt:{n}: `{k}` is covered now; remove the line")
            bad += 1
    print(f"card coverage: {len(need)} gaps, {len(exc)} exceptions, {bad} problems")
    return 1 if bad else 0


sys.exit(main())
