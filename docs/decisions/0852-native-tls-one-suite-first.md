# 0852. Native TLS 1.3 offers `TLS_CHACHA20_POLY1305_SHA256` first, and lands in checked slices

- **Status:** Accepted
- **Date:** 2026-10-22
- **Decided by:** AI (delegated, ADR 0076; allocated by the integrator)
- **Topic:** 23 — native backend; standard library
- **Spec rules:** std.net.5, std.net.9, native.subset.1
- **Builds on:** [ADR 0836](0836-native-tls.md), [ADR 0360](0360-in-house-tls-for-std-net.md)

## Context

Native `https://` fails with kind `tls` (ADR 0836 lists four slices). Garlic
needs it to load `projects.voltare.us` on the Mac. The Rust client offers
ChaCha20-Poly1305, AES-128-GCM and AES-256-GCM. The native runtime has SHA-256,
ChaCha20-Poly1305 and X25519 (slices 1-2a); HKDF is slice 2b.

## Decision

1. **The native client first offers only `TLS_CHACHA20_POLY1305_SHA256`.** It
   needs no AES and no SHA-384, and nearly every TLS 1.3 server accepts it
   (it is the suite the Rust client prefers). A server that supports only
   AES-GCM fails with kind `tls` and the same "needs a cipher suite AIL's TLS
   lacks" message as the Rust client's `handshake_failure` case. AES-GCM
   and SHA-384 follow only when a server Garlic needs asks for them.
2. **Certificate verification is exactly that of ADRs 0368-0369, and ships in
   the same pull request as the handshake.** No slice turns `https` on while
   a chain is unchecked: `http_exchange` keeps failing `https` with kind
   `tls` until the verifier is in.
3. Slices after HKDF and the key schedule (this ADR's first pull request):
   record layer; big numbers with RSA and ECDSA P-256/P-384 verification;
   DER and X.509 with the system roots and `SSL_CERT_FILE`; the handshake and
   `https` in `net.get`, `net.post`, `net.request`.
4. The test harness fixes `tools/native-tls/prim-check.sh` to hook
   `hash_hmac_sha256` in its temporary copy of the runtime, since
   `hash.hmac_sha256` is a real module function now (it had silently run the
   real HMAC and failed every vector).

5. **Landed (slice 4):** `tls_client.ail` is the record layer, the handshake
   and the post-handshake messages, wired into `net_fill`/`net_write` through
   the socket object's `NET_TLS` word. Known gaps against the Rust client:
   a HelloRetryRequest fails with a clear message; no `close_notify` on
   close; the roots are parsed on every handshake (a cache needs a global
   slot); macOS reads `/etc/ssl/cert.pem` and does not run `security`.

## Rejected alternatives

- **All three suites at once:** about 400 lines of AES-GCM and SHA-384 for
  servers that do not exist on our path; deferred, not refused.
- **An unverified handshake first, verification later:** a working `https`
  that checks nothing is worse than a clear failure.

## Evidence

`tools/native-tls/https-check.sh`: a live `openssl s_server` with a throwaway
CA, EC and RSA leaves, an IP SAN, a wrong host, an unknown CA and a bad
`SSL_CERT_FILE`; the native client and `ail run` print the same on x86-64 and
aarch64 under qemu.

`prim-check.sh`: 97 vectors (SHA-256, HMAC-SHA-256, HKDF, ChaCha20-Poly1305,
Poly1305, X25519 and the RFC 8448 early/derived secrets) pass natively on
x86-64 and aarch64 under qemu.
