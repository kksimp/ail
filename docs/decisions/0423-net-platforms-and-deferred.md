# 0423. `std.net` on Windows and what is deferred

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** standard library
- **Spec rules:** [std.net.15](../spec/17-std.md#network-stdnet)
- **Criterion that decided:** AI understanding (no ambiguity, good diagnostics), then tokens; speed is a tie

## Context

Linux x86-64 and macOS ARM64 run in lockstep; Windows is
Winsock-aware but untested (ADR 0004).

## Options considered

### Option A — use only the Rust standard library's sockets and mark Windows untested (chosen)
- **Pros:** no `unsafe`, no `cfg` for sockets; Winsock is what Rust's
  `std::net` uses on Windows, with `set_nonblocking` and `peer_addr`.
- **Cons:** untested there until lane 153 or the designer runs it.

### Option B — a hand-written Winsock layer
- **Pros:** control.
- **Cons:** a layer to write and test for no gain now.

## Decision

Option A. Differences noted for Windows: `WouldBlock` is returned the
same way; addresses may differ in how a wildcard binds. Deferred, with
reasons in ADRs 0418–0422: streaming bodies (need `bytes`), keep-alive,
HTTP/2, server TLS, UDP, a readiness API, `net.serve(addr, handler)`.

## Consequences

- Lane 153 checks the Windows notes; nothing here is a stub that
  returns an error.
