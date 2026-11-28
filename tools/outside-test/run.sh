#!/bin/sh
# Outside-agent kit: `tools/outside-test/run.sh make KIT` or `... score KIT` (see README.md). Run from the repository root.
set -e
cd "$(dirname "$0")/../.."
[ -x ./ail ] || { echo "run from an AIL checkout with ./ail"; exit 1; }
./ail --version >/dev/null
command -v python3 >/dev/null || { echo "python3 is required"; exit 1; }
exec python3 tools/outside-test/outside.py "$@"
