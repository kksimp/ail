#!/usr/bin/env bash
# The timing test of the in-house constant-time code (ADR 1118; docs/design/crypto-assurance.md, "Constant time").
# Builds tools/native-tls/timing.ail in a temporary copy of compiler/ where `hash.hmac_sha256("time", ...)` calls
# tools/native-tls/tlsprim.ail (test-only, never part of the product), then runs it: a fixed secret against random
# secrets for `ct_eq`, X25519, ChaCha20-Poly1305, AES-GCM and Poly1305, and Welch's t between the two timing
# distributions. A comparison that stops at the first difference is the control and must be flagged.
#   tools/native-tls/timing-check.sh [SAMPLES_PER_CLASS]      (default 1500; about a minute)
# Not part of the gate (a busy machine gives false alarms): run it on an idle machine, on a schedule. On Linux it
# pins the run to one core with `taskset` when that is installed. Prints `TIMING GREEN` or `TIMING RED`.
set -euo pipefail
cd "$(dirname "$0")/../.."
samples="${1:-1500}"
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
# `./ail build` pins AIL_COMPILER to this checkout, so call the native CLI itself.
./ail check tools/native-tls/timing.ail > /dev/null
case "$(uname -sm)" in
  "Darwin arm64") host=aarch64-macos ;;
  "Linux aarch64"|"Linux arm64") host=aarch64-linux ;;
  *) host=x86_64-linux ;;
esac
AIL_ROOT="$PWD" AIL_COMPILER="$tmp/compiler" .native/ail build tools/native-tls/timing.ail --target "$host" -o "$tmp/timing" > /dev/null
pin=""
if command -v taskset >/dev/null 2>&1; then pin="taskset -c 1"; fi
$pin "$tmp/timing" "$samples" | tee "$tmp/out"
grep -q '^TIMING GREEN' "$tmp/out"
