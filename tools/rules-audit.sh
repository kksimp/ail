#!/usr/bin/env bash
# Audit the spec rule IDs against the code (ADR 0002; lane 146, ADRs 0483-0485).
#
#   tools/rules-audit.sh
#
# Checks six things and exits 1 when any is not empty:
#   1. cited but not in the spec: a rule ID named in a comment or a string
#      literal of compiler/ that docs/spec/ does not define;
#   2. spec rules never cited: a rule in docs/spec/ that no comment or string
#      literal in compiler/ names and no spec block the spec suite runs lists in `rules=` (`[removed in ...]`
#      rules are skipped; they stay in place by the spec README, rule 2;
#      `[definitional]` rules describe the language and need no code);
#   3. diagnostics with no `ail explain` entry: a rule ID emitted as a string
#      literal (a diagnostic's rule) that `ail explain` cannot explain;
#   4. `ail explain` entries with no example: a rule whose explanation has no
#      `example` (checked through `ail explain RULE --json`);
#   5. std functions with no spec mention: a function in the MODULES table of
#      compiler/check/stdlib.ail that docs/spec/ never names (tools/card-spec-audit.py,
#      ADR 0698);
#   6. test cases: duplicate programs (same program modulo names, same
#      companion files) and diagnostic cases with no `.rules` file
#      (tools/cases-audit.py, ADR 0725).
#
# Ranges such as `eff.cap.4-5` count as citations of each ID in the range.
# Self-setting-up (ADR 0071): the first `./ail explain` builds the native CLI
# when it is stale. Needs only python3, which both platforms already have.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

# Build (if stale) and check the binary answers before auditing.
./ail explain --json > /dev/null

exec python3 -I - "$root" <<'PY'
import json
import pathlib
import re
import subprocess
import sys

root = pathlib.Path(sys.argv[1])
ail = root / "ail"

RULE = re.compile(r"(?<![\w.])([a-z_]+\.[a-z_]+\.[0-9]+)(?:-([0-9]+))?(?!\w)(?!\.\d)")
HEADED = re.compile(r"^\*\*([a-z_]+\.[a-z_]+\.[0-9]+)\*\*")
QUOTED = re.compile(r'"([a-z_]+\.[a-z_]+\.[0-9]+)"')


def expand(match):
    """The rule IDs a citation names: one, or every number in a `a-b` range."""
    base, end = match.group(1), match.group(2)
    if end is None:
        return [base]
    stem, start = base.rsplit(".", 1)
    lo, hi = int(start), int(end)
    return [f"{stem}.{n}" for n in range(lo, hi + 1)] if lo <= hi else [base]


# 1. The spec's rules.
spec = {}  # rule -> file name
removed = set()
for path in sorted((root / "docs" / "spec").glob("*.md")):
    for line in path.read_text(encoding="utf-8").splitlines():
        m = HEADED.match(line)
        if m:
            spec.setdefault(m.group(1), path.name)
            if "[removed in" in line:
                removed.add(m.group(1))
            # `[definitional]`: a rule that describes the language, not a behaviour the code has to implement.
            if "[definitional]" in line:
                removed.add(m.group(1))

# 2. Citations in code: comments name rules, string literals emit them.
cited = {}  # rule -> set of files that cite it (comments or literals)
emitted = {}  # rule -> set of files that emit it as a literal
sources = ([root / "tools" / "gate" / "gate.ail"] + list((root / "compiler").rglob("*.ail")) + list((root / "std").rglob("*.ail"))
           + list((root / "tools" / "ailtest").rglob("*.ail")) + list((root / "tools" / "ailtask").rglob("*.ail")) + list((root / "tools" / "ailmerge").rglob("*.ail"))
           + list((root / "tools" / "ailmcp").rglob("*.ail"))
           + list((root / "tools" / "ailsp").rglob("*.ail")) + list((root / "tools" / "ailsan").rglob("*.ail"))
           + list((root / "tools" / "verify").rglob("*.ail")) + list((root / "tools" / "infer").rglob("*.ail")) + list((root / "tools" / "shrink").rglob("*.ail")))
