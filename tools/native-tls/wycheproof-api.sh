#!/usr/bin/env bash
# Regenerate tests/data/net/crypto/wycheproof-api.txt from Project Wycheproof (C2SP/wycheproof, Apache-2.0; ADR 1118).
# Needs curl and python3 and network access. The files are fetched into a temporary directory and read only as data.
#   tools/native-tls/wycheproof-api.sh
set -euo pipefail
cd "$(dirname "$0")/../.."
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
base=https://raw.githubusercontent.com/C2SP/wycheproof/main/testvectors_v1
for f in ed25519_test x25519_test ecdsa_secp256r1_sha256_p1363_test chacha20_poly1305_test aes_gcm_test \
         hkdf_sha256_test hmac_sha256_test; do
  curl -fsS "$base/$f.json" -o "$tmp/$f.json"
done
python3 -I tools/native-tls/wycheproof_api.py "$tmp" tests/data/net/crypto/wycheproof-api.txt
