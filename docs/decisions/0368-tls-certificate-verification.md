# 0368. TLS certificate verification: X.509 chains to the platform's roots

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076), under the designer's [ADR 0360](0360-in-house-tls-for-std-net.md)
- **Topic:** standard library; 27 — Dependencies and tooling
- **Spec rules:** none yet (internal to `std.net`; `std.net.4`'s `tls` kind is unchanged until slice 4)

## Context

Slice 3 of ADR 0360. The handshake of [ADR 0367](0367-tls13-client-handshake.md)
hands the server's certificates and its CertificateVerify signature to a
`Verifier`. This slice is the real one: without it a TLS connection proves
nothing about who is on the other end.

## Options considered

### Option A — a browser-style verifier: path building, web PKI rules, platform roots (chosen)
- **Pros:** accepts what the public web serves (RSA and ECDSA chains,
  cross-signed roots, servers that send the root or extra certificates);
  rejects what browsers reject; uses the roots the user's OS trusts.
- **Cons:** X.509 is large; the parts not implemented must be listed.

### Option B — a bundled root list (Mozilla's) instead of the platform store
- **Pros:** the same roots on every platform.
- **Cons:** a list to keep current in the repository, and a corporate or
  user-added root would be ignored. Rejected; `SSL_CERT_FILE` covers a
  custom bundle.

### Option C — full RFC 5280 path validation (policies, every name form)
- **Pros:** completeness.
- **Cons:** policy processing is unused on the web and browsers skip it;
  much more code to verify.

## Decision

`bootstrap/src/net/x509/` (and `crypto/bignum.rs`, `crypto/sig.rs`):

- **Signatures:** ECDSA P-256 and P-384 with SHA-256/384/512, RSA PKCS#1
  v1.5 and RSASSA-PSS (hash-length salt) with SHA-256/384/512, RSA keys of
  2048–8192 bits. CertificateVerify accepts `ecdsa_secp256r1_sha256`,
  `ecdsa_secp384r1_sha384` (the key's curve must match) and
  `rsa_pss_rsae_sha256/384/512`. Big-number arithmetic is variable time:
  verification only touches public data.
- **Parsing:** strict DER; v1–v3; the two signature algorithm fields must
  agree; an unknown critical extension rejects the certificate; policy
  extensions are accepted without processing.
- **Path building:** depth first from the leaf through the server's
  certificates to any root whose subject matches and whose key verifies,
  backtracking across alternatives; the server may send the root, extras,
  or a shuffled order; at most 8 certificates.
- **Checks:** validity dates of the leaf and intermediates (roots are trust
  anchors: name and key); intermediates must be CAs, allow certificate
  signing if keyUsage is present, allow serverAuth if extKeyUsage is
  present, and respect path length; the leaf must allow serverAuth and
  digital signatures when those extensions are present; DNS and IP name
  constraints from every CA on the path; the host against subjectAltName
  only (RFC 6125: `*` as a whole leftmost label, IP addresses against IP
  entries, no common-name fallback).
- **Roots:** `SSL_CERT_FILE` when set; otherwise the distribution bundle on
  Linux and other Unixes, the system root keychain (`security
  find-certificate`) then `/etc/ssl/cert.pem` on macOS, and the `ROOT`
  store on Windows. Loaded once per process; a root that doesn't parse is
  skipped. With none found, the error says to set `SSL_CERT_FILE`.
- **Messages** name the cause, e.g. "the certificate for `example.test`
  expired on 2026-06-01" or "the server's certificate isn't valid for
  `other.test`; it covers example.test, *.wild.test".

## Rationale

Option A matches what agents expect of `https` (it works where a browser
works, fails where a browser warns) at a size that can be tested
thoroughly: a 19-certificate test PKI covering each rule, 108 OpenSSL
signatures, live handshakes with real checks for ECDSA and RSA-PSS
servers, every root in Ubuntu's bundle parsing, and mutation tests.

## Consequences

- Slice 4 (ADR 0369) wires `WebVerifier` with `roots::system()` into
  `net.get`/`net.post`.
- Not implemented, by design: revocation (CRL, OCSP, stapling; browsers
  rely on short lifetimes and pushed lists instead), Certificate
  Transparency, policy processing, name constraints on directory names and
  email, Ed25519 and other curves, SHA-1 anywhere in a chain (such
  certificates fail with "signed with SHA-1 with RSA, which AIL's TLS
  doesn't support"), AIA fetching of missing intermediates, and the macOS
  and Windows user trust settings (only the system roots are read).
- Windows fills its ROOT store lazily, so a root the system hasn't needed
  yet may be missing; `SSL_CERT_FILE` is the fallback. The Windows path is
  written but not run in the gate (ADR 0085).

## Rejected alternatives

- Option B: stale, and ignores the user's trust decisions.
- Option C: work without a user on the web.
- Common-name fallback for the host: deprecated by RFC 6125 and removed
  from browsers; it lets a certificate claim a name its SAN doesn't list.
