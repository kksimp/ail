#!/bin/sh
# std.protobuf against the reference implementation (ADR 1293). Run from the
# repository root: tools/protobuf-interop/run.sh. Needs python3 with venv and
# network for pip; builds nothing but the venv under $TMPDIR.
set -e
root=$(cd "$(dirname "$0")/../.." && pwd)
tmp=${TMPDIR:-/tmp}/ail-protobuf-interop
mkdir -p "$tmp"
python3 -m venv "$tmp/venv"
"$tmp/venv/bin/pip" install -q protobuf grpcio-tools
"$tmp/venv/bin/python" "$root/tools/protobuf-interop/check.py" "$root/ail" "$tmp/work"
