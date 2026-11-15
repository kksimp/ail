#!/usr/bin/env bash
# The native TLS primitives (ADR 0836) against the test vectors of the Rust
# client (ADR 0366) and Project Wycheproof (ADR 1118). The primitives have no public API, so this builds the
# checker in a temporary copy of compiler/ where `hash.hmac_sha256(op, args)`
# calls tools/native-tls/tlsprim.ail (test-only, never part of the product).
# Runs on the host (Linux x86-64, Linux ARM64 or macOS ARM64), and ARM64 Linux under qemu when it is installed.
# Run from anywhere: tools/native-tls/prim-check.sh
set -euo pipefail
cd "$(dirname "$0")/../.."
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
./ail check tools/native-tls/prim.ail > /dev/null
nbuild() { AIL_ROOT="$PWD" AIL_COMPILER="$tmp/compiler" .native/ail build "$@"; }
vectors="tests/data/net/crypto/crosscheck.txt tests/data/net/crypto/signatures.txt tests/data/net/x509/chain.txt tools/native-tls/vectors384.txt tests/data/net/crypto/wycheproof-sigs.txt tests/data/net/crypto/wycheproof.txt"
prog=tools/native-tls/prim.ail
exe=
case "$(uname -sm)" in
  "Darwin arm64") host=aarch64-macos ;;
  "Linux aarch64"|"Linux arm64") host=aarch64-linux ;;
  MINGW*|MSYS*|CYGWIN*) host=x86_64-windows; exe=.exe ;;
  *) host=x86_64-linux ;;
esac
nbuild "$prog" --target "$host" -o "$tmp/x64$exe" > /dev/null
"$tmp/x64$exe" $vectors | tee "$tmp/out"
grep -q ' 0 failed' "$tmp/out"
qemu=$(command -v qemu-aarch64-static || command -v qemu-aarch64 || true)
if [ -n "$qemu" ]; then
  nbuild "$prog" --target aarch64-linux -o "$tmp/a64" > /dev/null
  "$qemu" "$tmp/a64" $vectors | tee "$tmp/out64"
  grep -q ' 0 failed' "$tmp/out64"
fi
