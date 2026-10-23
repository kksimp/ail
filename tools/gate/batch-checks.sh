#!/bin/bash
# The cheap checks on a batch (item 253), printing only what failed; the last line is
# `CHECKS GREEN` or `CHECKS RED (N)`. Regenerates the cards once and commits them (item 251 d).
#   tools/gate/batch-checks.sh        from the repository root, on the batch branch
set -u
cd "$(git rev-parse --show-toplevel)"
out="${TMPDIR:-/tmp}/ail-batch"; mkdir -p "$out"; fails=0
run() { # run NAME CMD...
  name=$1; shift
  "$@" >"$out/$name.log" 2>&1 && return 0
  fails=$((fails+1)); echo "RED $name"; grep -vE '^\s*$' "$out/$name.log" | tail -6 | cut -c1-200
}
# ./ail rebuilds the bootstrap binary (it embeds the compiler) when its sources changed.
run build ./ail --version
[ $fails = 0 ] || { echo "CHECKS RED ($fails)"; exit 1; }
for d in compiler tools/garlic; do run check-${d##*/} ./ail check $d; done
# The file build nativex and self-host stage 1 do: it sees only main.ail and what it `use`s (item 267).
run build-cli env AIL_ROOT="$PWD" AIL_COMPILER="$PWD/compiler" AIL_RUST=1 ./ail build compiler/main.ail -o "$out/ail-cli"
run fmt ./ail fmt --check compiler examples tools/garlic
run rules tools/rules-audit.sh
for p in links dupes snippets; do run doc-$p python3 tools/check-doc-$p.py; done
run cards tools/gate/regen-cards.sh
if [ -n "$(git status --porcelain docs examples)" ]; then
  git add docs examples && git commit -q -m "Regenerate the cards for the batch (item 251 d)"
fi
[ $fails = 0 ] && echo "CHECKS GREEN" || echo "CHECKS RED ($fails)"
[ $fails = 0 ]
