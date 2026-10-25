#!/usr/bin/env bash
# Native HTTPS through a proxy (std.net.30): the client sends `CONNECT
# example.test:PORT` to a scripted proxy (tunnel.py), which relays to a local
# `openssl s_server` whose certificate is for example.test. TLS runs inside the
# tunnel, so the request succeeds only if the tunnel carries the handshake.
# NO_PROXY for the host skips the proxy (then example.test doesn't resolve).
# Run from anywhere: tools/native-net/proxy-tunnel-check.sh
set -euo pipefail
cd "$(dirname "$0")/../.."
tmp=$(mktemp -d)
pids=""
trap 'kill $pids 2>/dev/null || true; rm -rf "$tmp"' EXIT
(
  cd "$tmp"
  openssl req -x509 -newkey ec -pkeyopt ec_paramgen_curve:prime256v1 -nodes -keyout ca.key -out ca.pem \
    -subj "/CN=AIL Local CA" -days 30 -addext "basicConstraints=critical,CA:TRUE" \
    -addext "keyUsage=critical,keyCertSign,cRLSign" 2>/dev/null
  openssl req -newkey ec -pkeyopt ec_paramgen_curve:prime256v1 -nodes -keyout leaf.key -out leaf.csr -subj "/CN=example.test" 2>/dev/null
  printf "subjectAltName=DNS:example.test\nextendedKeyUsage=serverAuth\nkeyUsage=digitalSignature\nbasicConstraints=CA:FALSE\n" > ext.cnf
  openssl x509 -req -in leaf.csr -CA ca.pem -CAkey ca.key -CAcreateserial -out leaf.pem -days 30 -extfile ext.cnf 2>/dev/null
)
cat > "$tmp/get.ail" <<'P'
use std.net

fn main()
    r := net.get(env.args()[0]) else err
        print("{err.kind}: {err.message}")
        return
    print("status {r.status} {r.body.starts_with("<HTML>")}")
P
./ail build "$tmp/get.ail" -o "$tmp/client" > /dev/null
tls=$((20000 + RANDOM % 20000))
openssl s_server -accept "$tls" -cert "$tmp/leaf.pem" -key "$tmp/leaf.key" -www -tls1_3 > "$tmp/server.log" 2>&1 &
pids="$!"
python3 tools/native-net/tunnel.py "$tls" > "$tmp/pport" 2> "$tmp/connect.log" &
pids="$pids $!"
while [ ! -s "$tmp/pport" ]; do sleep 0.1; done
pport=$(head -1 "$tmp/pport")
sleep 1
got=$(env -u http_proxy -u HTTP_PROXY -u https_proxy -u HTTPS_PROXY -u no_proxy -u NO_PROXY \
  SSL_CERT_FILE="$tmp/ca.pem" HTTPS_PROXY="http://127.0.0.1:$pport" "$tmp/client" "https://example.test:$tls/")
sleep 0.3
want="status 200 true"
[ "$got" = "$want" ] || { echo "FAIL tunnel: want '$want' got '$got'"; exit 1; }
grep -q "^CONNECT example.test:$tls " "$tmp/connect.log" || { echo "FAIL: proxy saw '$(cat "$tmp/connect.log")'"; exit 1; }
echo "proxy-tunnel-check: same"
