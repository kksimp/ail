# 0959. std.hash and std.crypto: the primitives, in-house, constant-time, vector-checked

- **Status:** Accepted
- **Date:** 2026-11-05
- **Decided by:** AI (delegated, ADR 0076); in-house only is the designer's decision
- **Topic:** 17 — Standard library
- **Spec rules:** `std.hash.5` (this slice); `std.crypto.*` follow with each slice
- **Builds on:** [ADR 0442](0442-std-hash.md), [ADR 0836](0836-native-tls.md)
- **Amended by:** [ADR 1238](1238-std-crypto-seal-open.md) (public `seal` and `open`)

## Context

AIL's TLS client already holds AES-GCM, ChaCha20-Poly1305, X25519, HKDF, ECDSA
and RSA verification in `compiler/rt/native/tls_*.ail`, but a program can only
reach SHA-2 and HMAC-SHA-256. C (OpenSSL, libsodium), C++ and Rust (ring,
RustCrypto) offer these as libraries; replacing them needs the same in std.

## Decision

1. **In-house code only** (ADR 0010, ADR 0009): no OpenSSL, no system crypto.
2. **Every secret-dependent path is constant-time:** no branch or table index
   on key or plaintext bytes; tag and MAC comparison goes through one
   `crypto.ct_eq`.
3. **Every primitive ships with published test vectors** (FIPS 180-4, RFC 4231,
   RFC 5869, RFC 8439, RFC 7748, RFC 8032, RFC 7914/8018) in `tests/cases/`,
   and a cross-check against a second implementation in `tools/native-tls/`.
4. **Slices**, each its own PR and spec rules:
   - S1a (this change): `hash.hmac_sha384`, `hash.hmac_sha512` (`std.hash.5`).
   - S1b: `hash.sha3_256/512`, and `hash.sha1_legacy` / `hash.md5_legacy` (broken; the `_legacy` suffix is the only name, so an agent can't pick them for security by accident) (`std.hash.6`, done); `hash.blake3` follows.
   - S1c: new `std.crypto` (`std.crypto.1-4`, this change): `ct_eq`, `random_bytes` (OS source, effect `random`), `hkdf_sha256`, `pbkdf2_sha256`. `hkdf_sha512`/`pbkdf2_sha512` follow, as does a faster PBKDF2 that precomputes the key pads.
   - S1d: `crypto.seal/open` for AES-256-GCM and ChaCha20-Poly1305 (key, nonce, aad), exposing the TLS code.
   - S1e: `crypto.x25519`, `crypto.ed25519_sign/verify`, then `argon2id`.
5. Data arguments follow `std.hash.1`: `bytes` or `str`. Digests and MACs of
   the existing functions stay lower-case hex `str`; `std.crypto` returns `bytes`.
6. No effects for pure functions; `random_bytes` has effect `rand`.

## Rejected alternatives

- **Link OpenSSL or libsodium:** breaks ADR 0010, static executables and Windows parity.
- **Expose only HMAC and leave the rest to packages:** a package would re-implement
  constant-time code with no shared vectors; agents would pick unsafe variants.
- **One giant module:** `std.hash` stays for values with no keys; keyed and
  random operations live in `std.crypto` so the effect rules stay simple.
