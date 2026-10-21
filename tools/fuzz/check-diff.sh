#!/usr/bin/env bash
# Differential fuzz of the self-hosted checker against the bootstrap's (ADR 0753).
# usage: tools/fuzz/check-diff.sh [N] [SEED] [SAVE_DIR]
# Generates N programs (tools/fuzz/gen), runs `ail check --json` and the AIL
# checker (`ail run compiler check`) on each, and compares rule, line, column
# and message. Each disagreement is minimised by deleting lines while it
# still disagrees, then saved under SAVE_DIR (default tools/fuzz/found/check).
set -u
cd "$(dirname "$0")/../.."
N=${1:-1000}; SEED=${2:-1}; SAVE=${3:-tools/fuzz/found/check}
W=$(mktemp -d); trap 'rm -rf "$W"' EXIT
CHUNK=250

# "rule line col message @file" of each JSON diagnostic line on stdin.
norm() {
  sed 's/","\(help\|end_line\|fixes\)".*/"}/; s/.*"rule":"\([^"]*\)","file":"\([^"]*\/\)\?\([^"\/]*\)","line":\([0-9]*\),"col":\([0-9]*\),"message":"\(.*\)"}$/\1 \4 \5 \6 @\3/'
}

# Oracle diagnostics of a program as "rule line col message" lines.
oracle() {
  ./ail check --json "$1" 2>&1 >/dev/null | grep '^{' | grep -v '"rule":"eff.cap.[45]"' | norm | sort
}

# Splits the AIL checker's output on stdin into $W/ail/NAME.{diag,rules,skip}.
split_ail() {
  mkdir -p "$W/ail"
  awk -v dir="$W/ail" '
    /^== / { n=$2; sub(/.*\//,"",n); sub(/\.json$/,"",n); cur=dir "/" n; printf "" > cur ".diag"; printf "" > cur ".rules"; printf "" > cur ".skip"; next }
    /^rules / { sub(/^rules /,""); print > (cur ".rules"); next }
    /^skip / { print > (cur ".skip"); next }
    /^bodies / { next }
    /^\{/ { print > (cur ".diag") }
  '
}

# The differences between the two checkers on NAME (JSON in $W/NAME.json, program in $W/NAME.ail).
# Prints oracle-only lines with "<" and AIL-only lines with ">".
compare() {
  n=$1
  oracle "$(cat "$W/$n.path")" > "$W/$n.o"
  norm < "$W/ail/$n.diag" | sort > "$W/$n.a"
  # Only the rules the AIL side reports and the lines it checks, on both sides.
  counted() { awk -v rules="$(cat "$W/ail/$n.rules")" -v skips="$W/ail/$n.skip" '
    BEGIN { split(rules, r, " "); for (i in r) ok[r[i]]=1
            while ((getline l < skips) > 0) { k=split(l, w, " "); a[++m]=w[k-1]; b[m]=w[k]; f=w[k-2]; sub(/.*\//,"",f); sf[m]=f } }
    { if (!($1 in ok)) next; for (i=1;i<=m;i++) if ($2>=a[i] && $2<=b[i] && "@" sf[i]==$NF) next; print }' "$1"; }
  counted "$W/$n.o" > "$W/$n.of"; counted "$W/$n.a" > "$W/$n.af"; mv "$W/$n.af" "$W/$n.a"
  # The AIL side has no `help` text, and the oracle's `help` is cut above.
  diff "$W/$n.of" "$W/$n.a" | grep '^[<>]' || true
}

# Runs the AIL checker on one JSON (twice, so the section has its header).
ail_one() {
  ./ail run compiler check "$1" "$1" 2>/dev/null | split_ail
}

# $1: replacement text for the case's main program (a file, or main.ail of a package).
# True if the checkers still differ in the way $TARGET names.
disagrees() {
  if [ -d "$CASE" ]; then
    rm -rf "$W/mindir"; cp -r "$CASE" "$W/mindir"; cp "$1" "$W/mindir/main.ail"
    echo "$W/mindir" > "$W/min.path"
  else
    cp "$1" "$W/min.ail"; echo "$W/min.ail" > "$W/min.path"
  fi
  ./ail api --ast --program "$(cat "$W/min.path")" > "$W/min.json" 2>/dev/null || return 1
  [ "$(wc -l < "$W/min.json")" = 1 ] || return 1
  ail_one "$W/min.json"
  compare min | cut -d" " -f2,5- | grep -qxF -- "$TARGET"
}

# $1: the case path. Writes the minimal main program to $W/best.ail.
minimise() {
  CASE=$1
  if [ -d "$CASE" ]; then cp "$CASE/main.ail" "$W/best.ail"; else cp "$CASE" "$W/best.ail"; fi
  for size in 8 4 2 1; do
    i=$(wc -l < "$W/best.ail")
    while [ "$i" -ge 1 ]; do
      lo=$((i - size + 1)); [ "$lo" -ge 1 ] || lo=1
      sed "${lo},${i}d" "$W/best.ail" > "$W/try.ail"
      if disagrees "$W/try.ail"; then cp "$W/try.ail" "$W/best.ail"; fi
      i=$((lo - 1))
    done
  done
}

mkdir -p "$W/gen" "$SAVE"
./ail run tools/fuzz/gen/gen.ail "$N" "$SEED" "$W/gen" >/dev/null
total=0; checked=0; bad=0; saved=0
mkdir -p "$W/ail"
for ((c=0; c<N; c+=CHUNK)); do
  list=()
  for ((k=c; k<c+CHUNK && k<N; k++)); do
    total=$((total+1))
    f="$W/gen/p$k.ail"; [ -d "$W/gen/p$k" ] && f="$W/gen/p$k"
    ./ail api --ast --program "$f" > "$W/p$k.json" 2>/dev/null && [ "$(wc -l < "$W/p$k.json")" = 1 ] || continue
    echo "$f" > "$W/p$k.path"; list+=("$W/p$k.json")
  done
  [ ${#list[@]} -ge 1 ] || continue
  [ ${#list[@]} -ge 2 ] || list+=("${list[0]}")
  ./ail run compiler check "${list[@]}" 2>/dev/null | split_ail
  for j in "${list[@]}"; do
    n=$(basename "$j" .json); checked=$((checked+1))
    d=$(compare "$n")
    if [ -n "$d" ]; then
      bad=$((bad+1))
      TARGET=$(echo "$d" | head -1 | cut -d" " -f2,5-)
      case=$(cat "$W/$n.path"); minimise "$case"
      sig=$(cksum < "$W/best.ail" | cut -d' ' -f1)
      if [ -d "$case" ]; then
        dest="$SAVE/$sig"
        if [ ! -e "$dest" ]; then cp -r "$case" "$dest"; cp "$W/best.ail" "$dest/main.ail"; saved=$((saved+1)); fi
      else
        dest="$SAVE/$sig.ail"
        if [ ! -e "$dest" ]; then cp "$W/best.ail" "$dest"; saved=$((saved+1)); fi
      fi
      echo "disagree: $n ($(wc -l < "$W/best.ail") lines) -> $dest"; echo "$d" | head -4
    fi
  done
  echo "-- $checked checked of $total, $bad disagreements"
done
echo "programs $total, checked $checked, disagreements $bad, new repros $saved"
