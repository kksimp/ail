#!/usr/bin/env bash
# Native blockers (day 3): which examples `ail build` can't build yet, and the first construct each one stops at.
#
#   tools/native-blockers/run.sh              # every example, one build at a time; prints the tables
#   tools/native-blockers/run.sh --only json  # examples whose file name contains "json"
#
# Prints a table of the first rejection's group, a total line (N of M examples build natively) and the
# ffi.native.1 examples on their own. Writes nothing to the repo. Measurement only: not a merge gate, and not run by
# ./ail selftest. Needs python3; uses ./ail, which builds the bootstrap.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
command -v python3 >/dev/null 2>&1 || { echo "native-blockers: python3 is required" >&2; exit 1; }
"$root/ail" run "$root/examples/hello.ail" >/dev/null   # builds the bootstrap
exec python3 -I "$here/blockers.py" "$@"
