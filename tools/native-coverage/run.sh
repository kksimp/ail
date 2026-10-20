#!/usr/bin/env bash
# Native coverage tracker (ADR 0583): which programs `ail build` handles, and why the rest fail.
#
#   tools/native-coverage/run.sh               # all programs; writes the report and results.json
#   tools/native-coverage/run.sh --no-write    # print only
#   tools/native-coverage/run.sh --only json   # programs whose path contains "json" (print only)
#
# Measurement only: it is not a merge gate. Needs python3; uses ./ail, which builds the bootstrap.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
command -v python3 >/dev/null 2>&1 || { echo "native-coverage: python3 is required" >&2; exit 1; }
"$root/ail" run "$root/examples/hello.ail" >/dev/null   # builds the bootstrap
exec python3 -I "$here/coverage.py" "$@"
