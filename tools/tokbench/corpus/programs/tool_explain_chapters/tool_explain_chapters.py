import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]

AIL = str(ROOT / "ail")


def ail(*args, stdin=None):
    return subprocess.run([AIL, *args], capture_output=True, text=True, input=stdin)


r = ail("explain", "--json")
doc = json.loads(r.stdout)
rules = doc.get("rules", [])
total = {}
fixable = {}
for rule in rules:
    chapter = rule.get("chapter") or "?"
    total[chapter] = total.get(chapter, 0) + 1
    if rule.get("fix") is True:
        fixable[chapter] = fixable.get(chapter, 0) + 1
for chapter in sorted(total):
    print(f"{chapter} {total[chapter]} {fixable.get(chapter, 0)}")
print(f"all {len(rules)} {doc.get('fixable', 0)}")
