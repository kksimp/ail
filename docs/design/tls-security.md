# TLS security note

Non-normative. What the in-house TLS 1.3 client (ADR 0360) has and hasn't
been verified to do. Update it in the same change as the code.

## Status

`https` still fails with kind `tls` (`std.net.4`). Slices:

| Slice | ADR | State |
|---|---|---|
| 1. Primitives | [0366](../decisions/0366-tls-crypto-primitives.md) | Done, tested |
| 2. Record layer and handshake | [0367](../decisions/0367-tls13-client-handshake.md) | Done, tested |
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
- **Handshake, recorded:** a real TLS_AES_256_GCM_SHA384 handshake (The
  Illustrated TLS 1.3 Connection) replays with every client byte equal to
  the recording: key schedule, both directions' record protection, both
  Finished messages, tickets and application data. Every 7th byte of the
  server's flight corrupted, and truncation at each record, fails cleanly.
- **Handshake, live:** `openssl s_server` (OpenSSL 3.0) for each of the
  three suites, a server that prefers P-256, a server-requested KeyUpdate
  with data both ways afterwards, and refusals from a TLS 1.2-only and a
  P-256-only server with messages naming the cause. Skipped with a note
  where `openssl` isn't OpenSSL 1.1.1 or 3 (macOS ships LibreSSL), so on
  macOS only the recorded and RFC tests run unless OpenSSL is installed.
- **RFC 8448:** handshake and application secrets, `finished` key, traffic
  key and IV, and the first application-data record (values as transcribed
  by Mbed TLS's test suite).
- **Mutation:** 3,000 mutated ServerHello and Certificate bodies and 300
  mutated server flights per selftest (100,000 and 10,000 run once with
  `AIL_FUZZ_N=100000`) without a panic.

## Not verified

- **Timing:** constant time is by construction (no secret branches or
  indexes, reviewed by reading), not measured. The compiler could in
  principle reintroduce a branch; no tool checks the machine code yet.
- **Zeroising:** keys aren't wiped from memory after use.
- **Side channels beyond timing** (power, faults): out of scope.
- **HelloRetryRequest with a cookie:** implemented per RFC 8446 §4.1.4
  and unit-tested, but no available test server sends one.
- **Certificates:** slice 2 hands the chain and the CertificateVerify
  signature to a `Verifier`; only tests have one (it accepts anything).
  Until slice 3, nothing outside tests can make a TLS connection, and
  `https` stays off.

## Limits

- AES-GCM runs about 10 MB/s in software (ChaCha20-Poly1305 about
  290 MB/s; Linux x86-64, release build). Hardware AES and carry-less
  multiply are a follow-up.
- TLS 1.2, client certificates, 0-RTT, session resumption and key shares
  other than X25519 are not implemented (ADRs 0360, 0367).
