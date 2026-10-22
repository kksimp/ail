#!/bin/bash
cd "$(git rev-parse --show-toplevel)"
export RUSTUP_HOME=${RUSTUP_HOME:-$PWD/.toolchain/rustup} CARGO_HOME=${CARGO_HOME:-$PWD/.toolchain/cargo}
A=bootstrap/target/release/ail
for pair in tools/edit:edit bootstrap/src/std:std tools/garlic:garlic tools/garlic/js:garlic-js tools/pdf:pdf compiler:compiler; do
  d=${pair%%:*}; n=${pair##*:}
  $A api --card $d > docs/design/cards/$n.md
  if [ -d docs/design/cards/$n ]; then
    $A api --card --index $d > docs/design/cards/$n/index.md
    for f in docs/design/cards/$n/*.md; do m=$(basename $f .md); [ $m = index ] && continue; mm=$m; [ $m = root ] && mm="(root)"
      $A api --card --budget 2000 --module "$mm" $d > $f
    done
  fi
done
$A run examples/tool_explain_chapters.ail > examples/tool_explain_chapters.out 2>/dev/null
for x in tool_explain_rules tool_card_budgets; do [ -f examples/$x.ail ] && $A run examples/$x.ail > examples/$x.out 2>/dev/null; done
git status --short | head -30
