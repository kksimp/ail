#!/usr/bin/env bash
# HTTP/2 (std.net.31, ADR 0913) end to end: h2_client.ail against h2_server.py
# (TLS 1.3 with ALPN `h2`, on 127.0.0.1) must print h2_client.out; against a
# server that picks `http/1.1` or sends no ALPN, and with AIL_HTTP2=0, the
# client falls back to HTTP/1.1. Needs python3 with OpenSSL 1.1.1+ (TLS 1.3).
# Run from anywhere: tools/native-net/h2-check.sh
set -euo pipefail
cd "$(dirname "$0")/../.."
tmp=$(mktemp -d)
srv=""
stop() { [ -n "$srv" ] && kill "$srv" 2>/dev/null || true; srv=""; }
trap 'stop; rm -rf "$tmp"' EXIT
x509=tests/data/net/x509
cat "$x509/localhost_leaf.pem" "$x509/inter.pem" > "$tmp/chain.pem"
export SSL_CERT_FILE="$PWD/$x509/root.pem"
start() { # MODE: starts the server and sets $url
  stop
  python3 -I tools/native-net/h2_server.py "$tmp/chain.pem" "$x509/localhost_leaf.key.pem" "$1" > "$tmp/port" &
  srv=$!
  for _ in $(seq 100); do [ -s "$tmp/port" ] && break; sleep 0.1; done
  url="https://localhost:$(head -1 "$tmp/port")"
}
./ail build tools/native-net/h2_client.ail -o "$tmp/h2c" > /dev/null
./ail build tools/native-net/h2_fallback.ail -o "$tmp/fb" > /dev/null
start h2
"$tmp/h2c" "$url" > "$tmp/got" 2>&1 || true
diff tools/native-net/h2_client.out "$tmp/got" && echo "h2_client: same"
got=$(AIL_HTTP2=0 "$tmp/fb" "$url" 2>&1 || true)
[ "$got" = "200 fallback http/1.1" ] && echo "AIL_HTTP2=0: http/1.1" || { echo "AIL_HTTP2=0: $got"; exit 1; }
for mode in h1 none; do
  start "$mode"
  got=$("$tmp/fb" "$url" 2>&1 || true)
  [ "$got" = "200 fallback http/1.1" ] && echo "$mode: http/1.1" || { echo "$mode: $got"; exit 1; }
done
