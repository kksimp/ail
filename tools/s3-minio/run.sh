#!/usr/bin/env bash
# Runs tools/s3-minio/real.ail against a real MinIO server on 127.0.0.1:9000 (std.cloud.s3).
# Uses `minio` from PATH, or builds it with Go (`go install github.com/minio/minio@latest`) into a temp GOPATH.
# From the repository root: tools/s3-minio/run.sh. Needs Linux or macOS, and Go when MinIO is not installed.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
work="$(mktemp -d)"
pid=
cleanup() { [ -z "$pid" ] || kill "$pid" 2>/dev/null || true; rm -rf "$work"; }
trap cleanup EXIT
minio="$(command -v minio || true)"
if [ -z "$minio" ]; then
    command -v go >/dev/null || { echo "s3-minio: install minio or Go first" >&2; exit 2; }
    GOPATH="$work/gopath" go install github.com/minio/minio@latest
    minio="$work/gopath/bin/minio"
fi
mkdir -p "$work/data/bkt"
MINIO_ROOT_USER=minioadmin MINIO_ROOT_PASSWORD=minioadmin123 "$minio" server "$work/data" \
    --address 127.0.0.1:9000 --console-address 127.0.0.1:9001 >"$work/minio.log" 2>&1 &
pid=$!
for _ in $(seq 1 50); do
    curl -s -o /dev/null http://127.0.0.1:9000/minio/health/live && break
    sleep 0.2
done
out="$("$root/ail" run "$root/tools/s3-minio/real.ail" 2>&1)" || { echo "$out"; exit 1; }
echo "$out"
if echo "$out" | grep -q ": false"; then echo "s3-minio: FAILED" >&2; exit 1; fi
echo "s3-minio: all checks passed"