for path in sources:
    rel = str(path.relative_to(root))
    for line in path.read_text(encoding="utf-8").splitlines():
        comment = line if line.lstrip().startswith("#") else ""
        if comment:
            for m in RULE.finditer(comment):
                for rule in expand(m):
                    # gate.ail is read for its explain suite only.
                    if rel != "tools/gate/gate.ail" or rule.startswith("explain.cover."):
                        cited.setdefault(rule, set()).add(rel)
        if rel == "tools/gate/gate.ail":
            continue  # its explain suite's string literals are deliberately bad rule IDs; its comments are read above
        for m in QUOTED.finditer(line):
            cited.setdefault(m.group(1), set()).add(rel)
            emitted.setdefault(m.group(1), set()).add(rel)

# A spec block that the spec suite runs (no `skip=`) and names rules with `rules=` is a conformance test of them (ADR 0002).
for path in sorted((root / "docs" / "spec").glob("*.md")):
    for m in re.finditer(r"^```ail ([^\n]*)$", path.read_text(encoding="utf-8"), re.M):
        info = m.group(1)
        named = re.search(r"rules=(\S+)", info)
        if named and "skip=" not in info:
            for rule in named.group(1).split(","):
                cited.setdefault(rule, set()).add(str(path.relative_to(root)))

problems = 0


def report(title, items):
    global problems
    problems += len(items)
    print(f"{title}: {len(items)}")
    for item in items:
        print(f"  {item}")


report(
    "1. cited but not in docs/spec/",
    [f"{r}  ({', '.join(sorted(cited[r]))})" for r in sorted(set(cited) - set(spec))],
)
# Check 2 exemptions are the rows of the "## Check 2 exemptions" table in
# docs/design/rule-coverage.md (lane 244): the one list the audit reads. A row
# whose rule is now cited, or that has no owner or reason, is a finding.
coverage = (root / "docs" / "design" / "rule-coverage.md").read_text()
section = coverage.split("## Check 2 exemptions", 1)[1].split("\n## ", 1)[0] if "## Check 2 exemptions" in coverage else ""
exempt, incomplete = {}, []
for line in section.splitlines():
    m = re.match(r"^\| `([a-z0-9.]+)` \| ([^|]*)\| ([^|]*)\|$", line)
    if m:
        rule, owner, reason = m.group(1), m.group(2).strip(), m.group(3).strip()
        exempt[rule] = owner
        if not owner or not reason:
            incomplete.append(rule)
report(
    "2. spec rules never cited in compiler/, std/, tools/ailtest/, tools/verify/, tools/infer/ or tools/shrink/ (after exemptions)",
    [f"{r}  ({spec[r]})" for r in sorted(set(spec) - set(cited) - removed) if r not in exempt],
)
report("2a. exemptions for rules that are now cited (remove the row)", sorted(r for r in exempt if r in cited))
report("2b. exemptions with no owner or reason", incomplete)

# 3 and 4 need `ail explain`, which reads the spec it was built from.
listing = json.loads(subprocess.run([str(ail), "explain", "--json"], capture_output=True, text=True, check=True).stdout)
explained = {entry["rule"] for entry in listing["rules"]}
report(
    "3. diagnostics with no `ail explain` entry",
    [f"{r}  ({', '.join(sorted(emitted[r]))})" for r in sorted(set(emitted) - explained)],
)

no_example = []
for rule in sorted(explained):
    out = subprocess.run([str(ail), "explain", rule, "--json"], capture_output=True, text=True, check=True).stdout
    entry = json.loads(out)
    # A worked example (error=RULE, printed as `rejected`) replaces the nearest
    # plain example in the JSON, so both count (ADR 0670).
    if entry.get("example") in (None, "") and entry.get("rejected") in (None, ""):
        no_example.append(rule)
report("4. `ail explain` entries with no example", [f"{r}  ({spec.get(r, '?')})" for r in no_example])

# 5. The std functions against the spec (ADR 0698).
audit = subprocess.run(
    [sys.executable, "-I", str(root / "tools" / "card-spec-audit.py"), str(root)],
    capture_output=True, text=True, check=False,
)
report("5. std functions with no spec mention", [l.removeprefix("no spec mention: ") for l in audit.stdout.splitlines() if l.startswith("no spec mention: ")])

# 6. The test cases (ADR 0725): duplicates and missing .rules.
cases = subprocess.run([sys.executable, "-I", str(root / "tools" / "cases-audit.py"), str(root)], capture_output=True, text=True, check=False)
report("6. test cases: duplicates or missing .rules", [l.strip() for l in cases.stdout.splitlines()[1:]])

print(f"rules in spec: {len(spec)}; explained: {len(explained)}; findings: {problems}")
sys.exit(1 if problems else 0)
PY
