"""Attribute AIL's token cost to construct classes, by ablation.

Measurement only (ADR 0070); not part of AIL. For each construct class, the
class is deleted (or replaced by its cheapest stand-in) from every program,
and the token difference from the original is that class's cost. The edited
text is not valid AIL; only the counts matter. A Python column gives the cost
of Python's own per-construct overhead for comparison.

Usage (from the repository root, after tools/tokbench/run.sh has set up once):
    tools/tokbench/.venv/bin/python tools/tokbench/attribute.py [--json] [DIR ...]

DIR defaults to every tools/tokbench/corpus/programs/*; each holds NAME.ail and
NAME.py. Counts are the mean of the offline tokenizers, as in tokbench.py.
"""

import json
import pathlib
import re
import sys

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from tokbench import offline_tokenizers  # noqa: E402

TYPE = r"[A-Za-z_][\w.]*(?:\[[^\]\n]*\])?\??"


def strip_sig_types(src):
    """Remove parameter and return types from `fn` lines (keep names)."""
    out = []
    for line in src.split("\n"):
        m = re.match(r"^(\s*(?:@\w+ )?fn [\w.]+(?:\[[^\]]*\])?)\((.*)\)(.*)$", line)
        if m:
            params = []
            for p in m.group(2).split(", ") if m.group(2) else []:
                words = p.split(" ")
                if words[0] == "mut":
                    params.append(" ".join(words[:2]))
                else:
                    params.append(words[0])
            line = f"{m.group(1)}({', '.join(params)})"
        out.append(line)
    return "\n".join(out)


def strip_return_types(src):
    return re.sub(r"^(\s*(?:@\w+ )?fn [^\n]*\))[^\n]*$", r"\1", src, flags=re.M)


def _keep_returns(src):
    """Remove parameter types only."""
    out = []
    for orig, stripped in zip(src.split("\n"), strip_sig_types(src).split("\n")):
        m = re.match(r"^\s*(?:@\w+ )?fn [^\n]*\)([^\n]*)$", orig)
        out.append(stripped + (m.group(1) if m and orig != stripped else ""))
    return "\n".join(out)


def strip_type_decls(src):
    """Remove struct fields' and variant fields' types."""
    out = []
    in_struct = False
    for line in src.split("\n"):
        if re.match(r"^(error )?struct ", line):
            in_struct = True
        elif not line.startswith(" "):
            in_struct = False
        elif in_struct and re.match(r"^    \w+ ", line):
            line = "    " + line.strip().split(" ")[0]
        out.append(line)
    src = "\n".join(out)
    # variant fields: Name(a int, b str) -> Name(a, b), only on enum variant lines
    return re.sub(r"^(    [A-Z]\w*)\(([^)\n]*)\)$",
                  lambda m: m.group(1) + "(" + ", ".join(p.split(" ")[0] for p in m.group(2).split(", ")) + ")",
                  src, flags=re.M)


def strip_method_owner(src):
    """`fn Parser.peek(` -> `fn peek(`: the owner prefix on method definitions."""
    return re.sub(r"^(\s*fn )[A-Z]\w*\.", r"\1", src, flags=re.M)


def strip_named_args(src):
    """`Point(x: 1)` -> `Point(1)`: labels in calls (not in format specs)."""
    return re.sub(r"(?<=[(,] )(\w+): (?!\})|(?<=\()(\w+): ", "", src)


