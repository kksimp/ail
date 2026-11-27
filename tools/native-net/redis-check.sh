#!/usr/bin/env bash
# std.db.redis against a real redis-server (needs `redis-server` on PATH): starts one on a free
# port with the password `pw`, runs tools/native-net/redis_live.ail, prints only failures.
# Run from anywhere: tools/native-net/redis-check.sh
set -euo pipefail
cd "$(dirname "$0")/../.."
tmp=$(mktemp -d)
trap 'kill $srv 2>/dev/null || true; rm -rf "$tmp"' EXIT
port=$((20000 + RANDOM % 20000))
redis-server --port "$port" --bind 127.0.0.1 --requirepass pw --save "" --appendonly no \
    --dir "$tmp" >"$tmp/server.log" 2>&1 &
srv=$!
for _ in $(seq 1 50); do
    (exec 3<>"/dev/tcp/127.0.0.1/$port") 2>/dev/null && break
    sleep 0.1
done
./ail build tools/native-net/redis_live.ail -o "$tmp/live"
"$tmp/live" "127.0.0.1:$port" | tee "$tmp/out"
tail -1 "$tmp/out" | grep -q 'GREEN'
