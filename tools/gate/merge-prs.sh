#!/bin/bash
# usage: BASE=REF tools/gate/merge-prs.sh "N:title" ...  merges refs/pull/N/head in order onto a trial branch; docs conflicts are unioned, code conflicts stop the run
set -u
S=$(cd "$(dirname "$0")" && pwd)
export RUSTUP_HOME=${RUSTUP_HOME:-$PWD/.toolchain/rustup} CARGO_HOME=${CARGO_HOME:-$PWD/.toolchain/cargo}
cd "$(git rev-parse --show-toplevel)"
git fetch -q origin claude/project-thread-ipafhh
git checkout -q -B trial ${BASE:-origin/claude/project-thread-ipafhh}
for spec in "$@"; do
  n=${spec%%:*}; title=${spec#*:}
  git fetch -q origin "+refs/pull/$n/head:refs/remotes/origin/pr$n"
  if ! git merge --no-ff origin/pr$n -m "Merge $title (#$n)" >/dev/null 2>&1; then
    c=$(git diff --name-only --diff-filter=U); echo "PR $n conflicts: $c"
    for f in $c; do case "$f" in docs/*|*.md|examples/*.out|tests/diag_text_known.txt) python3 $S/union-docs.py "$f";; *) echo "CODE CONFLICT in $f: stopping"; exit 3;; esac; done
    git add -A; git commit -q -m "Merge $title (#$n)"
  fi
done
git grep -n "^<<<<<<< \|^>>>>>>> " -- . ":!tests" | head -3
git log --oneline -1
