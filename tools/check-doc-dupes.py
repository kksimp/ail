#!/usr/bin/env python3
"""Check that the docs have no duplicate rows from merges.

Three places repeat a row per item, and a union merge can add a second copy:
the ADR index (`docs/decisions/README.md`, one row per ADR number), the glossary
(`docs/glossary.md`, one row per term) and the quick reference's `std.*`
table rows (`docs/quick-reference.md`, one row per module). This exits 1 when
any of them repeats.

Run with `python3 -I tools/check-doc-dupes.py` from the repository root, or
with `--root DIR` to check another tree (the fixture under
`tools/check-doc-dupes-fixture/` must fail: `--self-test` checks that).
"""
import os
import re
import sys

ADR_ROW = re.compile(r"^\| \[?(\d{4})")
GLOSSARY_ROW = re.compile(r"^\| \*\*([^*]+)\*\* \|")
STD_ROW = re.compile(r"^\| `(std\.[a-z_]+)` \|")


def duplicates(path, pattern):
    """Return the keys that appear on more than one matching row of `path`."""
    if not os.path.isfile(path):
        return []
    seen = {}
    with open(path, encoding="utf-8") as f:
        for lineno, line in enumerate(f, 1):
            m = pattern.match(line)
            if m:
                seen.setdefault(m.group(1), []).append(lineno)
    return [(path, k, lines) for k, lines in sorted(seen.items()) if len(lines) > 1]


def check(root):
    problems = []
    problems += duplicates(os.path.join(root, "docs/decisions/README.md"), ADR_ROW)
    problems += duplicates(os.path.join(root, "docs/glossary.md"), GLOSSARY_ROW)
    problems += duplicates(os.path.join(root, "docs/quick-reference.md"), STD_ROW)
    return problems


def main(argv):
    if argv[1:2] == ["--self-test"]:
        here = os.path.dirname(os.path.abspath(__file__))
        fixture = os.path.join(here, "check-doc-dupes-fixture")
        if not check(fixture):
            print("check-doc-dupes: the fixture has a duplicate but the check found none", file=sys.stderr)
            return 1
        print("check-doc-dupes: self-test ok (the fixture fails as it should)")
        return 0
    root = "."
    if argv[1:2] == ["--root"] and len(argv) > 2:
        root = argv[2]
    problems = check(root)
    for path, key, lines in problems:
        print(f"{path}: duplicate '{key}' on lines {', '.join(map(str, lines))}")
    if problems:
        print(f"check-doc-dupes: {len(problems)} duplicate(s)")
        return 1
    print("check-doc-dupes: no duplicate rows")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
