#!/usr/bin/env bash
# Native std.net.2/3 (the HTTP client) against the interpreter, over a scripted
# local server (server.py): `ail run` and the native executables must print the
# same. Linux x86-64 always, and Linux ARM64 under qemu when it is installed.
# Until native tasks exist (a client and a server cannot share a program), this
# is the check for net.get, net.post and net.request. Run from anywhere: tools/native-net/check.sh
set -euo pipefail
cd "$(dirname "$0")/../.."
tmp=$(mktemp -d)
trap 'kill "$srv" 2>/dev/null || true; rm -rf "$tmp"' EXIT
python3 tools/native-net/server.py > "$tmp/port" &
srv=$!
while [ ! -s "$tmp/port" ]; do sleep 0.1; done
url="http://127.0.0.1:$(head -1 "$tmp/port")"
qemu=$(command -v qemu-aarch64-static || command -v qemu-aarch64 || true)
for prog in tools/native-net/http_client.ail tools/native-net/http_request.ail tools/native-net/http_bytes.ail tools/native-net/http_jar.ail; do
  ./ail run "$prog" "$url" > "$tmp/want" 2>&1
  ./ail build "$prog" -o "$tmp/x64" > /dev/null
  "$tmp/x64" "$url" > "$tmp/got" 2>&1
  diff "$tmp/want" "$tmp/got" && echo "$prog x86-64: same"
  if [ -n "$qemu" ]; then
    ./ail build "$prog" --target aarch64-linux -o "$tmp/a64" > /dev/null
    "$qemu" "$tmp/a64" "$url" > "$tmp/got64" 2>&1
    diff "$tmp/want" "$tmp/got64" && echo "$prog arm64: same"
  fi
done
