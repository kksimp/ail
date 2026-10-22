#!/bin/sh
# Multi-core drag race in one command (macOS and Linux): run from the repository root,
#   tools/perfbench/parallel/run.sh [--runs N] [--python] [NAME...]
# Prints a Markdown table; copy it into docs/design/native-performance.md. On macOS install
# libomp (brew install libomp) for an OpenMP C column.
cd "$(dirname "$0")/../../.." || exit 1
exec python3 -I tools/perfbench/parallel/run.py --python "$@"
