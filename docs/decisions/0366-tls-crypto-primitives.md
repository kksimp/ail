# 0366. TLS cryptographic primitives: what, where, and how they are tested

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076), under the designer's [ADR 0360](0360-in-house-tls-for-std-net.md)
- **Topic:** standard library; 27 — Dependencies and tooling
- **Spec rules:** none yet (internal to `std.net`; `std.net.4`'s `tls` kind is unchanged until slice 4)

## Context

ADR 0360 chose an in-house TLS 1.3 client for `https`. Its first slice is
the primitives TLS 1.3 needs (RFC 8446 §9.1): a hash, HMAC and HKDF for the
key schedule, the two AEADs this client offers, and X25519 for the key
share. They are written from the specifications, in Rust with the standard
library only, so they must be proven correct by tests and must not leak
secrets through timing.

## Options considered

### Option A — portable constant-time code with known-answer and cross-check tests (chosen)
- **Pros:** one code path on every target; no secret-indexed tables; the
  RFC/NIST vectors plus 160 random OpenSSL-generated vectors pin every edge
  (lengths around block boundaries, Poly1305's final carries).
- **Cons:** AES-GCM is slow without hardware (about 10 MB/s on Linux
  x86-64, against about 290 MB/s for ChaCha20-Poly1305).

### Option B — table-based AES and GHASH
- **Pros:** several times faster in software.
- **Cons:** the lookups are indexed by key and data bytes, a known cache
  timing channel. Rejected by ADR 0360's constant-time requirement.

### Option C — hardware AES and carry-less multiply first (AES-NI/PCLMULQDQ, ARMv8 AES/PMULL)
- **Pros:** fast and constant time.
- **Cons:** needs a portable fallback anyway, and two architecture paths to
  verify in lockstep (ADR 0008). Deferred, not rejected (see Consequences).

## Decision

`bootstrap/src/net/crypto/` holds SHA-256, SHA-384 and SHA-512
(FIPS 180-4), HMAC (RFC 2104), HKDF with TLS 1.3's `HKDF-Expand-Label` and
`Derive-Secret` (RFC 5869, RFC 8446 §7.1), ChaCha20 and ChaCha20-Poly1305
(RFC 8439), AES-128/AES-256 encryption and AES-GCM with 96-bit nonces
(FIPS 197, SP 800-38D), and X25519 (RFC 7748). Secret data never steers a
branch or an index:

- AES's S-box is the Boyar–Peralta circuit on bit planes; MixColumns uses
  a masked `xtime`. Only encryption exists; GCM never needs decryption.
- GHASH multiplies bit by bit with masks; Poly1305 uses three 44/44/42-bit
  limbs and selects the final reduction with a mask.
- X25519 uses five 51-bit limbs, a fixed 255-step ladder with masked swaps,
  and inversion by a public exponent.
- AEAD `open` checks the tag with `ct::eq` before decrypting anything.

Each primitive is tested against the vectors of the document that defines
it and against `testdata/crosscheck.txt`. SHA-384/512 are included now
because certificate chains on the public web use them (slice 3).

## Rationale

Correctness and timing safety rank above speed for code that guards the
network (ADR 0360). One portable path keeps x86-64 and ARM64 identical
(ADR 0008); hardware paths can be added later behind the same functions
and the same tests.

## Consequences

- The handshake (slice 2, ADR 0367) builds on these functions unchanged.
- The client will offer TLS_CHACHA20_POLY1305_SHA256 first; a server that
  insists on AES-GCM gets the slower portable path until hardware AES and
  carry-less multiply land (follow-up, verified on both architectures).
- Not implemented, by design: AES decryption, other AES modes, nonces other
  than 96 bits, P-256 ECDH (X25519 is the only key share offered), and the
  TLS 1.2 primitives (CBC, SHA-1 PRF); TLS 1.2 is out of scope because
  every public server this client targets speaks TLS 1.3, and 1.2 doubles
  the protocol surface to verify (ADR 0360).
- What has and hasn't been verified is kept in
  [tls-security.md](../design/tls-security.md).

## Rejected alternatives

- Table-based AES and GHASH: secret-indexed memory access (Option B).
- Taking test vectors only from the RFCs: they miss lengths around block
  edges; the OpenSSL cross-check file covers those.
- Generating vectors at test time with Python or OpenSSL: `./ail selftest`
  must not need either; the vectors are generated once and committed.
