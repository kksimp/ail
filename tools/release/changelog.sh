#!/usr/bin/env bash
# Print the changelog entry for a release: a hand-written highlights block, then the
# merged pull-request titles on main since the previous tag (ADR 1043). Nothing is
# written or tagged. The block sits between `<!-- highlights:begin -->` and
# `<!-- highlights:end -->` in the CHANGELOG.md section of the same version and is
# copied unchanged (the heading is `## vVERSION` or `## Unreleased (MAJOR.MINOR)`); only the list under "### Changes" is regenerated.
#   tools/release/changelog.sh VERSION [FROM_REF] [TO_REF]
# VERSION is like 0.1.0. FROM_REF defaults to the newest v* tag, or the first commit
# when there is none. TO_REF defaults to HEAD. Prepend the output to CHANGELOG.md.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$root"
ver="${1:?usage: tools/release/changelog.sh VERSION [FROM_REF] [TO_REF]}"
to="${3:-HEAD}"
from="${2:-$(git describe --tags --match 'v*' --abbrev=0 "$to" 2>/dev/null || true)}"
range="$to"; [ -n "$from" ] && range="$from..$to"
echo "## v$ver - $(TZ=UTC git log -1 --date=format-local:%Y-%m-%d --format=%cd "$to")"
echo
CHANGELOG_VER="$ver" python3 -c '
import os, re
s = open("CHANGELOG.md").read()
m = re.search(r"## (?:v" + re.escape(os.environ["CHANGELOG_VER"]) + r"|Unreleased \(" + re.escape(os.environ["CHANGELOG_VER"].rsplit(".", 1)[0]) + r"\))[^\n]*\n.*?(<!-- highlights:begin -->.*?<!-- highlights:end -->)", s, re.S)
print(m.group(1) if m else "<!-- highlights:begin -->\n<!-- highlights:end -->")
print()
print("### Changes")
print()
'
# First-parent history is main's PR landings. A GitHub merge commit carries the PR
# title as the first body line; a squash commit has "title (#N)" as its subject.
git log --first-parent --format='%H%x1f%s%x1f%b%x1e' "$range" | python3 -c '
import re, sys
for rec in sys.stdin.read().split("\x1e"):
    rec = rec.strip("\n")
    if not rec: continue
    _, subj, body = (rec.split("\x1f") + ["", ""])[:3]
    m = re.match(r"Merge pull request #(\d+) from \S+", subj)
    if m:
        title = next((l for l in body.splitlines() if l.strip()), subj)
        print("- %s (#%s)" % (title.strip(), m.group(1)))
    elif re.search(r"\(#\d+\)$", subj):
        print("- " + subj)
'
