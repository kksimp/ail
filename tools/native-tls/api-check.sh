#!/usr/bin/env bash
# The public std.crypto API against Project Wycheproof (ADR 1118): tools/native-tls/wycheproof_api.ail on
# tests/data/net/crypto/wycheproof-api.txt. Unlike prim-check.sh it needs no test hook: the program is an ordinary
# AIL program that calls std.crypto. Run from anywhere: tools/native-tls/api-check.sh
set -euo pipefail
cd "$(dirname "$0")/../.."
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
./ail build tools/native-tls/wycheproof_api.ail -o "$tmp/api" > /dev/null
"$tmp/api" tests/data/net/crypto/wycheproof-api.txt | tee "$tmp/out"
grep -q ' 0 failed' "$tmp/out"
