#!/bin/bash
# The cheap checks on a batch (item 253), printing only what failed; the last line is
# `CHECKS GREEN` or `CHECKS RED (N)`. Regenerates the cards once and commits them (item 251 d).
#   tools/gate/batch-checks.sh        from the repository root, on the batch branch
set -u
cd "$(git rev-parse --show-toplevel)"
# ADR 1130: `ail check` here must not apply its certain fixes to wrong-on-purpose fixtures.
export AIL_FIX=0
out="${TMPDIR:-/tmp}/ail-batch"; mkdir -p "$out"; fails=0
run() { # run NAME CMD...
  name=$1; shift
  "$@" >"$out/$name.log" 2>&1 && return 0
  # SIGBUS (135): the native CLI was rebuilt under a running copy; one rerun settles it.
  [ $? = 135 ] && "$@" >"$out/$name.log" 2>&1 && return 0
  fails=$((fails+1)); echo "RED $name"; grep -vE '^\s*$|: 0$' "$out/$name.log" | tail -12 | cut -c1-200
}
# The native CLI from the pinned seed (D4); every check below is native, no cargo (CQ, #878).
run build ./ail --version
[ $fails = 0 ] || { echo "CHECKS RED ($fails)"; exit 1; }
for d in compiler; do run check-${d##*/} ./ail check $d; done
# The file build nativex and self-host stage 1 do: it sees only main.ail and what it `use`s (item 267).
run build-cli env AIL_ROOT="$PWD" AIL_COMPILER="$PWD/compiler" ./ail build compiler/main.ail -o "$out/ail-cli"
run fmt ./ail fmt --check compiler examples tools/gate
# Every native run program has a frozen golden, or a pending.txt line (item 282; goldens.ail reds it on the Mac otherwise).
missing_goldens() {
  local g=tests/native/golden bad=0
  for f in tests/native/run/*.ail; do
    n=$(basename "$f" .ail)
    [ -f "$g/$n.out" ] || grep -qE "^$n([[:space:]]|$)" "$g/pending.txt" 2>/dev/null || { echo "no golden: $n"; bad=1; }
  done
  return $bad
}
run goldens missing_goldens
# A union that missed a hunk leaves markers that still build when they sit in docs (BN, 2026-10-24).
markers() { ! git grep -n "^<<<<<<< \|^>>>>>>> " -- . ":!tests"; }
run markers markers
run rules tools/rules-audit.sh
for p in links dupes snippets; do run doc-$p python3 tools/check-doc-$p.py; done
run cards tools/gate/regen-cards.sh
if [ -n "$(git status --porcelain docs examples)" ]; then
  git add docs examples && git commit -q -m "Regenerate the cards for the batch (item 251 d)"
fi
[ $fails = 0 ] && echo "CHECKS GREEN" || echo "CHECKS RED ($fails)"
[ $fails = 0 ]
