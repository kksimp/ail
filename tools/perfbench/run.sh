#!/usr/bin/env bash
# Timing and drag race (ADR 0175, ADR 0460): min and median wall time per program.
#
#   tools/perfbench/run.sh                 # AIL (`ail run`) vs Python, 5 runs
#   tools/perfbench/run.sh 10              # 10 runs per cell
#   tools/perfbench/run.sh --all           # + native (`ail build`) + C (clang, gcc -O2)
#   tools/perfbench/run.sh --native --c --python --md docs/design/drag-race.md
#   tools/perfbench/run.sh --all --programs tools/perfbench/programs/race_smoke.ail
#
# Builds the native CLI with ./ail, then times `ail run` on each program. Twins
# NAME.c and NAME.py beside a program (or in the tokbench corpus) fill the C and
# Python columns; ratios below 1 beat the reference. Needs python3 (measurement only,
# ADR 0070). Columns that can't run (no `ail build` yet, no compiler, no twin) say so.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
command -v python3 >/dev/null 2>&1 || { echo "perfbench: python3 is required" >&2; exit 1; }
"$root/ail" run "$root/examples/hello.ail" >/dev/null   # builds the native CLI
exec python3 -I "$here/race.py" "$@"
