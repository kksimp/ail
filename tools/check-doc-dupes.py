#!/usr/bin/env python3
"""Check that the docs have no duplicate rows from merges.

Three places repeat a row per item, and a union merge can add a second copy:
the ADR index (`docs/decisions/README.md`, one row per ADR number), the glossary
(`docs/glossary.md`, one row per term) and the quick reference's `std.*`
table rows (`docs/quick-reference.md`, one row per module). This exits 1 when
any of them repeats.

The quick reference and the language card get a second check for exact repeats,
which a union merge also leaves behind (`SQUARES := squares(8)` twice in one
example):
- in one code fence, two identical lines at column 0 that are 8 or more
  characters long (a repeated `else` or `fn` line is not flagged);
- under one heading, two identical table rows, list items or whole code fences.
Lines that differ only in a comment are not flagged.

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


def card_duplicates(path):
    """Return (path, text, lines) for each exact repeat in a card (see the module doc)."""
    if not os.path.isfile(path):
        return []
    with open(path, encoding="utf-8") as f:
        lines = f.read().split("\n")
    found = []
    in_fence = False
    fence_start = 0
    fence_top = {}
    fence_body = []
    rows = {}
    blocks = {}
    for n, line in enumerate(lines, 1):
        if line.startswith("```"):
            if not in_fence:
                in_fence, fence_start, fence_top, fence_body = True, n, {}, []
            else:
                in_fence = False
                body = "\n".join(fence_body)
                if body.strip():
                    if body in blocks:
                        found.append((path, "the code block starting `%s`" % fence_body[0].strip(), [blocks[body], fence_start]))
                    else:
                        blocks[body] = fence_start
            continue
        if in_fence:
            fence_body.append(line)
            if line and not line[0].isspace() and len(line.strip()) >= 8:
                if line in fence_top:
                    found.append((path, line, [fence_top[line], n]))
                else:
                    fence_top[line] = n
            continue
        if line.startswith("#"):
            rows, blocks = {}, {}
            continue
        is_row = line.startswith("|") and not set(line) <= set("|-: ")
        if is_row or re.match(r"^([-*] |\d+\. )", line):
            if line in rows:
                found.append((path, line, [rows[line], n]))
            else:
                rows[line] = n
    return found


def card_problems(root):
    out = []
    for name in ("docs/quick-reference.md", "docs/language-card.md", "docs/language-card-core.md"):
        out += card_duplicates(os.path.join(root, name))
    return out


def check(root):
    problems = card_problems(root)
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
        if len(card_problems(fixture)) != 3:
            print("check-doc-dupes: the fixture cards hold 3 exact repeats and the check did not find them", file=sys.stderr)
            return 1
        print("check-doc-dupes: self-test ok (the fixture fails as it should)")
        return 0
    root = "."
    if argv[1:2] == ["--root"] and len(argv) > 2:
        root = argv[2]
    problems = check(root)
    for path, key, lines in problems:
        print(f"{path}: duplicate '{key}' on lines {' and '.join(map(str, lines))}")
    if problems:
        print(f"check-doc-dupes: {len(problems)} duplicate(s)")
        return 1
    print("check-doc-dupes: no duplicate rows")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
