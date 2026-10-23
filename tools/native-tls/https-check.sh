#!/usr/bin/env bash
# The native `https` client (ADR 0852) against a live `openssl s_server` with a
# throwaway CA, compared with the interpreter: a good chain (EC and RSA leaf),
# a wrong host name, an unknown CA and a bad SSL_CERT_FILE. Linux x86-64
# always, and ARM64 under qemu when it is installed.
# Run from anywhere: tools/native-tls/https-check.sh
set -euo pipefail
cd "$(dirname "$0")/../.."
tmp=$(mktemp -d)
srv=""
trap 'kill $srv 2>/dev/null || true; rm -rf "$tmp"' EXIT
cd "$tmp"
openssl req -x509 -newkey ec -pkeyopt ec_paramgen_curve:prime256v1 -nodes -keyout ca.key -out ca.pem \
  -subj "/CN=AIL Local CA" -days 30 -addext "basicConstraints=critical,CA:TRUE" \
  -addext "keyUsage=critical,keyCertSign,cRLSign" 2>/dev/null
openssl req -newkey ec -pkeyopt ec_paramgen_curve:prime256v1 -nodes -keyout leaf.key -out leaf.csr -subj "/CN=localhost" 2>/dev/null
printf "subjectAltName=DNS:localhost,IP:127.0.0.1\nextendedKeyUsage=serverAuth\nkeyUsage=digitalSignature\nbasicConstraints=CA:FALSE\n" > ext.cnf
openssl x509 -req -in leaf.csr -CA ca.pem -CAkey ca.key -CAcreateserial -out leaf.pem -days 30 -extfile ext.cnf 2>/dev/null
openssl req -x509 -newkey rsa:2048 -nodes -keyout rsa.key -out rsa.pem -subj "/CN=localhost" -days 30 \
  -addext "subjectAltName=DNS:localhost" -addext "basicConstraints=critical,CA:FALSE" \
  -addext "extendedKeyUsage=serverAuth" -addext "keyUsage=digitalSignature" 2>/dev/null
cd - > /dev/null
port=$((20000 + RANDOM % 20000))
cat > "$tmp/get.ail" <<'P'
use std.net

fn main()
    url := env.args()[0]
    r := net.get(url) else err
        print("error: {err}")
        return
    print("status {r.status} {r.body.len()} {r.body.starts_with("<HTML>")}")
P
check() { # name server-cert server-key cafile host
  openssl s_server -accept "$port" -cert "$2" -key "$3" -www -ciphersuites "${suite:-TLS_CHACHA20_POLY1305_SHA256}" \
    -tls1_3 > "$tmp/server.log" 2>&1 &
  srv=$!
  sleep 1
  url="https://$5:$port/"
  want=$(SSL_CERT_FILE="$4" ./ail run "$tmp/get.ail" "$url" 2>&1 | sed "s/$port/PORT/g")
  got=$(SSL_CERT_FILE="$4" "$tmp/x64" "$url" 2>&1 | sed "s/$port/PORT/g")
  if [ "$want" != "$got" ]; then echo "FAIL $1 (x86-64)"; echo " want $want"; echo " got  $got"; exit 1; fi
  if [ -n "$qemu" ]; then
    got=$(SSL_CERT_FILE="$4" "$qemu" "$tmp/a64" "$url" 2>&1 | sed "s/$port/PORT/g")
    if [ "$want" != "$got" ]; then echo "FAIL $1 (arm64)"; echo " want $want"; echo " got  $got"; exit 1; fi
  fi
  echo "ok $1: ${got:0:90}"
  kill $srv 2>/dev/null || true
  wait $srv 2>/dev/null || true
  srv=""
}
./ail build "$tmp/get.ail" -o "$tmp/x64" > /dev/null
qemu=$(command -v qemu-aarch64-static || command -v qemu-aarch64 || true)
if [ -n "$qemu" ]; then ./ail build "$tmp/get.ail" --target aarch64-linux -o "$tmp/a64" > /dev/null; fi
check "EC leaf under a CA" "$tmp/leaf.pem" "$tmp/leaf.key" "$tmp/ca.pem" localhost
check "RSA leaf, self-signed root" "$tmp/rsa.pem" "$tmp/rsa.key" "$tmp/rsa.pem" localhost
check "IP address in the SAN" "$tmp/leaf.pem" "$tmp/leaf.key" "$tmp/ca.pem" 127.0.0.1
suite=TLS_AES_128_GCM_SHA256 check "AES-128-GCM-SHA256" "$tmp/leaf.pem" "$tmp/leaf.key" "$tmp/ca.pem" localhost
suite=TLS_AES_256_GCM_SHA384 check "AES-256-GCM-SHA384" "$tmp/leaf.pem" "$tmp/leaf.key" "$tmp/ca.pem" localhost
# 127.0.0.2 is not routed on macOS: a certificate for localhost only, reached by 127.0.0.1.
check "wrong host name" "$tmp/rsa.pem" "$tmp/rsa.key" "$tmp/rsa.pem" 127.0.0.1
check "unknown CA" "$tmp/leaf.pem" "$tmp/leaf.key" "$tmp/rsa.pem" localhost
check "bad SSL_CERT_FILE" "$tmp/leaf.pem" "$tmp/leaf.key" /etc/hostname localhost
# Raw TLS: net.connect_tls with write_bytes and read_bytes (std.net.12, std.net.16).
cat > "$tmp/raw.ail" <<'P'
use std.net

fn main()
    c := net.connect_tls("localhost", env.args()[0].to_int() else 0) else err
        print("error: {err.kind}")
        return
    c.write_bytes(b"GET / HTTP/1.0\r\n\r\n") else err
        print("write failed")
        return
    mut total := 0
    mut first := b""
    while true
        got := c.read_bytes(4096) else err
            break
        if got.len() == 0
            break
        if first.len() == 0
            first = got[0..12]
        total += got.len()
    print("{first.text()} {total > 3000}")
P
openssl s_server -accept "$port" -cert "$tmp/leaf.pem" -key "$tmp/leaf.key" -www -tls1_3 > "$tmp/server.log" 2>&1 &
srv=$!
sleep 1
./ail build "$tmp/raw.ail" -o "$tmp/rawx" > /dev/null
want=$(SSL_CERT_FILE="$tmp/ca.pem" ./ail run "$tmp/raw.ail" "$port" 2>&1)
got=$(SSL_CERT_FILE="$tmp/ca.pem" "$tmp/rawx" "$port" 2>&1)
if [ "$want" != "$got" ] || [ "$got" != "HTTP/1.0 200 true" ]; then echo "FAIL raw TLS"; echo " want $want"; echo " got  $got"; exit 1; fi
echo "ok raw TLS conn: $got"
kill $srv 2>/dev/null || true
echo "https-check: all passed"
