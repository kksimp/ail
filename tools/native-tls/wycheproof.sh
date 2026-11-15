#!/usr/bin/env bash
# Regenerate tests/data/net/crypto/wycheproof*.txt from Project Wycheproof (C2SP/wycheproof, Apache-2.0; ADR 1118).
# Needs curl and python3 and network access. The files are fetched into a temporary directory and read only as data.
#   tools/native-tls/wycheproof.sh
set -euo pipefail
cd "$(dirname "$0")/../.."
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
base=https://raw.githubusercontent.com/C2SP/wycheproof/main/testvectors_v1
for f in aes_gcm_test chacha20_poly1305_test x25519_test hmac_sha256_test hkdf_sha256_test \
         ecdsa_secp256r1_sha256_test ecdsa_secp256r1_sha512_test ecdsa_secp384r1_sha384_test \
         rsa_signature_2048_sha256_test rsa_pss_2048_sha256_mgf1_32_test; do
  curl -fsS "$base/$f.json" -o "$tmp/$f.json"
done
python3 -I tools/native-tls/wycheproof.py "$tmp" tests/data/net/crypto
