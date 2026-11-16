#!/usr/bin/env python3
"""Audit tests/cases (ADR 0725; rules-audit check 6).

    python3 -I tools/cases-audit.py ROOT

Reports, and exits 1 when there is any finding:
  duplicate:     two programs the same modulo identifier names, with the same
                 companion files (`.out`, `.rules`, ...), so one tests nothing new;
  missing .rules: a case with no `.rules`, `.test` or `.trap` file whose
                 `ail check --json` reports a diagnostic, so it tests a diagnostic
                 without naming its rule ID.
"""
import hashlib, json, os, pathlib, re, subprocess, sys
from collections import defaultdict

# ADR 1130: the `ail check` runs here must not apply their certain fixes to the inputs.
os.environ.setdefault("AIL_FIX", "0")

KEYWORDS = set("fn mut if else for in while return struct enum match use pub let true false main print assert Map List int str float bool".split())
root = pathlib.Path(sys.argv[1])
cases = root / "bootstrap" / "tests" / "cases"
ail = root / "ail"

def normal(text):
    text = re.sub(r"(?m)#.*$", "", text)
    names = {}
    out = []
    for tok in re.findall(r'"(?:\\.|[^"\\])*"|[A-Za-z_]\w*|\d[\d.]*|\S', text):
        if re.match(r"[A-Za-z_]", tok) and tok not in KEYWORDS:
            out.append(names.setdefault(tok, f"v{len(names)}"))
        else:
            out.append(tok)
    return " ".join(out)

programs = sorted(cases.glob("*.ail"))
groups = defaultdict(list)
for path in programs:
    companions = {p.name[len(path.stem):]: p.read_bytes() for p in cases.glob(path.stem + ".*") if p != path}
    key = hashlib.sha256((normal(path.read_text()) + repr(sorted(companions.items()))).encode()).hexdigest()
    groups[key].append(path.name)
findings = [f"duplicate: {', '.join(names)}" for names in groups.values() if len(names) > 1]

for path in programs:
    # Test and trap cases name their expectation in another file; a module
    # without `main` is not a program (mod.main.1 is not a diagnostic case).
    if any((cases / (path.stem + ext)).exists() for ext in (".rules", ".test", ".trap")):
        continue
    run = subprocess.run([str(ail), "check", "--json", str(path)], capture_output=True, text=True, check=False)
    rules = []
    for line in run.stderr.splitlines():
        try:
            rule = json.loads(line).get("rule")
        except (ValueError, AttributeError):
            continue
        if rule and rule != "mod.main.1":
            rules.append(rule)
    if rules:
        findings.append(f"missing .rules: {path.name}  (reports {', '.join(sorted(set(rules)))})")

print(f"cases: {len(programs)}; findings: {len(findings)}")
for finding in findings:
    print(f"  {finding}")
sys.exit(1 if findings else 0)
