#!/bin/bash
cd "$(git rev-parse --show-toplevel)"
export RUSTUP_HOME=${RUSTUP_HOME:-$PWD/.toolchain/rustup} CARGO_HOME=${CARGO_HOME:-$PWD/.toolchain/cargo}
A=bootstrap/target/release/ail
for pair in tools/edit:edit bootstrap/src/std:std tools/garlic:garlic tools/garlic/js:garlic-js tools/pdf:pdf compiler:compiler; do
  d=${pair%%:*}; n=${pair##*:}
  # Write through a temporary file so a failing generator never commits an empty card.
  $A api --card $d > /tmp/card.$$ && mv /tmp/card.$$ docs/design/cards/$n.md || { echo "card failed: $d" >&2; exit 1; }
  if [ -d docs/design/cards/$n ]; then
    $A api --card --index $d > /tmp/card.$$ && mv /tmp/card.$$ docs/design/cards/$n/index.md || { echo "card index failed: $d" >&2; exit 1; }
    # One card per module in the index, so a new module gets its file (api.card.5).
    for m in $(tail -n +2 docs/design/cards/$n/index.md | awk '{print $1}'); do
      mm=$m; f=docs/design/cards/$n/$m.md; [ $m = "(root)" ] && { mm="(root)"; f=docs/design/cards/$n/root.md; }
      $A api --card --budget 2000 --module "$mm" $d > /tmp/card.$$ && mv /tmp/card.$$ $f || { echo "card failed: $d $m" >&2; exit 1; }
    done
  fi
done
$A run examples/tool_explain_chapters.ail > examples/tool_explain_chapters.out 2>/dev/null
for x in tool_explain_rules tool_card_budgets; do [ -f examples/$x.ail ] && $A run examples/$x.ail > examples/$x.out 2>/dev/null; done
git status --short | head -30
