#!/usr/bin/env python3
"""D6 (ADR 0855): rewrite citations of the deleted `bootstrap/` tree.

Dry run by default: prints a unified diff and a summary, edits nothing.
`--apply` writes the files. Run from the repository root the day after D5.

Rules (first match wins):
  1. `bootstrap/target/...` is a build output, not a citation: reported under
     "manual" (D3/D4 own the scripts that still call it), never rewritten.
  2. `bootstrap/tests/X` where `tests/X` exists becomes `tests/X`.
  3. Anything else under `bootstrap/` becomes `REF:bootstrap/...`, which
     `git show REF:bootstrap/src/check.rs` resolves (REF is the commit before the D5
     deletion, `77592931` (ADR 0904); `--tag` overrides).

Code files (.ail, .sh, .py, .rs, .toml and the like) are rewritten only on
whole-line `#` comments; a citation in code or a trailing comment could be a
live path, so it is listed under "manual". Markdown and text files are
rewritten everywhere.

Skipped: docs/decisions/ (ADRs are history), the `.git` directory, binary
files, this script. Rewriting is idempotent: a path already prefixed `REF:`
is left alone.
"""
import argparse, difflib, os, re, subprocess, sys

PATH = re.compile(r"(?<![\w:./-])bootstrap/[A-Za-z0-9_./-]*[A-Za-z0-9_/]")
SKIP_DIRS = ("docs/decisions/", "bootstrap/", ".git/")
SELF = "tools/d6/rewrite-citations.py"
PROSE = (".md", ".txt")


def files():
    out = subprocess.run(["git", "ls-files", "-z"], capture_output=True, check=True).stdout
    for p in out.decode().split("\0"):
        if p and p != SELF and not p.startswith(SKIP_DIRS):
            yield p


def rewrite(m, tag, manual, where):
    s = m.group(0)
    if s.startswith("bootstrap/target"):
        manual.append((where, s))
        return s
    if s.startswith("bootstrap/tests/"):
        rest = s[len("bootstrap/"):]
        if os.path.exists(rest.rstrip("/")):
            return rest
    return f"{tag}:{s}"


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--apply", action="store_true", help="write the files (default: dry run)")
    ap.add_argument("--tag", default="77592931", help="git ref of the last tree with bootstrap/ (default: the parent of the D5 deletion)")
    a = ap.parse_args()
    manual, changed, edits = [], 0, 0
    for p in files():
        try:
            old = open(p, encoding="utf-8").read()
        except (UnicodeDecodeError, IsADirectoryError, FileNotFoundError):
            continue
        lines = old.split("\n")
        new_lines = []
        prose = p.endswith(PROSE)
        for i, line in enumerate(lines, 1):
            if prose or line.lstrip().startswith("#"):
                line = PATH.sub(lambda m: rewrite(m, a.tag, manual, f"{p}:{i}"), line)
            else:
                for m in PATH.finditer(line):
                    manual.append((f"{p}:{i}", m.group(0)))
            new_lines.append(line)
        new = "\n".join(new_lines)
        if new == old:
            continue
        changed += 1
        edits += sum(1 for x, y in zip(lines, new_lines) if x != y)
        sys.stdout.writelines(difflib.unified_diff(
            old.splitlines(True), new.splitlines(True), f"a/{p}", f"b/{p}", n=0))
        if a.apply:
            open(p, "w", encoding="utf-8").write(new)
    print(f"\n{'applied' if a.apply else 'dry run'}: {edits} lines in {changed} files", file=sys.stderr)
    if manual:
        print(f"manual (build outputs and code, D3/D4 or by hand): {len(manual)}", file=sys.stderr)
        by = {}
        for w, s in manual:
            by.setdefault(w.rsplit(":", 1)[0], []).append(w.rsplit(":", 1)[1])
        for f, ls in sorted(by.items()):
            print(f"  {f}: {len(ls)} (first line {ls[0]})", file=sys.stderr)


main()
