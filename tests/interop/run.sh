#!/usr/bin/env bash
# AIL std interop: compare the std modules with real reference implementations (tests/interop/README.md).
#   tests/interop/run.sh [module ...]     from the repo root; modules: compress image crypto encoding jwt binfmt tar
# Needs python3. Installs the reference libraries into a venv under $WORK; tools missing on the host (zstd, brotli, tar)
# make their cases SKIP, never pass. Exit 1 when any case FAILs. Results: $OUT/results.tsv and $OUT/summary.txt.
set -u
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
WORK=${WORK:-${TMPDIR:-/tmp}/ail-interop}
OUT=${OUT:-$WORK/out}
BIN=$WORK/bin
mkdir -p "$BIN" "$OUT"
cd "$ROOT" || exit 2
ALL="compress image crypto encoding jwt binfmt tar"
MODS=${*:-$ALL}
echo "tree $(git rev-parse --short=8 HEAD 2>/dev/null) on $(uname -sm)"
if [ ! -x "$WORK/venv/bin/python" ]; then
  python3 -m venv "$WORK/venv" && "$WORK/venv/bin/pip" -q install pillow cryptography PyJWT cbor2 msgpack zstandard brotli base58 pypng 2>&1 | tail -3
fi
PY=$WORK/venv/bin/python; [ -x "$PY" ] || PY=python3
for t in zstd brotli tar gzip; do command -v $t >/dev/null || echo "note: no $t CLI, python module or SKIP"; done
for m in $MODS; do
  rm -f "$BIN/cli_$m"
  ./ail build "tests/interop/$m/cli.ail" -o "$BIN/cli_$m" > "$OUT/build_$m.log" 2>&1 \
    || { echo "BUILD FAIL $m"; grep -m6 'error\[' "$OUT/build_$m.log"; }
done
BIN=$BIN WORK=$WORK OUT=$OUT "$PY" -I "$ROOT/tests/interop/interop.py" $MODS | tee "$OUT/summary.txt"
exit "${PIPESTATUS[0]}"
