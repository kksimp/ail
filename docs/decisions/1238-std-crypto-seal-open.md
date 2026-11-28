# 1238. `std.crypto` authenticated encryption: `seal` and `open`

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** the designer (ship it public, labelled unaudited, 2026-11-26); the API by AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 17 — Standard library
- **Spec rules:** [std.crypto.9 and std.crypto.10](../spec/17-std.md#cryptography-stdcrypto)
- **Amends:** [ADR 0959](0959-std-crypto.md)

## Context

The TLS client has ChaCha20-Poly1305 and AES-256-GCM, checked against RFC, NIST, OpenSSL and Wycheproof vectors
([crypto assurance](../design/crypto-assurance.md)), but a program that wants to encrypt a file or a token has no way to
reach them. The usual first draft of AEAD use gets the nonce wrong: a fixed or counter nonce reused across runs, which
breaks both ciphers completely. Encryption in the public std is escalation criterion 4 (public, legal exposure) because
the code has had no third-party audit; the designer approved shipping it with "unaudited" in the spec and the card.

## Decision

- `crypto.seal(key, plaintext, aad) bytes` (ChaCha20-Poly1305) and `crypto.seal_aes_gcm(key, plaintext, aad) bytes`
  (AES-256-GCM). The call draws a 12-byte nonce from the system generator and prepends it; the result is nonce,
  ciphertext, tag (28 bytes of overhead). The program never sees or picks a nonce, so it cannot reuse one.
- `crypto.open(key, sealed, aad) bytes?` and `crypto.open_aes_gcm`: `none` on any mismatch or a short input, one way to
  fail, no reason given (an AEAD must not say which byte was wrong). The tag is compared in constant time before
  anything is decrypted (the primitives' `tls_ct_eq`).
- 32-byte keys only (AES-256, never AES-128); another length traps, as do plaintexts over 2^36 - 32 bytes (the GCM
  limit). Keys come from `random_bytes(32)` or `hkdf_sha256`; no `crypto.key` function, since `random_bytes(32)`
  already says it.
- `seal` has the effect `random`; `open` has none.
- The runtime is `compiler/rt/native/crypto_aead.ail` over `tls_chacha.ail` and `tls_aes.ail` (hardware AES and GHASH
  where present); the checks are `compiler/mir/lower_aead.ail`.
- Tests: `tests/cases/std_crypto_aead.ail` opens the RFC 8439 section 2.8.2 vector, the GCM specification's AES-256
  test cases 13 and 16 and two OpenSSL cross-check vectors, flips bytes of the nonce, ciphertext, tag, key and
  `aad`, and round-trips lengths around the 16- and 64-byte block edges with both ciphers.

## Rejected alternatives

- **A caller-supplied nonce** (`seal(key, nonce, plaintext, aad)`): the misuse this API exists to prevent. Test
  vectors are checked through `open` instead.
- **`open` returning `bytes ! CryptoError`**: there is one failure and its cause must stay hidden, so an error enum
  would carry nothing; `bytes?` with `else` reads as plainly.
- **One `seal` with an algorithm argument**: a string or enum argument is one more thing to get wrong and to look up;
  two names keep each call self-describing, and ChaCha20-Poly1305 is the default because it is constant time without
  hardware support.
- **XChaCha20 (24-byte nonces)**: removes the 2^32-message bound, but it is not in the TLS code, has only a draft
  specification, and would be new unaudited code. Possible later as a separate name.
- **AES-128-GCM**: no reason to offer a smaller key in new code.
