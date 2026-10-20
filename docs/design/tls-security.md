# TLS security note

Non-normative. What the in-house TLS 1.3 client (ADR 0360) has and hasn't
been verified to do. Update it in the same change as the code.

## Status

`https` still fails with kind `tls` (`std.net.4`). Slices:

| Slice | ADR | State |
|---|---|---|
| 1. Primitives | [0366](../decisions/0366-tls-crypto-primitives.md) | Done, tested |
| 2. Record layer and handshake | 0367 | Not started |
| 3. Certificates | 0368 | Not started |
| 4. `net.get`/`net.post` over TLS | 0369 | Not started |

## Verified

- **Known answers:** SHA-256/384/512 (FIPS 180-4 examples, including the
  million-`a` message), HMAC (RFC 4231 cases 1, 2, 6), HKDF (RFC 5869
  A.1–A.3; RFC 8448 early secret), Poly1305 (RFC 8439 §2.5.2, A.3 incl. the
  carry edge cases), ChaCha20 block, cipher and AEAD (RFC 8439 §2.3.2,
  §2.4.2, §2.6.2, §2.8.2), AES-128/256 (FIPS 197 C.1, C.3, A.1 key
  expansion), AES-GCM (GCM spec cases 1–4, 13, 14), X25519 (RFC 7748 §5.2
  incl. 1,000 iterations, §6.1).
- **S-box:** the bitsliced S-box equals the S-box computed from its
  definition for all 256 inputs.
- **Cross-check:** 160 random vectors whose outputs came from OpenSSL 3.0,
  lengths 0–1000 around every block edge, for every primitive
  (`bootstrap/src/net/crypto/testdata/crosscheck.txt`).
- **Tamper:** a flipped bit in ciphertext, tag, AAD or nonce fails `open`.

## Not verified

- **Timing:** constant time is by construction (no secret branches or
  indexes, reviewed by reading), not measured. The compiler could in
  principle reintroduce a branch; no tool checks the machine code yet.
- **Zeroising:** keys aren't wiped from memory after use.
- **Side channels beyond timing** (power, faults): out of scope.
- Everything in slices 2–4: the handshake, certificate checks and the
  wiring are not written yet, so `https` stays off.

## Limits

- AES-GCM runs about 10 MB/s in software (ChaCha20-Poly1305 about
  290 MB/s; Linux x86-64, release build). Hardware AES and carry-less
  multiply are a follow-up.
- TLS 1.2, client certificates, 0-RTT and session resumption are not
  implemented (ADR 0360).
