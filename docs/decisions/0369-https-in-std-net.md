# 0369. `https` in `std.net` over the in-house TLS 1.3 client

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076), under the designer's [ADR 0360](0360-in-house-tls-for-std-net.md)
- **Topic:** standard library; 27 — Dependencies and tooling
- **Spec rules:** `std.net.2`, `std.net.4` (changed), `std.net.5` (new)

## Context

Slice 4 of ADR 0360. The primitives ([ADR 0366](0366-tls-crypto-primitives.md)),
handshake ([ADR 0367](0367-tls13-client-handshake.md)) and certificate
verification ([ADR 0368](0368-tls-certificate-verification.md)) are in
place; `net.get` and `net.post` still refuse `https` with kind `tls`
(ADR 0283).

## Options considered

### Option A — `https` always verified, roots from the platform or `SSL_CERT_FILE` (chosen)
- **Pros:** safe by default; one environment variable for custom or
  corporate roots, the convention OpenSSL, Python and Go share.
- **Cons:** no way to reach a server with a self-signed certificate
  except by adding its root to a PEM file.

### Option B — also an "insecure" switch (a parameter or flag)
- **Pros:** quick for local development servers.
- **Cons:** agents copy switches that make errors go away; a silent
  downgrade of every check. Rejected; `SSL_CERT_FILE` does the same job
  explicitly and only for the roots named.

## Decision

`https://` URLs (default port 443) go over TLS 1.3 with the
`WebVerifier` of ADR 0368 and `roots::system()`. `std.net.5` states the
guarantees. A handshake, certificate or later TLS failure is kind `tls`
with the cause in the message ("the server's certificate isn't valid for
`127.0.0.1`; it covers localhost"); a transport failure during the
handshake keeps its usual kind (`refused`, `timed_out`, `other`). The
`Host` header omits the default port for the scheme. Redirects may move
between `http` and `https`, as browsers allow. `--deterministic` and
`--deny net` are unchanged: they refuse `std.net` at check time
(`eff.det.1`, `eff.cap.1`). `examples/fetch.ail` asks an `https` URL first
and reports a failure on standard error, so its printed lines are the
same with or without a network.

## Rationale

Verification on by default is what makes `https` worth having (ADR 0360);
the error messages say exactly what to fix. Every check behind it is
tested (ADRs 0366–0368) and, end to end, an AIL program fetches from a
local `openssl s_server` over `https` with the test root as
`SSL_CERT_FILE`, and is refused with the expected messages for a wrong
host and an untrusted root.

## Consequences

- `https` works wherever the server speaks TLS 1.3 with X25519, which is
  every major public site tried; a TLS 1.2-only server fails with
  "the server doesn't support TLS 1.3 …".
- No proxy support: `std.net` connects directly and ignores
  `HTTPS_PROXY`. A follow-up can add `CONNECT` tunnelling.
- `AIL_TLS_PUBLIC=1 ./ail selftest public_sites -- --nocapture` checks a
  list of public sites; it's opt in because the gate must not need the
  network.
- AES-GCM performance (about 10 MB/s, ADR 0366) bounds downloads from
  servers that refuse ChaCha20-Poly1305; hardware AES is a follow-up.

## Rejected alternatives

- Option B, an insecure switch: see above.
- Falling back to `http` when `https` fails: a silent downgrade.
