#!/usr/bin/env bash
# AIL std interop: compare the std modules with real reference implementations (tests/interop/README.md).
#   tests/interop/run.sh [module ...]     from the repo root; modules: compress image crypto encoding jwt binfmt tar, and the lanes' own scripts tls static wsdeflate
#   server modules (need docker, never in the default list): redis postgres mysql mariadb, run by servers.py
#   network modules: smtp sse s3 oauth2, run by net_servers.py (never in the default list)
# Needs python3. Installs the reference libraries into a venv under $WORK; tools missing on the host (zstd, brotli, tar)
# make their cases SKIP, never pass. Exit 1 when any case FAILs. Results: $OUT/results.tsv and $OUT/summary.txt.
set -u
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
WORK=${WORK:-${TMPDIR:-/tmp}/ail-interop}
OUT=${OUT:-$WORK/out}
BIN=$WORK/bin
mkdir -p "$BIN" "$OUT"
cd "$ROOT" || exit 2
ALL="compress image crypto aead encoding jwt binfmt tar tls static wsdeflate"
CLI="compress image crypto aead encoding jwt binfmt tar redis postgres mysql smtp sse s3 oauth2"
SERVERS="redis postgres mysql mariadb"   # need docker: run only when named (db-job.sh on ail-linux)
NETS="smtp sse s3 oauth2"   # network modules against aiosmtpd, curl and a standard parser, SeaweedFS, Ory Hydra (net_servers.py; s3 and oauth2 need docker, never in the default list)
MODS=${*:-$ALL}
LOCAL=""; SRV=""; NET=""; BUILD=""
for m in $MODS; do
  case " $NETS " in
    *" $m "*) NET="$NET $m"; case " $BUILD " in *" $m "*) ;; *) BUILD="$BUILD $m";; esac; continue;;
  esac
  case " $SERVERS " in
    *" $m "*) SRV="$SRV $m"; [ "$m" = mariadb ] && m=mysql; case " $BUILD " in *" $m "*) ;; *) BUILD="$BUILD $m";; esac;;
    *) LOCAL="$LOCAL $m"; BUILD="$BUILD $m";;
  esac
done
echo "tree $(git rev-parse --short=8 HEAD 2>/dev/null) on $(uname -sm)"
if [ ! -x "$WORK/venv/bin/python" ]; then
  python3 -m venv "$WORK/venv" && "$WORK/venv/bin/pip" -q install pillow cryptography PyJWT cbor2 msgpack zstandard brotli base58 pypng websockets aiosmtpd boto3 requests sseclient-py authlib flask 2>&1 | tail -3
fi
PY=$WORK/venv/bin/python; [ -x "$PY" ] || PY=python3
for t in zstd brotli tar gzip; do command -v $t >/dev/null || echo "note: no $t CLI, python module or SKIP"; done
for m in $BUILD; do
  case " $CLI " in *" $m "*) ;; *) continue;; esac
  rm -f "$BIN/cli_$m"
  ./ail build "tests/interop/$m/cli.ail" -o "$BIN/cli_$m" > "$OUT/build_$m.log" 2>&1 \
    || { echo "BUILD FAIL $m"; grep -m6 'error\[' "$OUT/build_$m.log"; }
done
RC=0
if [ -n "$LOCAL" ]; then
  ROOT=$ROOT BIN=$BIN WORK=$WORK OUT=$OUT "$PY" -I "$ROOT/tests/interop/interop.py" $LOCAL | tee "$OUT/summary.txt"; RC=${PIPESTATUS[0]}
fi
if [ -n "$SRV" ]; then
  BIN=$BIN WORK=$WORK OUT=$OUT "$PY" -I "$ROOT/tests/interop/servers.py" $SRV | tee "$OUT/summary-servers.txt"; R2=${PIPESTATUS[0]}
  [ "$R2" -ne 0 ] && RC=$R2
fi
if [ -n "$NET" ]; then
  BIN=$BIN WORK=$WORK OUT=$OUT "$PY" -I "$ROOT/tests/interop/net_servers.py" $NET | tee "$OUT/summary-net.txt"; R3=${PIPESTATUS[0]}
  [ "$R3" -ne 0 ] && RC=$R3
fi
exit "$RC"
