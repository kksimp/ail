#!/usr/bin/env bash
# Self-host bootstrap harness (ADR 0022, native self-compile push). Proves the
# native compiler can build itself, stage by stage, and says where it stops.
#
#   tools/self-host/run.sh [OUT_DIR]
#
# Stages (each runs only if the one before it worked):
#   0  needs: the library calls the compiler's own source makes (fs, io, env)
#      that the native backend can't compile yet.
#   1  `./ail build compiler/main.ail` (Rust-hosted) makes the native compiler.
#   2  stage 1 builds compiler/main.ail again.
#   3  stage 2 builds it a third time; stage 2 and 3 must be byte-identical
#      (the fixed point), and stage 1 and 2 too (same compiler, same output).
#   T  the native test programs (bootstrap/tests/native/run/*.ail) built with
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
tests=bootstrap/tests/native/run
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

# build SRC with compiler CMD into OUT; the compiler prints hex
# The derive output (ADR 0807) of SRC, as the Rust-hosted `ail build` makes it;
# the native compiler reads it through AIL_DERIVED.
build_with() { # compiler src out
    local cc="$1" src="$2" dst="$3"
    ./ail build "$src" --derived-out "$dst.derived" > /dev/null 2>&1 || : > "$dst.derived"
    AIL_DERIVED="$dst.derived" "$cc" build "$src" "$rt" "$TARGET" > "$dst.hex" 2> "$dst.err" || return 1
    unhex "$dst" < "$dst.hex"
}

say "# Self-host bootstrap ($(date -u +%Y-%m-%dT%H:%M:%SZ), $(git rev-parse --short HEAD))"
say ""

# ---- stage 0: what the compiler's source needs from the library ----
# `env.args` is native. With the native OS layer (lane 177, ADR 0580, file
# compiler/mir/lower_os.ail) `fs.*`, `io.*` are native too, but only in `else`
# forms; a call ending in `?` or `else err` waits for error values (lane 175).
say "## Stage 0: library calls in compiler/ (outside rt)"
os_layer=0
[ -f compiler/mir/lower_os.ail ] && os_layer=1
srcs=$(find compiler -name '*.ail' ! -path 'compiler/rt/*')
needs=$(grep -hoE '\b(fs|io|env)\.[a-z_]+\(' $srcs | sed 's/($//' | sort | uniq -c | sort -rn)
missing=0
while read -r n name; do
    [ -z "${name:-}" ] && continue
    if [ "$name" == "env.args" ]; then
        say "- $name ($n uses): native"
    elif [ "$os_layer" == 1 ]; then
        q=$(grep -hE "\b${name//./\\.}\(" $srcs | grep -cE '\)\?|else err')
        if [ "$q" -gt 0 ]; then
            say "- $name ($n uses): native in else forms; $q use(s) with \`?\` or \`else err\` need error values (lane 175)"
            missing=$((missing + 1))
        else
            say "- $name ($n uses): native"
        fi
    else
        say "- $name ($n uses): NOT native (no OS layer yet, lane 177)"
        missing=$((missing + 1))
    fi
done <<< "$needs"
say ""

# ---- stage 1: the Rust-hosted build ----
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
        blocked "stage 1" "the Rust-hosted build of compiler/main.ail reports native.subset.1 problems"
    fi
fi
say ""

# ---- stage 2 and 3: the fixed point ----
if [ "$TARGET" != "$host_target" ]; then
    say "OK (build only: $TARGET can't run on this $host_target host, so stages 2, 3 and T were skipped)."
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
        say "stage 1 != stage 2: the native compiler's output differs from the interpreted one's"
        cmp "$out/stage1" "$out/stage2" 2>&1 | head -1 | tee -a "$report"
        blocked "stage 2" "stage-1 and stage-2 executables differ; the native compiler miscompiles something"
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
for f in "$tests"/*.ail; do
    name=$(basename "$f" .ail)
    if [ -n "${ONLY:-}" ] && [[ ",$ONLY," != *",$name,"* ]]; then continue; fi
    exe="$out/t_$name"
    if ! build_with "$out/stage2" "$f" "$exe"; then
        fail=$((fail + 1)); failed+=("$name (build)"); continue
    fi
    want_out=$(./ail run "$f" 2> "$out/want.err"); want_rc=$?
    got_out=$("$exe" 2> "$out/got.err"); got_rc=$?
    if [ "$want_out" == "$got_out" ] && [ "$want_rc" == "$got_rc" ] &&
        [ "$(head -1 "$out/want.err")" == "$(head -1 "$out/got.err")" ]; then
        pass=$((pass + 1))
    else
        fail=$((fail + 1)); failed+=("$name (output)")
    fi
    rm -f "$exe" "$exe.hex" "$exe.err"
done
say "$pass passed, $fail failed"
for n in ${failed[@]+"${failed[@]}"}; do say "  - $n"; done
[ "$fail" -eq 0 ] || blocked "stage T" "$fail native test programs differ when built by the self-hosted compiler"
say ""
if [ -n "${TESTS_ONLY:-}" ]; then
    say "OK (stage T only: no fixed point was checked)."
else
    say "OK: the native compiler builds itself to a fixed point and passes the native tests."
fi
