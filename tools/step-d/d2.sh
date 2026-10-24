#!/usr/bin/env bash
# Step D2 (plan.md, item 277): move the data out of bootstrap/ so the Rust can be deleted later.
#
#   tools/step-d/d2.sh plan     what would move and how many files change (changes nothing)
#   tools/step-d/d2.sh dry      apply in a throwaway worktree, check there, print the result
#   tools/step-d/d2.sh apply    apply in THIS checkout (the integrator's freeze window); then commit
#
# Moves (git mv, history kept):
#   bootstrap/src/std                       -> std
#   bootstrap/src/prelude.ail               -> tools/native/prelude.ail
#   bootstrap/src/fixable.txt               -> compiler/explain/fixable.txt
#   bootstrap/src/net/{crypto,tls,x509}/testdata -> tests/data/net/{crypto,tls,x509}
#   bootstrap/tests/** except *.rs          -> tests/**
# Then every reference outside docs/decisions/ is rewritten with the same table. The Rust sources
# keep building until D5 through symlinks left at the old places (bootstrap/src/std -> ../../std, ...);
# D5 deletes bootstrap/ with them. Paths of the Rust binary (bootstrap/target/release/ail) are D3's.
set -euo pipefail
mode="${1:-plan}"
here="$(cd "$(dirname "$0")/../.." && pwd)"

rewrite_table() {
    # perl s///: order matters (most specific first).
    cat <<'EOF'
s{bootstrap/src/net/(crypto|tls|x509)/testdata}{tests/data/net/$1}g
s{bootstrap/src/prelude\.ail}{tools/native/prelude.ail}g
s{bootstrap/src/fixable\.txt}{compiler/explain/fixable.txt}g
s{bootstrap/src/std\b}{std}g
s{bootstrap/tests(?=/(?![A-Za-z_]+\.rs\b)|\b(?!/))}{tests}g
EOF
}

