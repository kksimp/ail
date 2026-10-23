#!/bin/bash
# The batch builder (item 253): merges every open PR that doesn't clash onto BASE, in one branch.
#   tools/gate/batch.sh [--base REF] [--branch NAME] [--checks] [--push] [N ...]
# With no N it takes the open PRs from `gh pr list` (oldest first). Per PR it prints one line:
#   IN N      merged (docs conflicts unioned, as merge-prs.sh does)
#   HAVE N    its changes are already on BASE: close it
#   CLASH N   a code conflict with BASE or an earlier PR in the batch; it waits for the next batch
# then WATCH lines (code files two merged PRs both touch, and duplicate top-level fns in them:
# a clean merge can still carry two copies of an arm), and `BATCH <sha> PRs ...` last.
# --checks runs tools/gate/batch-checks.sh on the result; --push pushes the branch to origin.
set -u
S=$(cd "$(dirname "$0")" && pwd)
cd "$(git rev-parse --show-toplevel)"
base=origin/main branch=claude/trial-prep checks=0 push=0 prs=()
while [ $# -gt 0 ]; do
  case $1 in
    --base) base=$2; shift;; --branch) branch=$2; shift;;
    --checks) checks=1;; --push) push=1;;
    *) prs+=("${1%%:*}");;
  esac; shift
done
[ ${#prs[@]} -eq 0 ] && prs=($(gh pr list --state open --limit 200 --json number -q '.[].number' | sort -n))
[ ${#prs[@]} -eq 0 ] && { echo "no PRs (pass numbers, or log gh in)"; exit 2; }
git fetch -q origin "${base#origin/}" $(for n in "${prs[@]}"; do echo "+refs/pull/$n/head:refs/remotes/origin/pr$n"; done) || exit 2
git checkout -q -B "$branch" "$base"
docs() { case $1 in docs/*|*.md|examples/*.out|bootstrap/tests/diag_text_known.txt) return 0;; esac; return 1; }
in=() files=$(mktemp)
for n in "${prs[@]}"; do
  r=origin/pr$n
  # Already on base (merged by commit, or a stacked parent that landed): nothing to add.
  t=$(git merge-tree --write-tree "$base" $r 2>/dev/null | head -1)
  if [ -n "$t" ] && git diff --quiet "$base" "$t"; then echo "HAVE $n"; continue; fi
  if ! git merge -q --no-ff $r -m "Merge #$n" >/dev/null 2>&1; then
    c=$(git diff --name-only --diff-filter=U); code=""
    for f in $c; do docs "$f" || code="$code $f"; done
    if [ -n "$code" ]; then
      git merge --abort
      who=$(for f in $code; do awk -v f="$f" '$2==f {print "#"$1}' "$files"; done | sort -u | tr '\n' ' ')
      echo "CLASH $n:$code ${who:+(with ${who% })}${who:-(with $base)}"; continue
    fi
    python3 "$S/union-docs.py" $c; git add -A; git commit -q -m "Merge #$n"
  fi
  git diff --name-only "$(git merge-base "$base" $r)" $r | sed "s|^|$n |" >> "$files"
  in+=($n); echo "IN $n"
done
# Clean merges that touch the same code file: look there for duplicate arms and functions.
awk '{print $2}' "$files" | sort | uniq -d | while read -r f; do
  docs "$f" && continue
  echo "WATCH $f: $(awk -v f="$f" '$2==f {print "#"$1}' "$files" | sort -u | tr '\n' ' ')"
  case $f in *.ail) [ -f "$f" ] && grep -oE '^(pub )?fn [A-Za-z0-9_]+' "$f" | sort | uniq -d | sed "s|^|WATCH $f duplicate |";; esac
done
rm -f "$files"
git grep -n "^<<<<<<< \|^>>>>>>> " -- . ":!bootstrap/tests" | head -3
[ $checks = 1 ] && { "$S/batch-checks.sh" || exit 1; }
[ $push = 1 ] && git push -q -f origin "$branch"
echo "BATCH $(git rev-parse --short=8 HEAD) PRs ${in[*]}"
