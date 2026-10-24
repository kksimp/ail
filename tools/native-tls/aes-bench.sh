#!/usr/bin/env bash
# AES-128-GCM throughput of the native TLS runtime (ADR 0836): seal and open
# MIB MiB (default 64) of zeros, timed in seconds and reported in MB/s.
#   tools/native-tls/aes-bench.sh [MIB] [GIT_REF]
# With GIT_REF the runtime comes from that commit (`dfcf59bb^` is the software-only
# runtime before the hardware kernels), so one machine gives a before and an after.
# Needs the native CLI (.native/ail): run `./ail check tools/native-tls/prim.ail` once.
set -euo pipefail
cd "$(dirname "$0")/../.."
mib="${1:-64}"
ref="${2:-}"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
if [ -n "$ref" ]; then
  git archive "$ref" compiler bootstrap/src | tar -x -C "$tmp"
else
  cp -r compiler "$tmp/compiler"
  mkdir -p "$tmp/bootstrap"
  cp -r bootstrap/src "$tmp/bootstrap/src"
fi
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
cat > "$tmp/bench.ail" <<'AIL'
use std.hash

fn main()
    start := time.now_ms()
    r := hash.hmac_sha256("aesbench", env.args()[0])
    print("{r} {time.now_ms() - start}")
AIL
./ail check "$tmp/bench.ail" > /dev/null
case "$(uname -sm)" in
  "Darwin arm64") host=aarch64-macos ;;
  "Linux aarch64"|"Linux arm64") host=aarch64-linux ;;
  *) host=x86_64-linux ;;
esac
AIL_ROOT="$PWD" AIL_COMPILER="$tmp/compiler" .native/ail build "$tmp/bench.ail" --target "$host" -o "$tmp/bench" > /dev/null
out=$("$tmp/bench" "$mib")
ms="${out##* }"
[ "${out%% *}" = ok ] || { echo "FAIL: $out"; exit 1; }
# Seal plus open is two passes over the data.
python3 -c "print(f'aes-128-gcm {$mib} MiB seal+open: {$ms} ms, {2*$mib*1.048576/($ms/1000):.1f} MB/s')"
