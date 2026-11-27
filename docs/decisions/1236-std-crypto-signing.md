# 1236. `std.crypto` signing and key agreement: Ed25519, ECDSA P-256, X25519

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer approved shipping public
  signing on the in-house, unaudited crypto, labelled as such, on 2026-11-27
- **Topic:** 17 — Standard library
- **Spec rules:** [std.crypto.5 to std.crypto.8](../spec/17-std.md#cryptography-stdcrypto)

## Context

`std.crypto` (ADR 0959) has a constant-time compare, system random bytes, HKDF and PBKDF2, but no way to sign, verify
a signature or agree on a key. Programs that sign tokens, check release signatures or run a TLS server (the next std
lane) need them. The TLS client (ADR 0836) already has X25519 and ECDSA verification in the runtime, for public values
only.

## Decision

- Nine built-in calls in `std.crypto`, all over `bytes`, lowered like the existing ones (`crypto.NAME` is the runtime's
  `crypto_NAME`): `ed25519_public`, `ed25519_sign`, `ed25519_verify`; `x25519_public`, `x25519`; `p256_secret`,
  `p256_public`; `ecdsa_p256_sign`, `ecdsa_p256_verify`.
- Keys are plain `bytes`: an Ed25519 or X25519 secret is any 32 random bytes (`crypto.random_bytes(32)`), so no key
  struct and no keypair call are needed. A P-256 secret has a range, so `p256_secret()` makes one.
- Signatures are the fixed-size forms (Ed25519 64 bytes, ECDSA `r || s` 64 bytes), not DER. ECDSA hashes the message
  with SHA-256 itself and uses RFC 6979 deterministic nonces, so signing needs no random source and a weak generator
  cannot leak the key.
- Failures: a secret of the wrong length or range is a program error and traps with the rule's ID. Anything from
  outside (public keys, signatures, peer keys) is never a trap: verification gives `false`, and `x25519` gives
  `none` for a wrong-length or low-order peer key (an all-zero result).
- Ed25519 verification is strict: S must be below the group order and the key must be a canonical point encoding
  (RFC 8032 5.1.7), so signatures are not malleable.
- Code: `compiler/rt/native/ed25519.ail` (over the X25519 field code of `tls_x25519.ail`),
  `compiler/rt/native/p256_sign.ail` (constant-time Montgomery arithmetic with masked final subtractions, the complete
  a = -3 addition of Renes, Costello and Batina, a fixed 256-step ladder) and `compiler/rt/native/crypto_sign.ail`
  (the calls). Verification of ECDSA reuses `ecdsa_verify` from `tls_sig.ail`. All of it is written fresh under
  Apache 2.0; TweetNaCl (public domain) and the RFCs are the algorithm sources.
- The spec and the card say the code is unaudited.
- Tests: `tests/cases/std_crypto_sign.ail`, with RFC 8032 section 7.1 tests 1 to 3, RFC 7748 sections 5.2 and 6.1,
  RFC 6979 A.2.5 (SHA-256), signatures made by another implementation, and rejection cases.

## Rejected alternatives

- **A key struct (`Ed25519Key { secret, public }`) and `ed25519_keypair()`.** Two names for one thing and a type to
  learn; the seed already is the key and `ed25519_public` derives the rest. Rejected for AI understanding.
- **DER-encoded ECDSA signatures.** Variable length and an encoding to get wrong; the TLS server converts at its one
  use. Fixed `r || s` matches WebCrypto and JOSE.
- **A failable `x25519` with an error enum.** One way to fail (an unusable peer key) is absence, as std.stats and
  `bytes.from_hex` use; `none` keeps the call a one-liner with `else`.
- **Trapping on a bad public key or signature.** Remote input would stop the program.
- **Reusing the verifier's big-number code for signing.** It branches on values (it is for public data); signing
  needs the masked versions.
- **PEM and PKCS#8 key reading here.** Left to the TLS server lane, its first user.
