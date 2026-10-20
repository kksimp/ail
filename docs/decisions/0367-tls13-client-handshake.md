# 0367. The TLS 1.3 client handshake and record layer

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076), under the designer's [ADR 0360](0360-in-house-tls-for-std-net.md)
- **Topic:** standard library; 27 — Dependencies and tooling
- **Spec rules:** none yet (internal to `std.net`; `std.net.4`'s `tls` kind is unchanged until slice 4)

## Context

Slice 2 of ADR 0360: the TLS 1.3 record layer and client handshake
(RFC 8446) on top of the primitives of [ADR 0366](0366-tls-crypto-primitives.md).
It has to interoperate with real servers, give a sentence naming the cause
when it can't, and leave certificate checking to slice 3.

## Options considered

### Option A — a blocking client over any `Read + Write`, certificate check as a trait (chosen)
- **Pros:** `std.net` is blocking (ADR 0283), so `TlsStream` drops in where
  `TcpStream` was; tests drive it with a recorded byte stream; slice 3 plugs
  in a `Verifier` without touching the handshake.
- **Cons:** none for the bootstrap's needs.

### Option B — a sans-I/O state machine fed bytes by the caller
- **Pros:** reusable by an event loop.
- **Cons:** more code and states; AIL's tasks are colorless (ADR 0095), so
  nothing needs non-blocking TLS.

## Decision

`bootstrap/src/net/tls/` implements the client side of RFC 8446:

- **Offers:** cipher suites TLS_CHACHA20_POLY1305_SHA256,
  TLS_AES_128_GCM_SHA256, TLS_AES_256_GCM_SHA384 (ChaCha20 first; ADR 0366);
  key share and group X25519 only; signature schemes ECDSA P-256/SHA-256,
  ECDSA P-384/SHA-384, RSA-PSS-RSAE with SHA-256/384/512, and RSA PKCS#1
  v1.5 (for certificates); SNI for DNS names, not IP literals; middlebox
  compatibility mode (32-byte session id and a `change_cipher_spec`).
- **Handshake:** at most one HelloRetryRequest, honoured only for a cookie
  (a request for another group fails, naming it); the full key schedule
  (§7.1) with the `message_hash` transcript after a retry; CertificateVerify
  content built per §4.4.3 and handed to the `Verifier`; server Finished
  checked in constant time; an empty Certificate if the server asks for a
  client certificate.
- **Record layer:** §5 limits (2^14 plaintext, +256 protected), the inner
  content type and padding, per-record nonces, handshake messages that may
  span or share records but not a key change, and a rejected all-zero
  X25519 result (§7.4.2).
- **After the handshake:** NewSessionTicket is ignored; KeyUpdate rolls the
  read keys and, if asked, answers and rolls the write keys; close_notify
  ends the stream, as does an unannounced close (HTTP framing detects
  truncation).
- **Failures** are a `TlsError` with kind `Protocol`, `Certificate` or `Io`
  and a message naming the cause and the alert, e.g. "the server doesn't
  support TLS 1.3 (alert protocol_version); AIL supports TLS 1.3 only
  (ADR 0360)". OS randomness (`/dev/urandom`, `BCryptGenRandom`) supplies
  the random, session id and key; it never falls back.

## Rationale

The handshake is tested three ways: a recorded real handshake (The
Illustrated TLS 1.3 Connection, MIT licence) replayed so every byte the
client writes, including both Finished records and the application data,
must equal the recording; live handshakes against `openssl s_server` for
each suite and for KeyUpdate; and RFC 8448's key-schedule and record
values. Mutation tests cover the parsers and the record layer (ADR 0269).

## Consequences

- Slice 3 (ADR 0368) supplies the real `Verifier`; until then nothing can
  call `connect` outside tests, and `https` still fails with kind `tls`.
- Not implemented, by design: client certificates, PSK, 0-RTT, session
  resumption, ALPN, groups other than X25519 (a server that only accepts
  P-256 or P-384 key shares fails with `handshake_failure` and a message
  saying so), record padding on send, and TLS 1.2 (ADR 0360: one protocol
  to verify, and public servers speak 1.3).
- A HelloRetryRequest with a cookie follows RFC 8446 but no available test
  server sends one; the code path is unit-tested only (tls-security.md).

## Rejected alternatives

- Option B, sans-I/O: complexity without a user (colorless tasks).
- Offering P-256 key shares too: needs constant-time P-256 arithmetic,
  which slice 3's verification-only code isn't; revisit if real servers
  need it.
- Treating an unannounced close as an error: most HTTP servers close
  without close_notify after `Connection: close`; Content-Length and
  chunked framing already catch truncation.
