#!/usr/bin/env bash
# Scripted agent sessions, no model: one token table over every session in tasks/ (ADR 0916, amended 2026-11-10).
#   tools/agentbench/replay/run.sh [--only TASK] [--json] [--save FILE] [--compare FILE]
# Needs `ail` (./ail in the repository, or AIL=/path) and python3; run it on a designer machine (ADR 0842).
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
exec python3 -I "$here/replay.py" "$@"
