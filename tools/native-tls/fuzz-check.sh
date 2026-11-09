#!/usr/bin/env bash
# Mutation fuzz of the native TLS parsers (X.509 and ServerHello), in-process:
# tools/native-tls/fuzz.ail built against the test hook of tools/native-tls/prim-check.sh.
#   tools/native-tls/fuzz-check.sh [N [SEED]]     N iterations (default 2000) from SEED (default 1)
# Runs in chunks of 250 iterations; a failure names its range, and iteration IT alone is
# reproduced by the built fuzz.ail with START=IT COUNT=1 and the same SEED.
set -euo pipefail
cd "$(dirname "$0")/../.."
n="${1:-2000}"
seed="${2:-1}"
chunk=250
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
cp -r compiler "$tmp/compiler"
cp -r std "$tmp/std"
cp tools/native-tls/tlsprim.ail "$tmp/compiler/rt/native/tlsprim.ail"
python3 - "$tmp/compiler/rt/native/hash.ail" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
old = 'fn hash_hmac_sha256(key str, s str) str\n'
assert old in s
s = s.replace(old, 'fn hash_hmac_sha256(op str, args str) str\n    tlsprim_call(op, args)\n\nfn hash_hmac_sha256_real(key str, s str) str\n', 1)
open(p, 'w').write(s)
PY
./ail check tools/native-tls/fuzz.ail > /dev/null
exe=
case "$(uname -sm)" in
  "Darwin arm64") host=aarch64-macos ;;
  "Linux aarch64"|"Linux arm64") host=aarch64-linux ;;
  MINGW*|MSYS*|CYGWIN*) host=x86_64-windows; exe=.exe ;;
  *) host=x86_64-linux ;;
esac
AIL_ROOT="$PWD" AIL_COMPILER="$tmp/compiler" .native/ail build tools/native-tls/fuzz.ail --target "$host" -o "$tmp/fuzz$exe" > /dev/null
start=0
while [ "$start" -lt "$n" ]; do
  count=$((n - start < chunk ? n - start : chunk))
  if ! "$tmp/fuzz$exe" "$start" "$count" "$seed" tests/data/net/x509/chain.txt tests/data/net/tls/illustrated-tls13.txt; then
    echo "FAIL: iterations $start..$((start + count - 1)), seed $seed (rerun one with: fuzz.ail START 1 SEED CHAIN_FILE TLS_FILE)"
    exit 1
  fi
  start=$((start + count))
done
echo "fuzz tls: $n iterations ok (seed $seed)"
