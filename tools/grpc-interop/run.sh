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
GRPC_PORT_FILE="$work/port" "$work/server" &
pid=$!
for _ in $(seq 1 50); do [ -s "$work/port" ] && break; sleep 0.1; done
port=$(head -n1 "$work/port")
"$work/venv/bin/python" "$root/tools/grpc-interop/client.py" "$port"
# The other way round: the AIL client against a grpcio server.
"$root/ail" build "$root/tools/grpc-interop/client.ail" -o "$work/client"
kill "$pid" 2>/dev/null || true
"$work/venv/bin/python" "$root/tools/grpc-interop/server.py" "$work/pyport" &
pid=$!
for _ in $(seq 1 50); do [ -s "$work/pyport" ] && break; sleep 0.1; done
"$work/client" "$(head -n1 "$work/pyport")"
if command -v grpcurl >/dev/null; then
  grpcurl -plaintext -d '' "127.0.0.1:$port" t.S/Nope 2>&1 | grep -q Unimplemented && echo "ok   grpcurl unimplemented"
fi
