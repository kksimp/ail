#!/usr/bin/env python3
"""Check that relative links in the Markdown docs resolve to existing files.

Run with `python3 -I tools/check-doc-links.py` from the repository root.
Exits 1 when any link is broken. Anchors are not checked.
"""
import os
import re
import sys

LINK = re.compile(r"\]\(([^)\s]+)\)")
CODE = re.compile(r"`[^`]*`")
ROOTS = ["README.md", "AGENTS.md", "docs"]
PLACEHOLDER = "NNNN-file.md"


def markdown_files():
    for root in ROOTS:
        if os.path.isfile(root):
            yield root
        for dirpath, _, names in os.walk(root):
            for name in names:
                if name.endswith(".md"):
                    yield os.path.join(dirpath, name)


broken = []
for path in sorted(markdown_files()):
    with open(path, encoding="utf-8") as f:
        for lineno, line in enumerate(f, 1):
            line = CODE.sub("", line)
            for target in LINK.findall(line):
                if re.match(r"[a-z]+:|#", target) or "." not in target:
                    continue
                file_part = target.split("#", 1)[0]
                if not file_part or file_part == PLACEHOLDER:
                    continue
                resolved = os.path.normpath(os.path.join(os.path.dirname(path), file_part))
                if not os.path.exists(resolved):
                    broken.append(f"{path}:{lineno}: {target}")

for line in broken:
    print(line)
print(f"broken links: {len(broken)}")
sys.exit(1 if broken else 0)
