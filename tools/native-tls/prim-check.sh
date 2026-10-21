#!/usr/bin/env bash
# The native TLS primitives (ADR 0836) against the test vectors of the Rust
# client (ADR 0366). The primitives have no public API, so this builds the
# checker in a temporary copy of compiler/ where `hash.hmac_sha256(op, args)`
# calls tools/native-tls/tlsprim.ail (test-only, never part of the product).
# Linux x86-64 always, and ARM64 under qemu when it is installed.
# Run from anywhere: tools/native-tls/prim-check.sh
set -euo pipefail
cd "$(dirname "$0")/../.."
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
cp -r compiler "$tmp/compiler"
mkdir -p "$tmp/bootstrap"
cp -r bootstrap/src "$tmp/bootstrap/src"
cp tools/native-tls/tlsprim.ail "$tmp/compiler/rt/native/tlsprim.ail"
python3 - "$tmp/compiler/mir/lower_os.ail" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
if '"hash"' not in s:
    s = s.replace('"proc"', '"proc", "hash"', 1)
hook = '''    if m == "hash" and name == "hmac_sha256"
        return os_call(os_fn("rt.tlsprim_call", [Ty.Str, Ty.Str], Ty.Str, false), args, npos)
'''
anchor = '    f := os_lookup(m, name) else\n        return unsupported'
assert anchor in s
s = s.replace(anchor, hook + anchor, 1)
open(p, 'w').write(s)
PY
export AIL_COMPILER="$tmp/compiler"
vectors=bootstrap/src/net/crypto/testdata/crosscheck.txt
prog=tools/native-tls/prim.ail
./ail build "$prog" -o "$tmp/x64" > /dev/null
"$tmp/x64" "$vectors" | tee "$tmp/out"
grep -q ' 0 failed' "$tmp/out"
qemu=$(command -v qemu-aarch64-static || command -v qemu-aarch64 || true)
if [ -n "$qemu" ]; then
  ./ail build "$prog" --target aarch64-linux -o "$tmp/a64" > /dev/null
  "$qemu" "$tmp/a64" "$vectors" | tee "$tmp/out64"
  grep -q ' 0 failed' "$tmp/out64"
fi
