#!/usr/bin/env bash
# Delete remote branches that are already fully merged into main (ADR 0028).
#
#   tools/prune-merged.sh
#
# Only branches whose latest commit is contained in origin/main are deleted, so
# no work can be lost. main itself is never touched.
set -euo pipefail
root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$root"

git fetch --quiet --prune origin
merged=()
# Full ref names: the short name of refs/remotes/origin/HEAD is just "origin".
while read -r ref; do
    name="${ref#refs/remotes/origin/}"
    [ "$name" = main ] && continue
    [ "$name" = HEAD ] && continue
    if git merge-base --is-ancestor "$ref" origin/main; then
        merged+=("$name")
    else
        echo "keeping $name (has commits not in main)"
    fi
done < <(git for-each-ref --format='%(refname)' refs/remotes/origin)

if [ ${#merged[@]} -eq 0 ]; then
    echo "nothing to delete: no merged branches besides main"
    exit 0
fi
for name in "${merged[@]}"; do
    git push --quiet origin --delete "$name"
    git branch -D "$name" >/dev/null 2>&1 || true
    echo "deleted $name (already in main)"
done
git worktree prune
