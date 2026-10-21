# 0421. Raw TCP: `Listener` and `Conn`, text first, TLS over the same `Conn`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** standard library
- **Spec rules:** [std.net.12–13](../spec/17-std.md#network-stdnet)
- **Criterion that decided:** AI understanding (no ambiguity, good diagnostics), then tokens; speed is a tie

## Context

Agents need to speak protocols that aren't HTTP (line protocols, a
database wire format, a language server). `bytes` (lane 123) isn't merged.

## Options considered

### Option A — `listen`/`connect` and handles with methods, `str` I/O now, `bytes` later (chosen)
- **Pros:** the shape is final; only two methods are added when `bytes`
  lands; the line-oriented protocols agents meet first are text.
- **Cons:** binary data is lossy (U+FFFD) until `bytes`.

### Option B — wait for `bytes`
- **Pros:** one design pass.
- **Cons:** blocks this lane; TLS and the server don't depend on it.

### Option C — streams as `Channel[str]`
- **Pros:** reuses a concept.
- **Cons:** loses timeouts, close semantics and backpressure.

## Decision

Option A. `read(max)` returns whole characters (never splits one; up to
three bytes over `max`), `read_line` strips `\n` or `\r\n`, `set_timeout`
bounds idle time. `connect_tls(host, port)` returns the same `Conn` over the
in-house TLS 1.3 client (ADR 0360) with the checks of `std.net.5`; there is
no TLS server and no client certificate. A `Conn` is taken out of the table
while an operation waits, so two tasks on one `Conn` get `closed` for the
second; a `close` during a wait ends the connection when the wait does.

## Consequences

- `NetError.kind` gains `closed` and `bind`.
- Follow-ups: `read_bytes`, `write_bytes` (lane 123), UDP, half-close.

## Rejected alternatives

- Waiting for `bytes` before designing sockets (option B): rejected because it would block this lane, and TLS and the server do not depend on it.
- Streams as `Channel[str]` (option C): rejected because it loses timeouts, close semantics and backpressure.
