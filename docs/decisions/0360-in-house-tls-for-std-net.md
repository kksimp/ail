# 0360. `https` in `std.net` through an in-house TLS implementation, not a crate

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** Designer
- **Topic:** standard library; 27 — Dependencies and tooling
- **Spec rules:** `std.net.1–4` (the `tls` failure kind stays until the implementation lands)
- **Resolves:** the open point of [ADR 0283](0283-std-net-http-client.md)

## Context

`std.net` speaks HTTP/1.1 over plain sockets and fails `https://` URLs
with the kind `tls` (ADR 0283). Most agent-facing services are `https`,
so the gap matters. The choice was between a permissively licensed Rust
TLS library and writing TLS in-house. Custom tooling is the default
(ADR 0009), the bootstrap depends only on the Rust standard library plus
two UI crates (ADR 0023, ADR 0177), and dependency and licence choices
are the designer's (AGENTS.md).

## Options considered

### Option A — a permissively licensed TLS crate
- **Pros:** `https` quickly; audited cryptography.
- **Cons:** a large dependency tree in the bootstrap; a licence to track;
  against the custom-tooling default.

### Option B — an in-house TLS 1.3 client (chosen)
- **Pros:** no new dependency; the same code can later move to AIL for the
  self-hosted toolchain; the capability model stays whole.
- **Cons:** cryptography written from the specifications must be tested
  against known-answer vectors and a real server before it is trusted; it
  is weeks of careful work, not hours.

## Decision

Implement TLS 1.3 (client side) in the bootstrap, in Rust, without crates:
X25519, ChaCha20-Poly1305 and AES-128-GCM, SHA-256, HKDF, X.509 certificate
parsing and chain verification against the platform's root store, and the
TLS 1.3 handshake and record layer. `net.get`/`net.post` use it for
`https://`. Until it lands, `https` keeps failing with kind `tls`.

## Rationale

The designer chose to wait for an in-house implementation over taking a
dependency, consistent with ADRs 0009 and 0023.

## Consequences

- A lane (`/mnt/project-files/lanes/112-tls-in-house.md`) builds it in
  slices, each with known-answer tests; the first slice is the primitives,
  the second the handshake against a local test server, the third
  certificate verification. `https` is enabled only when all three pass.
- Constant-time primitives and no secret-dependent branches are required;
  the fuzz harness (ADR 0269) covers the record and certificate parsers.
- TLS 1.2 is not implemented.

## Rejected alternatives

- Option A, by the designer's choice.
- Shelling out to `curl` or the platform's TLS: bypasses the `net`
  capability and ADR 0009.
