import json
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]


def to_finding(doc):
    if not isinstance(doc, dict):
        return None
    severity, rule, line = doc.get("severity"), doc.get("rule"), doc.get("line")
    if not (isinstance(severity, str) and isinstance(rule, str) and isinstance(line, (int, float))):
        return None
    return severity, rule, int(line)


def parse_findings(report):
    found = []
    for line in report.splitlines():
        try:
            finding = to_finding(json.loads(line))
        except json.JSONDecodeError:
            continue
        if finding:
            found.append(finding)
    return found


def count(found, severity):
    return sum(1 for f in found if f[0] == severity)


def main():
    errors = warnings = 0
    for path in sorted((ROOT / "examples").glob("*.ail")):
        r = subprocess.run([shutil.which("ail") or str(ROOT / "ail"), "check", "--json", path], capture_output=True, text=True)
        found = parse_findings(r.stderr)
        errors += count(found, "error")
        warnings += count(found, "warning")
        for severity, rule, line in found:
            print(f"{path.name}:{line} {severity} {rule}")
    print(f"{errors} errors, {warnings} warnings")
    if errors:
        sys.exit(1)


def test_counts_by_severity():
    report = '{"severity":"error","rule":"a.b.1","line":3}\nnot json\n{"severity":"warning","rule":"c.d.2","line":9}'
    found = parse_findings(report)
    assert len(found) == 2 and count(found, "error") == 1 and found[1][2] == 9


def test_empty_report_has_no_findings():
    assert parse_findings("") == []


if __name__ == "__main__":
    main()
