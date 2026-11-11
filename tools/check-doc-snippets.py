#!/usr/bin/env python3
"""Check the AIL code blocks in the docs with `ail check`.

Every fenced block whose info string is exactly `ail` or `ail fragment` is
checked, in every Markdown file under `docs/` and in `README.md`, except
`docs/spec/` (checked by the `spec` suite of `./ail selftest`) and
the quick reference and language card (their token budgets are checked
elsewhere, and they are listed here but not edited).

- `ail` is checked as a whole program.
- `ail fragment` is wrapped the way the spec suite wraps one: column-0
  declarations stay top level and every other line goes into `fn main()`.
- Any other tag (`ail excerpt`, `ail skip=context`, `ail norun`, `ail
  rules=...`, `text`, `sh`) is not checked. An excerpt is an incomplete
  snippet whose point is one line.

Run `python3 -I tools/check-doc-snippets.py` from the repository root. It
exits 1 when a checked block fails. `--root DIR` checks another tree with
the same `./ail` binary. Build `./ail` first (`./ail check` builds it).
"""
import argparse
import os
import re
import subprocess
import sys
import tempfile

DECLS = ("struct ", "enum ", "error ", "unique ", "fn ", "unsafe fn ", "extern ",
         "trait ", "impl ", "const ", "use ", "type ", "@", "pub ", "#")
SKIP_FILES = {"docs/quick-reference.md", "docs/language-card.md", "docs/language-card-core.md"}
FENCE = re.compile(r"^```ail(?: (fragment))?\s*$")


def wrap(code):
    """Split a fragment into declarations and a `main` body (as spec.rs does)."""
    decls, body, in_decl = [], [], False
    for line in code.split("\n"):
        if line and not line.startswith(" "):
            in_decl = line.startswith(DECLS)
        if in_decl:
            decls.append(line)
        elif line == "":
            body.append("")
        else:
            body.append("    " + line)
    text = "\n".join(body)
    if not text.strip():
        text = '    print("")'
    return "\n".join(decls) + "\n\nfn main()\n" + text + "\n"


def blocks(path):
    """Yield (line, fragment, code) for each checked block in one Markdown file."""
    lines = open(path, encoding="utf-8").read().split("\n")
    i = 0
    while i < len(lines):
        m = FENCE.match(lines[i])
        if not m:
            i += 1
            continue
        start, code = i + 1, []
        i += 1
        while i < len(lines) and not lines[i].startswith("```"):
            code.append(lines[i])
            i += 1
        yield start, m.group(1) == "fragment", "\n".join(code) + "\n"
        i += 1


def files(root):
    out = []
    for base, _, names in os.walk(os.path.join(root, "docs")):
        for name in names:
            if name.endswith(".md"):
                out.append(os.path.join(base, name))
    out.append(os.path.join(root, "README.md"))
    for path in sorted(out):
        rel = os.path.relpath(path, root).replace(os.sep, "/")
        if rel.startswith("docs/spec/") or rel in SKIP_FILES or not os.path.isfile(path):
            continue
        yield rel, path


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[1])
    ap.add_argument("--root", default=os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    args = ap.parse_args()
    root = args.root
    ail = os.path.join(root, "ail")
    checked = failed = 0
    with tempfile.TemporaryDirectory(prefix="doc-snippets-") as tmp:
        for rel, path in files(root):
            for line, fragment, code in blocks(path):
                checked += 1
                src = wrap(code) if fragment else code
                snip = os.path.join(tmp, "snippet.ail")
                with open(snip, "w", encoding="utf-8") as f:
                    f.write(src)
                r = subprocess.run([ail, "check", snip], cwd=root, capture_output=True, text=True)
                if r.returncode != 0:
                    failed += 1
                    first = (r.stdout + r.stderr).strip().split("\n")[0].replace(snip, "snippet")
                    print(f"{rel}:{line}: {'ail fragment' if fragment else 'ail'}: {first}")
    print(f"doc snippets: {checked} checked, {failed} failed")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
