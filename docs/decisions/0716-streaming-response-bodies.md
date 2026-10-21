# 0716. Streaming response bodies: `net.open` and `net.Body`

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 21 — Standard library architecture
- **Spec rules:** [std.net.25](../spec/17-std.md), std.net.7, std.net.16
- **Builds on:** [ADR 0637](0637-http-persistent-connections-and-timeouts.md) (pool), [ADR 0416](0416-net-bodies-as-bytes.md)

## Context

`net.get` reads a body whole. An agent fetching a large download, a log
stream or a server-sent events feed needs the first bytes now and must not
hold the rest in memory. `std.net.7` deferred this until `bytes` and
incremental reads could be designed together; both exist.

## Decision

`net.open(method, url, opts) Body ! NetError` runs the request exactly as
`net.request` does (options, redirects, cookies headers, pool) up to the
end of the response head, and returns a `net.Body` with the status and
headers. `b.read_bytes(max)` returns the next piece, empty at the end;
`b.close()` ends early. The client reads framing incrementally: a length
countdown, chunk by chunk, or until the connection ends. When the body
is read to its end on a connection that can carry another request, the
connection goes back to the pool; a body closed early closes its connection.
The whole-body calls are `open` plus a read to the end, so there is one
reader for both.

## Rejected alternatives

- **A callback per piece.** Closures as handlers are open (OQ-121).
- **`Body` as a `Conn`.** `Conn` is a raw socket and its `read` would bypass
  the chunked and length framing.
- **An iterator (`for piece in body`).** Iterating a handle that can fail
  needs language support for fallible iterators; `read_bytes` in a loop works
  today.
- **Streaming request bodies.** Requests are sent from memory; a large upload
  uses `Conn` or chunked sends of a bytes value.
