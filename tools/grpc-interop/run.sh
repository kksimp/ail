#!/usr/bin/env bash
# Interop check for std.net.grpc: an AIL gRPC server against a Python grpcio
# client (and grpcurl when installed). Run from the repository root on a Linux
# or macOS host: tools/grpc-interop/run.sh
set -euo pipefail
root=$(cd "$(dirname "$0")/../.." && pwd)
work=$(mktemp -d)
trap 'kill "${pid:-0}" 2>/dev/null || true; rm -rf "$work"' EXIT
"$root/ail" build "$root/tools/grpc-interop/server.ail" -o "$work/server"
python3 -m venv "$work/venv"
"$work/venv/bin/pip" install -q grpcio
"$work/server" > "$work/port" &
pid=$!
for _ in $(seq 1 50); do [ -s "$work/port" ] && break; sleep 0.1; done
port=$(head -n1 "$work/port")
"$work/venv/bin/python" "$root/tools/grpc-interop/client.py" "$port"
if command -v grpcurl >/dev/null; then
  grpcurl -plaintext -d '' "127.0.0.1:$port" t.S/Nope 2>&1 | grep -q Unimplemented && echo "ok   grpcurl unimplemented"
fi
