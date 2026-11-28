#!/usr/bin/env bash
# compress.unbrotli against the Brotli project's reference encoder and decoder
# (ADR 1241): about 650 streams over qualities, windows and modes, and 4000
# damaged ones that must fail exactly when the reference decoder fails. Run
# from the repository root: tools/native-brotli/run.sh
set -euo pipefail
cd "$(dirname "$0")/../.."
work="$(mktemp -d "${TMPDIR:-/tmp}/ail-brotli.XXXXXX")"
trap 'rm -rf "$work"' EXIT
py=python3
if ! "$py" -c "import brotli" 2>/dev/null; then
    "$py" -m venv "$work/venv" >/dev/null
    "$work/venv/bin/pip" install -q --disable-pip-version-check brotli
    py="$work/venv/bin/python"
fi
"$py" tools/native-brotli/vectors.py "$work/vec" tests/data/brotli/alice29.txt.br
./ail build tools/native-brotli/check.ail -o "$work/check"
"$work/check" "$work/vec"
