#!/usr/bin/env bash
# Self-host bootstrap harness (ADR 0022, native self-compile push). Proves the
# native compiler can build itself, stage by stage, and says where it stops.
#
#   tools/self-host/run.sh [OUT_DIR]
#
# Stages (each runs only if the one before it worked):
#   0  the pinned seed (tools/release/SEED_TAG) is what ./ail starts from.
#   1  `./ail build compiler/main.ail` makes the native compiler (ADR 1208: `./ail` itself is already stage 2, so this is built by the tree).
#   2  stage 1 builds compiler/main.ail again.
#   3  stage 2 builds it a third time; stage 2 and 3 must be byte-identical
#      (the fixed point), and stage 1 and 2 too (same compiler, same output).
#   T  the native test programs (tests/native/run/*.ail) built with
#      the newest compiler, run, and diffed against `ail run` (stdout and exit
#      code).
# Prints one line per stage and the first blocker. Exit 0 only if every stage
# passes; 1 if a stage is blocked (after printing why); 2 on a setup error.
#
# Environment:
#   STAGE1=PATH   use this executable as the stage-1 compiler (it must take
#                 `build FILE RT [TARGET]` and print the executable as hex), to
#                 test the later stages before stage 1 builds. A wrapper that
#                 runs `ail run compiler build "$@"` stands in for it.
#   TARGET=T      x86_64-linux, aarch64-linux or aarch64-macos. Default: the host's
#                 (aarch64-macos on an Apple Silicon Mac, x86_64-linux on Linux
#                 x86-64). Only the host's target can be run; other targets stop
#                 after the build stages. The stage compilers are always given the
#                 target explicitly: their own default is x86_64-linux.
#   ONLY=NAMES    comma-separated test names (no .ail) for stage T.
#                 ONLY=none skips stage T (the fixed point alone, as seed.sh asks).
#   TESTS_ONLY=1  with STAGE1: skip stages 2 and 3 and run stage T with STAGE1
#                 as the compiler (this checks the harness itself).
#
# Writes only under OUT_DIR (default: a temp directory); leaves the stage
# executables and report.md there. Delete it when done: the executables are
# several MB each.
set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
out="${1:-${TMPDIR:-/tmp}/self-host-$$}"
mkdir -p "$out"
cd "$root" || exit 2
[ -x ./ail ] || { echo "run.sh: no ./ail launcher" >&2; exit 2; }

rt=compiler/rt/native
tests=tests/native/run
report="$out/report.md"
case "$(uname -s)-$(uname -m)" in
    Darwin-arm64) host_target=aarch64-macos ;;
    Linux-aarch64 | Linux-arm64) host_target=aarch64-linux ;;
    *) host_target=x86_64-linux ;;
esac
TARGET="${TARGET:-$host_target}"
: > "$report"

say() { echo "$*" | tee -a "$report"; }
blocked() { say ""; say "BLOCKED at $1: $2"; say "Report: $report"; exit 1; }

# hex text (stdin) to the executable PATH
unhex() { perl -ne 'chomp; print pack("H*", $_)' > "$1" && chmod +x "$1"; }

# build SRC with compiler CMD into OUT; the compiler prints hex. It runs the
# derive generators itself (ADR 0856), with no Rust-made derive file.
build_with() { # compiler src out
    local cc="$1" src="$2" dst="$3"
    # The paths `ail build` passes (`ail build`): absolute, since they
    # end up in the trap texts of the executable. Relative ones make stage 1 and
    # the later stages differ in size.
    "$cc" build "$src" "$root/compiler/rt/native" "$TARGET" 0 0 "$root/compiler/../std" > "$dst.hex" 2> "$dst.err" || return 1
    unhex "$dst" < "$dst.hex"
}

say "# Self-host bootstrap ($(date -u +%Y-%m-%dT%H:%M:%SZ), $(git rev-parse --short HEAD))"
say ""

# ---- stage 0: the seed ----
# ./ail builds the native compiler from the pinned seed (tools/release/SEED_TAG); no Rust is involved.
say "## Stage 0: the pinned seed (tools/release/SEED_TAG: $(cat tools/release/SEED_TAG 2>/dev/null || echo none))"
missing=0
say ""

# ---- stage 1: the seed-hosted build ----
say "## Stage 1: ./ail build compiler/main.ail"
if [ -n "${STAGE1:-}" ]; then
    stage1="$STAGE1"
    say "stage 1 supplied: $stage1"
else
    stage1="$out/stage1"
    if ./ail build compiler/main.ail -o "$stage1" --target "$TARGET" > "$out/stage1.log" 2>&1; then
        say "built $stage1 ($(wc -c < "$stage1") bytes)"
    else
        problems=$(grep -c 'error\[' "$out/stage1.log")
        say "FAILED with $problems problems (log: $out/stage1.log). Top constructs:"
        grep 'error\[' "$out/stage1.log" | sed -E 's/^[^ ]+ error\[[^]]+\]: //; s/`[^`]*`/`X`/g' |
            sort | uniq -c | sort -rn | head -8 | sed 's/^/  /' | tee -a "$report"
        if [ "$missing" -gt 0 ]; then
            say "Also: stage 0 lists $missing library calls the compiler can't make natively yet."
        fi
        blocked "stage 1" "the seed-hosted build of compiler/main.ail failed (see the log)"
    fi
fi
say ""

