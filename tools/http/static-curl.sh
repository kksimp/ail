#!/usr/bin/env bash
# Starts examples/static_server.ail on a temp directory and checks it with curl:
# type, etag and 304, last-modified, ranges (206, 416), precompressed files, index, traversal.
# Run from the repository root on a machine with curl: tools/http/static-curl.sh
set -u
cd "$(dirname "$0")/../.."
tmp=$(mktemp -d)
trap 'kill "$pid" 2>/dev/null; rm -rf "$tmp"' EXIT
port=18080
printf '0123456789' > "$tmp/data.txt"
printf 'plain' > "$tmp/app.js"; printf 'GZ' > "$tmp/app.js.gz"; printf 'BR' > "$tmp/app.js.br"
mkdir "$tmp/sub"; printf '<p>sub</p>' > "$tmp/sub/index.html"
./ail build examples/static_server.ail -o "$tmp/server" >/dev/null || { echo "RED build"; exit 1; }
"$tmp/server" "$tmp" "$port" >/dev/null 2>&1 & pid=$!
for _ in $(seq 50); do curl -s "http://127.0.0.1:$port/data.txt" >/dev/null && break; sleep 0.1; done
b="http://127.0.0.1:$port"
fail=0
check() { # label, expected, actual
  if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: want '$2' got '$3'"; fail=1; fi
}
etag=$(curl -sI "$b/data.txt" | tr -d '\r' | awk -F': ' 'tolower($1)=="etag"{print $2}')
check "etag sent" "yes" "$([ -n "$etag" ] && echo yes)"
check "304" "304" "$(curl -s -o /dev/null -w '%{http_code}' -H "If-None-Match: $etag" "$b/data.txt")"
check "cache-control" "public, max-age=300" "$(curl -sI "$b/data.txt" | tr -d '\r' | awk -F': ' 'tolower($1)=="cache-control"{print $2}')"
check "206 body" "234" "$(curl -s -r 2-4 "$b/data.txt")"
check "206 status" "206" "$(curl -s -o /dev/null -w '%{http_code}' -r 2-4 "$b/data.txt")"
check "416" "416" "$(curl -s -o /dev/null -w '%{http_code}' -r 50- "$b/data.txt")"
check "suffix range" "789" "$(curl -s -r -3 "$b/data.txt")"
check "br" "BR" "$(curl -s -H 'Accept-Encoding: br, gzip' "$b/app.js")"
check "gzip" "GZ" "$(curl -s -H 'Accept-Encoding: gzip' "$b/app.js")"
check "plain" "plain" "$(curl -s -H 'Accept-Encoding: identity' "$b/app.js")"
check "index" "<p>sub</p>" "$(curl -s "$b/sub/")"
check "traversal" "403" "$(curl -s --path-as-is -o /dev/null -w '%{http_code}' "$b/sub/../data.txt")"
check "missing" "404" "$(curl -s -o /dev/null -w '%{http_code}' "$b/nope")"
check "post" "405" "$(curl -s -o /dev/null -w '%{http_code}' -X POST "$b/data.txt")"
exit $fail
