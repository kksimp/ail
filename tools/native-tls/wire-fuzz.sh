#!/usr/bin/env bash
# The native `https` client against a server that sends mutated copies of a recorded
# TLS 1.3 server flight (record layer, ServerHello, then garbage under unknown keys).
# Every connection must end in an error; none may succeed, crash or hang.
#   tools/native-tls/wire-fuzz.sh [N [SEED]]     N connections (default 100) from SEED (default 1)
# Needs python3 3.9 or later (random.randbytes) and the native CLI (./ail).
set -euo pipefail
cd "$(dirname "$0")/../.."
n="${1:-100}"
seed="${2:-1}"
port=$((20000 + ($$ * 7 + 4219) % 10000))
python3 -I tools/native-tls/wire-fuzz.py "$port" "$seed" tests/data/net/tls/illustrated-tls13.txt </dev/null >/dev/null 2>&1 &
srv=$!
trap 'kill "$srv" 2>/dev/null || true' EXIT
for _ in $(seq 1 100); do
  python3 -I -c "import socket; socket.create_connection(('127.0.0.1', $port), 1).close()" 2>/dev/null && break
  sleep 0.1
done
urls=()
for i in $(seq 1 "$n"); do urls+=("https://127.0.0.1:$port/$i"); done
set +e
out=$(perl -e 'alarm 600; exec @ARGV' ./ail run tests/net/https.ail "${urls[@]}" 2>&1)
rc=$?
set -e
if [ "$rc" != 0 ]; then
  echo "FAIL: the client exited $rc"
  printf '%s\n' "$out" | tail -5
  exit 1
fi
got=$(printf '%s\n' "$out" | wc -l | tr -d ' ')
if [ "$got" != "$n" ]; then
  echo "FAIL: $got result lines for $n connections"
  printf '%s\n' "$out" | tail -5
  exit 1
fi
if printf '%s\n' "$out" | grep -E '^[0-9]+ '; then
  echo "FAIL: a mutated flight was accepted"
  exit 1
fi
echo "wire fuzz: $n mutated flights rejected (seed $seed)"