# ---- stage 2 and 3: the fixed point ----
if [ "$TARGET" != "$host_target" ]; then
    # The target's executables can't run here, but a host-target stage 1 can
    # cross-build compiler/main.ail for it, and that must equal stage 1 byte
    # for byte (the macOS stage 1 and 2 difference was the paths above).
    say "## Cross check: a $host_target stage 1 builds compiler/main.ail for $TARGET"
    if ./ail build compiler/main.ail -o "$out/host1" --target "$host_target" > "$out/host1.log" 2>&1 &&
        build_with "$out/host1" compiler/main.ail "$out/cross" && cmp -s "$out/cross" "$stage1"; then
        say "OK: the cross build is byte-identical to stage 1 ($(wc -c < "$stage1") bytes)."
    else
        blocked "cross check" "the $host_target stage 1's $TARGET build of compiler/main.ail differs from stage 1 (cmp $out/cross $stage1)"
    fi
    say "OK ($TARGET can't run on this $host_target host, so stages 2, 3 and T were skipped)."
    exit 0
fi
if [ -n "${TESTS_ONLY:-}" ] && [ -n "${STAGE1:-}" ]; then
    say "(stages 2 and 3 skipped: TESTS_ONLY)"
    cp "$stage1" "$out/stage2" 2>/dev/null || ln -sf "$stage1" "$out/stage2"
else
say "## Stage 2: stage 1 builds compiler/main.ail"
if build_with "$stage1" compiler/main.ail "$out/stage2"; then
    say "built $out/stage2 ($(wc -c < "$out/stage2") bytes)"
else
    say "failed: $(head -c 300 "$out/stage2.err")"
    blocked "stage 2" "the stage-1 compiler can't build compiler/main.ail (see $out/stage2.err)"
fi
if [ -z "${STAGE1:-}" ]; then
    if cmp -s "$out/stage1" "$out/stage2"; then
        say "stage 1 == stage 2 (byte-identical)"
    else
        # `ail build` passes absolute paths and the std directory, which end up
        # in the executable; stage 2 == stage 3 below is the real test.
        say "stage 1 != stage 2 (a note, not an error: stage 2 == stage 3 decides)"
    fi
fi
say ""
say "## Stage 3: stage 2 builds compiler/main.ail (the fixed point)"
if build_with "$out/stage2" compiler/main.ail "$out/stage3"; then
    if cmp -s "$out/stage2" "$out/stage3"; then
        say "stage 2 == stage 3 (byte-identical): fixed point reached"
    else
        say "stage 2 != stage 3"
        cmp "$out/stage2" "$out/stage3" 2>&1 | head -1 | tee -a "$report"
        blocked "stage 3" "no fixed point"
    fi
else
    blocked "stage 3" "stage 2 can't build compiler/main.ail (see $out/stage3.err)"
fi
fi
say ""

# ---- stage T: the native tests through the newest compiler ----
say "## Stage T: native test programs built by the stage-2 compiler"
pass=0; fail=0; failed=()
# The programs tests/native.rs skips on macOS (MACOS_KNOWN there: they differ
# from the interpreter by design), read from that list so the two agree.
macos_known=""
if [[ "$TARGET" == *macos* ]]; then
    macos_known=$(grep -v '^#' tests/native/golden/macos_known.txt | tr '\n' ',')
fi
for f in "$tests"/*.ail; do
    name=$(basename "$f" .ail)
    if [ -n "${ONLY:-}" ] && [[ ",$ONLY," != *",$name,"* ]]; then continue; fi
    if [[ ",$macos_known" == *",$name,"* ]]; then continue; fi
    exe="$out/t_$name"
    if ! build_with "$out/stage2" "$f" "$exe"; then
        fail=$((fail + 1)); failed+=("$name (build)"); continue
    fi
    want_out=$(./ail run "$f" 2> "$out/want.err" < /dev/null); want_rc=$?
    # `ail run` prints the checker's warnings first; a built program has none. Compare the trap line.
    got_out=$("$exe" 2> "$out/got.err" < /dev/null); got_rc=$?
    if [ "$want_out" == "$got_out" ] && [ "$want_rc" == "$got_rc" ] &&
        [ "$(grep -m1 'trap\[' "$out/want.err")" == "$(grep -m1 'trap\[' "$out/got.err")" ]; then
        pass=$((pass + 1))
    else
        fail=$((fail + 1)); failed+=("$name (output)")
        # What differs, so the report says why (ail run first, then the built program).
        { echo "- $name: rc $want_rc vs $got_rc"
          diff <(printf '%s\n%s\n' "$want_out" "$(head -3 "$out/want.err")") <(printf '%s\n%s\n' "$got_out" "$(head -3 "$out/got.err")") | head -8
        } | sed 's/^/    /' >> "$report"
    fi
    rm -f "$exe" "$exe.hex" "$exe.err"
done
say "$pass passed, $fail failed"
# A stage T that ran nothing is not a pass: name why (no programs found, or ONLY matched none).
# `ONLY=none` skips it on purpose (tools/release/seed.sh needs only the fixed point), and says so.
if [ "${ONLY:-}" = none ]; then
    say "(stage T skipped: ONLY=none)"
elif [ $((pass + fail)) -eq 0 ]; then
    blocked "stage T" "no native test program ran ($(ls "$tests"/*.ail 2>/dev/null | wc -l | tr -d ' ') in $tests, ONLY='${ONLY:-}', cwd $(pwd))"
fi
for n in ${failed[@]+"${failed[@]}"}; do say "  - $n"; done
[ "$fail" -eq 0 ] || blocked "stage T" "$fail native test programs differ when built by the self-hosted compiler"
say ""
if [ -n "${TESTS_ONLY:-}" ]; then
    say "OK (stage T only: no fixed point was checked)."
elif [ "${ONLY:-}" = none ]; then
    say "OK: the native compiler builds itself to a fixed point (native tests skipped: ONLY=none)."
else
    say "OK: the native compiler builds itself to a fixed point and passes the native tests."
fi
