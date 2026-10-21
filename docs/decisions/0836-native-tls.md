# 0836. Native TLS: the in-house client ported to AIL, in slices

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076), under the designer's [ADR 0360](0360-in-house-tls-for-std-net.md)
- **Topic:** 23 — Compiler architecture; standard library
- **Spec rules:** std.hash.1-4 (native), std.net.5 (native, later slices)
- **Builds on:** [ADR 0366](0366-tls-crypto-primitives.md), [ADR 0367](0367-tls13-client-handshake.md), [ADR 0368](0368-tls-certificate-verification.md), [ADR 0369](0369-https-in-std-net.md), [ADR 0830](0830-native-raw-tcp.md)

## Context

Native `net.get` fails `https://` with kind `tls` ([ADR 0830](0830-native-raw-tcp.md)).
Three examples (`fetch`, `poller`, `digest`) need it. The interpreter has the
in-house TLS 1.3 client of ADR 0360 in Rust; the native runtime has no libc and
no crypto. The designer's constraints stand: no new dependency, no LLVM or
external backend, constant-time comparisons where the Rust client has them.

## Decision

Port the Rust client to AIL in the native runtime (`compiler/rt/native/`),
keeping its behaviour: the same suites and key schedule, and exactly the
certificate verification of ADRs 0368-0369 (hostname rules, chain building,
validity, key usage, the system roots and `SSL_CERT_FILE`). The port lands in
slices, each its own pull request, each gated alone:

| Slice | Content | Test |
|---|---|---|
| 1 | SHA-256/384/512 and `hash_sha256_raw`-style functions over raw memory: already in `compiler/rt/native/hash.ail` (ADR 0614, lane 174, PR #208); this port reuses them and adds `hmac_sha256_raw` if missing | native `std.hash` tests of ADR 0614 |
| 2 | HKDF and `HKDF-Expand-Label`, ChaCha20-Poly1305, AES-GCM, X25519 | the RFC vectors and `crosscheck.txt` of ADR 0366, run natively |
| 3 | Big numbers, RSA and ECDSA verification, DER and X.509 parsing, the verifier | the 19-certificate test PKI and the 108 signatures of ADR 0368, run natively |
| 4 | Record layer and handshake; `https` in `net.get`, `net.post`, `net.request` | the recorded handshakes of ADR 0367 replayed against the Rust and the AIL client; a live `openssl s_server` run |

Every slice that has a Rust twin is tested against it on the same recorded
inputs. A comparison of secrets (tags, `Finished`) uses a constant-time
equality wherever the Rust code does (`ct::eq`); no branch or index depends on
secret data. Where a secret-independent shortcut would be faster, it is not
taken.

## Testing the primitives

The runtime functions have no public `std` face, so a slice needs a way to run
its vectors natively. The hook is test-only and is **not public API**:

- A `tlsprim` lowering module exposes the raw functions on hex `str`
  arguments. It is enabled only by a hidden compiler flag
  (`--internal-modules`) that the test harness passes, never by where a file
  lives, so behaviour does not depend on paths. It never appears in `ail api`,
  the cards, the quick reference or `ail explain`.
- Slice 2a (ChaCha20-Poly1305, X25519) needs no product change at all:
  `tools/native-tls/prim-check.sh` builds in a temporary copy of `compiler/`
  with a test-only runtime file and the 48 vectors of ADR 0366's
  `crosscheck.txt`. The flagged module replaces that overlay when a slice
  needs more than it can do.
- A public crypto API (a `std.crypto`) would be a separate design decision.

Test inputs are the vector files of ADR 0366 and ADR 0368 themselves; the
Rust client is the oracle.

## Rejected alternatives

- **Link a C or Rust TLS library into the runtime.** Breaks ADR 0010 (no
  external backend) and ADR 0360 (no new dependency).
- **Keep `https` interpreter-only.** Leaves three examples and Garlic's
  fetch path off the native backend.
- **One large pull request.** Cryptography is easiest to trust in pieces that
  each have vectors; a single change would gate for days.

## Clock caveat

The native clock of lane 177 (`time_mono_ms`) reads the wall clock on macOS
until a follow-up, so network timeouts must not assume it is monotonic there.
