# 1013. A TLS 1.3 server: `net.listen_tls`

- **Status:** Accepted
- **Date:** 2026-11-09
- **Decided by:** AI (delegated, ADR 0076); in-house TLS and crypto is the designer's rule ([ADR 0360](0360-in-house-tls-for-std-net.md))
- **Topic:** 17 — Standard library
- **Spec rules:** std.net.33 (std.net.13 now points to it)
- **Builds on:** [ADR 0421](0421-raw-tcp-and-tls-conn.md), [ADR 0836](0836-native-tls.md), [ADR 0959](0959-std-crypto.md)

## Context

The spec said "There is no TLS server" (std.net.13). A service that speaks
HTTPS, or any TLS protocol, needed a proxy in front of it. C (OpenSSL), Go
and Rust (rustls) all serve TLS. The native runtime already holds the record
layer, the key schedule, both AEADs, X25519 and ECDSA verification
(`compiler/rt/native/tls_*.ail`); what was missing is signing and the server
half of the handshake.

## Decision

1. **API:** `net.listen_tls(addr, cert_pem, key_pem) Listener ! NetError`
   (std.net.33). `accept` and the `Conn` are those of `listen` (std.net.12).
2. **Keys:** ECDSA P-256 only. The chain is PEM `CERTIFICATE` blocks, leaf
   first; the key is one `PRIVATE KEY` or `EC PRIVATE KEY` block, checked
   against the leaf's public key at `listen_tls`, so a mismatched pair fails
   at start-up with kind `tls` and not in a client's handshake. Encrypted,
   RSA and other-curve keys are refused with a message that says which.
   P-256 is what `openssl ecparam`, `certbot --key-type ecdsa` and most CAs
   issue, and verification of it is already in the client.
3. **Handshake:** TLS 1.3 only; `TLS_AES_128_GCM_SHA256` or
   `TLS_CHACHA20_POLY1305_SHA256` (both SHA-256, so one hash length);
   X25519; signature scheme `ecdsa_secp256r1_sha256`; no client
   certificate, resumption, early data or tickets; no HelloRetryRequest (a
   client with no X25519 share gets `handshake_failure`; browsers, curl and
   OpenSSL all send one); no ALPN reply (the connection is HTTP/1.1 or the
   program's own protocol). The server code (`tls_server.ail`) reuses the
   client's record layer, `tc_set_keys`, `tc_finished` and post-handshake
   code, and its error messages with "the client" for "the server".
4. **When the handshake runs:** on the connection's first read or write, not
   in `accept`. A stalled or hostile client then occupies the task that
   serves it and not the accept loop; a failed handshake is a `tls` error of
   that read or write, and the connection stays unusable (`TS_PEND = -1`),
   so no data is ever written in the clear.
5. **Signing** (`tls_sign.ail`): ECDSA over SHA-256 with a nonce from the OS
   generator, rejection-sampled below n. The scalar multiplication always
   runs 256 doublings and additions and selects the result with masks. The
   scalar is lengthened by n or 2n to exactly 257 bits (the usual defence
   against nonce-length timing). The limb arithmetic underneath
   (`tls_big.ail`) was written for public values and is **not constant
   time**: Montgomery multiplication ends with a data-dependent
   subtraction. This falls short of ADR 0959 rule 2. It is accepted for
   the first slice because the nonce is fresh per signature and blinded in
   length, and the follow-up is the constant-time P-256 of the std.crypto
   plan (planned ADR 0999); `tsg_sign` is the single place to switch.
6. **Recorder:** `listen_tls` has a `rec_` wrapper like `listen`.
7. **Not in this slice:** `net.serve` over TLS (HTTPS for the built-in HTTP
   server), RSA and P-384 keys, Ed25519 certificates, SNI-based certificate
   choice, client certificates, session tickets. Each is a later slice.

## Rejected alternatives

- **Wait for ADR 0999 before starting.** The signing step is one function; the
  rest of the server does not depend on how it is computed.
- **Do the handshake in `accept`.** A single slow client would stall every
  other one, and a bad client would surface as an error from `accept` that a
  plain `while true` loop does not expect.
- **Ed25519 first.** Constant-time code exists for it (the X25519 field
  routines), but no public CA issues Ed25519 server certificates, so a
  server that only does Ed25519 serves almost nobody.
- **Accept RSA keys too.** RSA signing needs constant-time modular
  exponentiation with CRT and blinding; that is a slice of its own.
- **A TLS terminator process or a C library.** Breaks ADR 0010 and ADR 0360.
