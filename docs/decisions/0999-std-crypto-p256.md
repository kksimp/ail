# 0999. std.crypto: P-256 ECDH and ECDSA signing, and HMAC-SHA1

- **Status:** Accepted
- **Date:** 2026-11-09
- **Decided by:** AI (delegated, ADR 0076); in-house only is the designer's decision
- **Topic:** 17 — Standard library
- **Spec rules:** `std.crypto.5` to `std.crypto.9`, `std.hash.7`
- **Builds on:** [ADR 0959](0959-std-crypto.md), [ADR 0836](0836-native-tls.md)

## Context

DTLS 1.2 for WebRTC (ADR 1000) needs ECDHE on P-256 and an ECDSA P-256 signature
over a self-signed certificate. STUN integrity and SRTP (ADRs 0998, 1001) need
HMAC-SHA1. `compiler/rt/native/tls_sig.ail` only *verifies* ECDSA, with
variable-time Jacobian arithmetic on public data, and `tls_big.ail`'s
`mont_mul` ends with a data-dependent subtraction: both are fine for public
inputs and wrong for a private key.

## Decision

1. `hash.hmac_sha1_legacy(key str, s str) str` (hex, like its siblings), for
   protocols that mandate it (`std.hash.7`).
2. New `compiler/rt/native/p256.ail`, separate from the verify-only code:
   - field arithmetic mod p and mod n in fixed-width limbs, with a
     constant-time Montgomery multiply (branch-free conditional subtract by
     mask) and Fermat inversion with a fixed exponent;
   - point arithmetic with the complete projective formulas of Renes, Costello
     and Batina (2015) for a = -3, which have no special cases, so there is no
     branch on the point;
   - scalar multiplication with a fixed 4-bit window, scanning the whole table
     with masks for each lookup (no secret index).
3. Public API in `std.crypto` (`std.crypto.5-9`):
   - `p256_keypair() P256Key` (private scalar from system random, rejection
     sampling below n; `P256Key { private bytes, public bytes }`, public in
     uncompressed SEC1, 65 bytes);
   - `p256_ecdh(private bytes, peer_public bytes) bytes ! CryptoError` (32-byte
     x coordinate; the peer point is validated on the curve, not infinity);
   - `p256_sign(private bytes, digest bytes) bytes` (raw r||s, 64 bytes; nonce
     per RFC 6979 with HMAC-SHA256, so a failing random source cannot leak the
     key);
   - `p256_verify(public bytes, digest bytes, sig bytes) bool` (reuses the
     existing verifier);
   - `p256_der_sig(raw bytes) bytes` and `p256_raw_sig(der bytes)` convert.
4. Vectors: NIST CAVP ECDH and ECDSA SigGen P-256, RFC 6979 A.2.5, Wycheproof
   `ecdh_secp256r1` and `ecdsa_secp256r1_sha256` (invalid points, edge scalars),
   and RFC 2202 for HMAC-SHA1. They run through `tools/native-tls/` and as
   `tests/cases/std_crypto_p256.ail`. A timing check in `tools/native-tls/` runs
   `p256_sign` with keys of low and high Hamming weight and fails on a
   difference over noise.
5. Speed target for the DTLS handshake: under 20 ms per sign or ECDH on the Mac.

## Consequences

- `std.crypto` gains `CryptoError` kind `bad_point`.
- The verify-only path in `tls_sig.ail` is unchanged; ECDSA P-384 signing is not
  added.

## Rejected alternatives

- Reusing `ec_double_mul`/`mont_mul` with a masked final step: rejected, the
  Jacobian formulas have exceptional cases that branch on the point.
- A random nonce for ECDSA: rejected, one bad random call leaks the key.
- X25519 plus Ed25519 only: rejected, browsers' DTLS-SRTP stacks require P-256
  with ECDSA certificates.
- Binding a system library for crypto: rejected by ADR 0010 and ADR 0959.
