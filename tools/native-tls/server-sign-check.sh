#!/usr/bin/env bash
# The TLS server's signatures (ADR 1237: RSASSA-PSS with SHA-256/384/512 and ECDSA P-256, std.net.47) against OpenSSL.
# Throwaway keys of each type and file format are made with `openssl`, tools/native-tls/sign.ail signs messages of
# several sizes with the server's own key loader and signer (built in a temporary compiler directory with the test
# hook of tools/native-tls/signhook.ail, as prim-check.sh does), and `openssl dgst -verify` must accept each
# signature and refuse it for a changed message and for a changed signature byte. PSS signatures of one message must
# differ (a fresh salt); ECDSA ones must not (RFC 6979). The Wycheproof cases of the PSS and ECDSA verifiers are in
# prim-check.sh. Needs OpenSSL 1.1.1 or newer; prints `N ok, M failed`; exits 0 with a SKIP line when the tree has
# no compiler/rt/native/tls_keys.ail (before the TLS server, ADR 1237, lands).
# Run from anywhere on a designer machine: tools/native-tls/server-sign-check.sh
set -euo pipefail
cd "$(dirname "$0")/../.."
if [ ! -f compiler/rt/native/tls_keys.ail ]; then echo "SKIP: no tls_keys.ail in this tree; 0 failed"; exit 0; fi
openssl=openssl
if command -v brew > /dev/null && [ -x "$(brew --prefix openssl@3 2>/dev/null)/bin/openssl" ]; then
  openssl="$(brew --prefix openssl@3)/bin/openssl"
