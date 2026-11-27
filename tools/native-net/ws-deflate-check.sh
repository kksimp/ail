#!/usr/bin/env bash
# permessage-deflate sending (std.net.52, compress.deflate_flush): an AIL
# router echoes through `upgrade_deflate`, and the Python `websockets` client
# and `zlib` check what it sends. Needs `pip install websockets`. Run from
# anywhere: tools/native-net/ws-deflate-check.sh
set -euo pipefail
cd "$(dirname "$0")/../.."
tmp=$(mktemp -d)
trap 'kill $srv 2>/dev/null || true; rm -rf "$tmp"' EXIT
./ail build tools/native-net/ws_deflate_server.ail -o "$tmp/server"
"$tmp/server" > "$tmp/port" &
srv=$!
while [ ! -s "$tmp/port" ]; do sleep 0.1; done
python3 tools/native-net/ws_deflate_client.py "$(head -1 "$tmp/port")"
echo "ws-deflate-check: ok"