moves() {
    # source -> destination, one per line
    echo "bootstrap/src/std std"
    echo "bootstrap/src/prelude.ail tools/native/prelude.ail"
    echo "bootstrap/src/fixable.txt compiler/explain/fixable.txt"
    for d in crypto tls x509; do
        [ -d "bootstrap/src/net/$d/testdata" ] && echo "bootstrap/src/net/$d/testdata tests/data/net/$d"
    done
    # tests: every top-level entry except Rust sources
    for e in bootstrap/tests/*; do
        case "$e" in *.rs) ;; *) echo "$e tests/$(basename "$e")" ;; esac
    done
}

# Files whose text is rewritten: tracked, outside bootstrap/src (Rust), outside ADR history,
# not binary, not generated results. bootstrap/tests files move first, so they are included at their new place.
rewrite_files() {
    git ls-files -z | grep -zv '^bootstrap/\(src/\|target/\|Cargo\|build\.rs\|rust\)\|^docs/decisions/\|^tools/native-coverage/results\.json$\|^\.git' \
        | xargs -0 grep -lIE 'bootstrap/(src/(std|prelude|fixable|net/(crypto|tls|x509)/testdata)|tests)' 2>/dev/null || true
}

do_plan() {
    echo "== moves"
    moves | while read -r a b; do
        n=$(git ls-files "$a" | wc -l)
        printf '%6d files  %s -> %s\n' "$n" "$a" "$b"
    done
    echo "== files whose text is rewritten: $(rewrite_files | wc -l)"
    rewrite_files | sed 's|/[^/]*$||' | sort | uniq -c | sort -rn | head -25
    echo "== Rust sources that include moved files (symlinks keep them building until D5)"
    grep -rn 'include_str!\|include_bytes!' bootstrap/src --include=*.rs | grep -c 'std\|prelude\|fixable\|testdata\|tests/' || true
}

do_apply() {
    [ -z "$(git status --porcelain --untracked-files=no)" ] || { echo "d2: commit or stash your changes first" >&2; exit 1; }
    mkdir -p tests/data/net
    moves | while read -r a b; do
        [ -e "$a" ] || continue
        mkdir -p "$(dirname "$b")"
        git mv "$a" "$b"
    done
    # the old places stay readable for the Rust build (D5 removes them with bootstrap/)
    ln -s ../../std bootstrap/src/std
    ln -s ../../tools/native/prelude.ail bootstrap/src/prelude.ail
    ln -s ../../compiler/explain/fixable.txt bootstrap/src/fixable.txt
    for d in crypto tls x509; do
        [ -d "tests/data/net/$d" ] && ln -s "../../../../tests/data/net/$d" "bootstrap/src/net/$d/testdata"
    done
    # tests: link the moved entries back into bootstrap/tests/ for the Rust tests still there
    for e in tests/*; do
        n=$(basename "$e")
        [ -e "bootstrap/tests/$n" ] || ln -s "../../tests/$n" "bootstrap/tests/$n"
    done
    rewrite_files | while read -r f; do
        [ -L "$f" ] && continue
        rewrite_table | while read -r expr; do perl -0pi -e "$expr" "$f"; done
    done
    git add -A
    echo "d2: moved and rewritten; review with 'git status --short | wc -l', then commit."
}

do_checks() {
    echo "== leftover references (functional ones must be empty)"
    git ls-files | grep -v '^bootstrap/\|^docs/decisions/\|results\.json$' \
        | xargs grep -nE 'bootstrap/(src/(std|prelude\.ail|fixable)|tests/[A-Za-z_./-]*[^s]\b)' 2>/dev/null \
        | grep -v '\.rs\b' | head -20 || true
    echo "== old places are links"
    ls -ld bootstrap/src/std bootstrap/src/prelude.ail bootstrap/src/fixable.txt 2>&1 | sed 's/^/  /'
    echo "== native CLI builds with the new layout"
    # the worktree has no cargo build of its own: use the checkout's Rust binary when there is one
    rust="$here/bootstrap/target/release/ail"
    [ -x "$rust" ] || rust="./ail"
    AIL_ROOT="$PWD" AIL_BINARY="$rust" "$rust" build compiler/main.ail -o "${TMPDIR:-/tmp}/d2-ail-cli" 2>&1 | tail -5
    echo "== a few native programs"
    for p in examples/hello.ail tests/native/run/arith.ail tests/native/run/std_app_run.ail; do
        if [ -f "$p" ]; then
            if AIL_ROOT="$PWD" AIL_BINARY="$rust" "$rust" build "$p" -o "${TMPDIR:-/tmp}/d2-prog" >/dev/null 2>&1 && "${TMPDIR:-/tmp}/d2-prog" >/dev/null 2>&1; then echo "  ok   $p"; else echo "  FAIL $p"; fi
        fi
    done
    echo "== the new native CLI with the old places removed (std must come from std/)"
    cli="${TMPDIR:-/tmp}/d2-ail-cli"
    rm -f bootstrap/src/std bootstrap/src/prelude.ail bootstrap/src/fixable.txt
    if [ -x "$cli" ] && AIL_ROOT="$PWD" AIL_COMPILER="$PWD/compiler" "$cli" build tests/native/run/std_app_run.ail -o "${TMPDIR:-/tmp}/d2-prog2" >/dev/null 2>&1 && "${TMPDIR:-/tmp}/d2-prog2" >/dev/null 2>&1; then echo "  ok   native build without bootstrap/src/std"; else echo "  FAIL native build without bootstrap/src/std"; fi
    echo "== changed files: $(git status --short | wc -l)"
}

case "$mode" in
    plan) cd "$here"; do_plan ;;
    apply) cd "$here"; do_apply ;;
    dry)
        cd "$here"
        wt="$(mktemp -d)/d2"
        git worktree add --quiet --detach "$wt" HEAD
        trap 'git -C "$here" worktree remove --force "$wt" >/dev/null 2>&1 || true' EXIT
        cd "$wt"
        do_apply
        do_checks
        echo "(dry run in $wt; removed on exit)"
        ;;
    *) echo "usage: tools/step-d/d2.sh plan|dry|apply" >&2; exit 2 ;;
esac