fi
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
cp -r compiler "$tmp/compiler"
cp -r std "$tmp/std"
cp tools/native-tls/signhook.ail "$tmp/compiler/rt/native/signhook.ail"
python3 - "$tmp/compiler/rt/native/hash.ail" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
old = 'fn hash_hmac_sha256(key str, s str) str\n'
assert old in s
s = s.replace(old, 'fn hash_hmac_sha256(op str, args str) str\n    tlssign_call(op, args)\n\nfn hash_hmac_sha256_real(key str, s str) str\n', 1)
open(p, 'w').write(s)
PY
AIL_ROOT="$PWD" AIL_COMPILER="$tmp/compiler" .native/ail build tools/native-tls/sign.ail -o "$tmp/sign" > /dev/null
ok=0; bad=0
fail() { bad=$((bad + 1)); echo "FAIL $*"; }
pass() { ok=$((ok + 1)); }
cd "$tmp"
# Keys: name, creation arguments. Each is also written in the other PEM form the loader reads.
"$openssl" req -x509 -newkey rsa:2048 -nodes -keyout rsa2048.p8 -out rsa2048.pem -subj "/CN=localhost" -days 2 2>/dev/null
"$openssl" req -x509 -newkey rsa:3072 -nodes -keyout rsa3072.p8 -out rsa3072.pem -subj "/CN=localhost" -days 2 2>/dev/null
"$openssl" req -x509 -newkey ec -pkeyopt ec_paramgen_curve:prime256v1 -nodes -keyout ec.p8 -out ec.pem -subj "/CN=localhost" -days 2 2>/dev/null
"$openssl" rsa -in rsa2048.p8 -traditional -out rsa2048.p1 2>/dev/null
"$openssl" rsa -in rsa3072.p8 -traditional -out rsa3072.p1 2>/dev/null
"$openssl" ec -in ec.p8 -out ec.sec1 2>/dev/null
for k in rsa2048.p8 rsa3072.p8 ec.p8; do "$openssl" pkey -in "$k" -pubout -out "${k%.p8}.pub" 2>/dev/null; done
for m in 0 1 64 1000; do dd if=/dev/urandom of="m$m" bs=1 count="$m" 2>/dev/null; done  # macOS `head -c 0` is an error
# One signing: KEY CERT SCHEME MSG OUT (the signature as raw bytes in OUT).
signit() {
  ./sign "$1" "$2" "$3" "$4" "$5.hex"
  if grep -q '^error' "$5.hex"; then echo "$(cat "$5.hex")"; return 1; fi
  # python3 for the hex: xxd is not on every host.
  python3 -c "import sys; open(sys.argv[2], 'wb').write(bytes.fromhex(open(sys.argv[1]).read().strip()))" "$5.hex" "$5"
}
verify() { # PUB DIGEST [SIGOPTS...] -- SIG MSG
  local pub=$1 md=$2; shift 2
  local opts=(); while [ "$1" != "--" ]; do opts+=("$1"); shift; done; shift
  "$openssl" dgst "-$md" ${opts[@]+"${opts[@]}"} -verify "$pub" -signature "$1" "$2" 2>&1 | grep -q 'Verified OK'
}
tamper() { python3 -c "import sys; b=bytearray(open(sys.argv[1],'rb').read()); b[int(sys.argv[3])%len(b)]^=1; open(sys.argv[2],'wb').write(b)" "$1" "$2" "$3"; }
for key in rsa2048.p8 rsa2048.p1 rsa3072.p8 rsa3072.p1; do
  base=${key%.*}
  for pair in "2052 sha256 32" "2053 sha384 48" "2054 sha512 64"; do
    set -- $pair; scheme=$1; md=$2; salt=$3
    opts=(-sigopt rsa_padding_mode:pss -sigopt "rsa_pss_saltlen:$salt" -sigopt "rsa_mgf1_md:$md")
    for m in 0 1 64 1000; do
      sig="$key.$scheme.$m.sig"
      if ! signit "$key" "$base.pem" "$scheme" "m$m" "$sig" > "$sig.err" 2>&1; then fail "$key $md m$m: $(cat "$sig.err")"; continue; fi
      if verify "$base.pub" "$md" ${opts[@]+"${opts[@]}"} -- "$sig" "m$m"; then pass; else fail "$key $md m$m: openssl refuses the signature"; fi
      tamper "$sig" "$sig.t" "$m"
      if verify "$base.pub" "$md" ${opts[@]+"${opts[@]}"} -- "$sig.t" "m$m"; then fail "$key $md m$m: openssl accepts a changed signature"; else pass; fi
      if [ "$m" != 0 ]; then
        if verify "$base.pub" "$md" ${opts[@]+"${opts[@]}"} -- "$sig" "m0"; then fail "$key $md m$m: openssl accepts it for another message"; else pass; fi
      fi
    done
    signit "$key" "$base.pem" "$scheme" m64 "$key.$scheme.again" > /dev/null 2>&1 || true
    if cmp -s "$key.$scheme.64.sig" "$key.$scheme.again"; then fail "$key $md: two PSS signatures of one message are equal (no fresh salt)"; else pass; fi
  done
done
for key in ec.p8 ec.sec1; do
  for m in 0 1 64 1000; do
    sig="$key.$m.sig"
    if ! signit "$key" ec.pem 1027 "m$m" "$sig" > "$sig.err" 2>&1; then fail "$key m$m: $(cat "$sig.err")"; continue; fi
    if verify ec.pub sha256 -- "$sig" "m$m"; then pass; else fail "$key m$m: openssl refuses the ECDSA signature"; fi
    tamper "$sig" "$sig.t" 20
    if verify ec.pub sha256 -- "$sig.t" "m$m"; then fail "$key m$m: openssl accepts a changed ECDSA signature"; else pass; fi
  done
  signit "$key" ec.pem 1027 m64 "$key.again" > /dev/null 2>&1 || true
  if cmp -s "$key.64.sig" "$key.again"; then pass; else fail "$key: ECDSA signatures of one message differ (RFC 6979 is deterministic)"; fi
done
# A key that does not belong to the certificate is refused, never used.
if signit rsa3072.p8 rsa2048.pem 2052 m64 "mismatch.sig" > mismatch.err 2>&1; then fail "a key that does not match the certificate was used"; else pass; fi
echo "$ok ok, $bad failed"
[ "$bad" = 0 ]
