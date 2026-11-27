#!/usr/bin/env bash
# The native TLS 1.3 server (ADR 1237, std.net.46-48) against outside clients:
# `openssl s_client` with each cipher suite, a HelloRetryRequest (a client
# whose first key share is P-256), TLS 1.2 only (refused), RSA and ECDSA keys,
# and `curl` over HTTP/1.1 with ALPN, all through a throwaway CA.
# Needs OpenSSL 1.1.1 or newer (on macOS the Homebrew openssl@3 is used when
# installed; the system LibreSSL lacks some of these options).
# Run from the repository root on a designer machine: tools/native-tls/server-check.sh
set -euo pipefail
cd "$(dirname "$0")/../.."
openssl=openssl
if command -v brew > /dev/null && [ -x "$(brew --prefix openssl@3 2>/dev/null)/bin/openssl" ]; then
  openssl="$(brew --prefix openssl@3)/bin/openssl"
fi
tmp=$(mktemp -d)
srv=""
trap 'kill $srv 2>/dev/null || true; rm -rf "$tmp"' EXIT
cd "$tmp"
"$openssl" req -x509 -newkey ec -pkeyopt ec_paramgen_curve:prime256v1 -nodes -keyout ca.key -out ca.pem \
  -subj "/CN=AIL Local CA" -days 30 -addext "basicConstraints=critical,CA:TRUE" \
  -addext "keyUsage=critical,keyCertSign,cRLSign" 2>/dev/null
printf "subjectAltName=DNS:localhost,IP:127.0.0.1\nextendedKeyUsage=serverAuth\nkeyUsage=digitalSignature\nbasicConstraints=CA:FALSE\n" > ext.cnf
"$openssl" req -newkey ec -pkeyopt ec_paramgen_curve:prime256v1 -nodes -keyout ec.key -out ec.csr -subj "/CN=localhost" 2>/dev/null
"$openssl" x509 -req -in ec.csr -CA ca.pem -CAkey ca.key -CAcreateserial -out ec.pem -days 30 -extfile ext.cnf 2>/dev/null
"$openssl" req -newkey rsa:2048 -nodes -keyout rsa.key -out rsa.csr -subj "/CN=localhost" 2>/dev/null
"$openssl" x509 -req -in rsa.csr -CA ca.pem -CAkey ca.key -CAcreateserial -out rsa.pem -days 30 -extfile ext.cnf 2>/dev/null
cat ec.pem ca.pem > ec-chain.pem
cd - > /dev/null
cat > "$tmp/srv.ail" <<'P'
use std.net.http

fn hello(r net.Request) net.Response
    http.text(200, "hello over tls {r.path}")

fn main()
    a := env.args()
    http.serve_tls("127.0.0.1:{a[0]}", a[1], a[2], hello) else err
        print("serve: {err}")
P
./ail build "$tmp/srv.ail" -o "$tmp/srv" > /dev/null
port=$((20000 + RANDOM % 20000))
start() { # cert key
  "$tmp/srv" "$port" "$1" "$2" > "$tmp/srv.log" 2>&1 &
  srv=$!
  sleep 1
}
stop() { kill $srv 2>/dev/null || true; wait $srv 2>/dev/null || true; srv=""; }
fail() { echo "FAIL $1"; echo "$2" | tail -20; cat "$tmp/srv.log"; exit 1; }
# An outside client, stopped after 10 seconds.
talk() {
  printf 'GET /x HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n' |
    perl -e 'alarm shift; exec @ARGV' 10 "$openssl" s_client -connect "127.0.0.1:$port" -servername localhost -CAfile "$tmp/ca.pem" \
      -verify_return_error -ign_eof "$@" 2>&1 || true
}
served() { # name args...: the request is answered
  local name=$1; shift
  out=$(talk "$@")
  echo "$out" | grep -q "hello over tls /x" || fail "$name" "$out"
  echo "ok $name"
}
refused() { # name args...: the handshake fails
  local name=$1; shift
  out=$(talk "$@")
  if echo "$out" | grep -q "hello over tls"; then fail "$name" "$out"; fi
  echo "ok $name"
}
start "$tmp/ec-chain.pem" "$tmp/ec.key"
for s in TLS_CHACHA20_POLY1305_SHA256 TLS_AES_128_GCM_SHA256 TLS_AES_256_GCM_SHA384; do
  served "ECDSA $s" -tls1_3 -ciphersuites "$s"
done
served "HelloRetryRequest (P-256 share first)" -tls1_3 -groups P-256:X25519
refused "no X25519 refused" -tls1_3 -groups P-256
refused "TLS 1.2 refused" -tls1_2
out=$(curl -sS --cacert "$tmp/ca.pem" "https://localhost:$port/curl" 2>&1 || true)
[ "$out" = "hello over tls /curl" ] || fail "curl" "$out"
echo "ok curl: $out"
out=$(curl -sS "http://127.0.0.1:$port/" 2>&1 || true)
echo "$out" | grep -q "speaks HTTPS" || fail "plain http answered" "$out"
echo "ok plain http answered with 400"
stop
start "$tmp/rsa.pem" "$tmp/rsa.key"
served "RSA-PSS" -tls1_3 -sigalgs rsa_pss_rsae_sha256
served "RSA-PSS SHA-384" -tls1_3 -sigalgs rsa_pss_rsae_sha384
refused "RSA PKCS#1 only refused" -tls1_3 -sigalgs rsa_pkcs1_sha256
stop
echo "server-check: all passed"
