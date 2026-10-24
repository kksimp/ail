#!/usr/bin/env bash
# Native HTTP proxy support (std.net.30): a scripted proxy (proxy.py)
# sees an absolute-form request with its Proxy-Authorization, a NO_PROXY host
# skips it, a loopback host never uses it, and a refused CONNECT is a
# `protocol` failure. Run from anywhere: tools/native-net/proxy-check.sh
set -euo pipefail
cd "$(dirname "$0")/../.."
tmp=$(mktemp -d)
srv=""
trap 'kill "$srv" 2>/dev/null || true; rm -rf "$tmp"' EXIT
./ail build tools/native-net/proxy_client.ail -o "$tmp/client" > /dev/null
python3 tools/native-net/proxy.py 3 > "$tmp/port" &
srv=$!
while [ ! -s "$tmp/port" ]; do sleep 0.1; done
port=$(head -1 "$tmp/port")
run() { env -u http_proxy -u HTTP_PROXY -u https_proxy -u HTTPS_PROXY -u no_proxy -u NO_PROXY "$@" "$tmp/client" "$url"; }
url="http://example.test/a?x=1"
{
  echo "proxied:";    run HTTP_PROXY="http://user:pw@127.0.0.1:$port"
  echo "lowercase:";  run http_proxy="127.0.0.1:$port"
  echo "no_proxy:";   run HTTP_PROXY="http://127.0.0.1:$port" NO_PROXY="other.test, .example.test"
  url="https://example.test/"
  echo "connect:";    run HTTPS_PROXY="http://127.0.0.1:$port"
} > "$tmp/got"
cat > "$tmp/want" <<'WANT'
proxied:
200 GET http://example.test/a?x=1 host=example.test auth=Basic dXNlcjpwdw==
lowercase:
200 GET http://example.test/a?x=1 host=example.test auth=-
no_proxy:
not_found: can't find host `example.test`
connect:
protocol: the proxy refused the tunnel: HTTP/1.1 403 Forbidden
WANT
diff "$tmp/want" "$tmp/got" && echo "proxy-check: same"