ABLATIONS = [
    ("signature param types", _keep_returns),
    ("signature return types", strip_return_types),
    ("struct/variant field types", strip_type_decls),
    ("`fn` keyword", lambda s: re.sub(r"^(\s*(?:@\w+ )?)fn ", r"\1", s, flags=re.M)),
    ("method owner `T.` in `fn T.m`", strip_method_owner),
    ("`self` / `mut self` params", lambda s: re.sub(r"\((mut )?self(, )?", "(", s)),
    ("`:=` (vs `=`)", lambda s: s.replace(" := ", " = ")),
    ("`mut` on declarations", lambda s: re.sub(r"^(\s*)mut (\w+ :=)", r"\1\2", s, flags=re.M)),
    ("`mut` elsewhere", lambda s: re.sub(r"(?<![\w.])mut (?!\w+ :=)", "", s)),
    ("`use` lines", lambda s: re.sub(r"^use [^\n]*\n", "", s, flags=re.M)),
    ("`else` fallbacks (`x else y`)", lambda s: re.sub(r" else (?=\S)", " ", s)),
    ("`?` propagation", lambda s: re.sub(r"\?(?=[\s).,\]])", "", s)),
    ("`!` result markers", lambda s: re.sub(r"(?<=\) )!(?=\w|$)|(?<=\) )![^\n]*$", "", s, flags=re.M)),
    ("named call arguments", strip_named_args),
    ("`return` keyword", lambda s: re.sub(r"\breturn ", "", s)),
    ("`match` arms `=>`", lambda s: s.replace(" => ", " ")),
    ("indentation (leading spaces)", lambda s: re.sub(r"^ +", "", s, flags=re.M)),
    ("newlines", lambda s: s.replace("\n", " ")),
]

PY_ABLATIONS = [
    ("`self.` prefixes", lambda s: s.replace("self.", "")),
    ("`self` params", lambda s: re.sub(r"\(self(, )?", "(", s)),
    ("`def` keyword", lambda s: re.sub(r"^(\s*)def ", r"\1", s, flags=re.M)),
    ("`return` keyword", lambda s: re.sub(r"\breturn\b ?", "", s)),
    ("block colons", lambda s: re.sub(r":$", "", s, flags=re.M)),
    ("`raise`", lambda s: re.sub(r"\braise ", "", s)),
    ("`import` lines", lambda s: re.sub(r"^(import|from) [^\n]*\n", "", s, flags=re.M)),
    ("indentation (leading spaces)", lambda s: re.sub(r"^ +", "", s, flags=re.M)),
    ("newlines", lambda s: s.replace("\n", " ")),
]


def main():
    as_json = "--json" in sys.argv
    dirs = [pathlib.Path(a) for a in sys.argv[1:] if a != "--json"]
    if not dirs:
        dirs = sorted(p for p in (HERE / "corpus" / "programs").iterdir() if p.is_dir())
    counters = offline_tokenizers()

    def count(text):
        return sum(fn(text) for fn in counters.values()) / len(counters)

    rows = {"ail": {}, "py": {}}
    totals = {"ail": 0.0, "py": 0.0}
    per_program = []
    for d in dirs:
        name = d.name
        ail = (d / f"{name}.ail").read_text()
        py = (d / f"{name}.py").read_text()
        a, p = count(ail), count(py)
        totals["ail"] += a
        totals["py"] += p
        per_program.append({"program": name, "ail": round(a, 1), "py": round(p, 1),
                            "ratio": round(a / p, 3)})
        for side, src, base, abl in (("ail", ail, a, ABLATIONS), ("py", py, p, PY_ABLATIONS)):
            for label, fn in abl:
                rows[side][label] = rows[side].get(label, 0.0) + base - count(fn(src))

    if as_json:
        print(json.dumps({"programs": per_program, "totals": totals, "costs": rows}, indent=2))
        return
    print(f"{'program':<12}{'AIL':>10}{'Python':>10}{'AIL/Py':>8}")
    for r in per_program:
        print(f"{r['program']:<12}{r['ail']:>10}{r['py']:>10}{r['ratio']:>8}")
    print(f"{'total':<12}{totals['ail']:>10.1f}{totals['py']:>10.1f}{totals['ail'] / totals['py']:>8.3f}")
    for side, title in (("ail", "AIL"), ("py", "Python")):
        print(f"\n{title} cost by construct (tokens, share of total)")
        for label, v in sorted(rows[side].items(), key=lambda kv: -kv[1]):
            print(f"  {label:<36}{v:>9.1f}{100 * v / totals[side]:>7.1f}%")


if __name__ == "__main__":
    main()
