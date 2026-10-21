# 0637. HTTP/1.1 client: persistent connections, chunked requests, connect and total timeouts

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 21 — Standard library architecture
- **Spec rules:** [std.net.19](../spec/17-std.md), std.net.6, std.net.7
- **Builds on:** [ADR 0418](0418-request-options-and-redirect-rules.md) (options), [ADR 0360](0360-in-house-tls-for-std-net.md) (TLS), [ADR 0420](0420-cooperative-socket-waits.md) (cooperative waits)

## Context

`std.net` sent `Connection: close` on every request, so an agent that makes
a hundred calls to one API opens a hundred sockets, and over `https` runs a
hundred TLS handshakes. A request body could only be sent with
`Content-Length`, and a stalled server could hold a request past its
`timeout_ms` as long as bytes kept trickling in.

## Decision

1. **A connection pool** in the client (`bootstrap/src/interp/net.rs`), keyed
   by scheme, host and port: 4 idle per origin, 32 in all, 30 seconds (less
   if `Keep-Alive: timeout=` says so). A connection goes back only when the
   response was framed by length or chunks, was HTTP/1.1 without
   `Connection: close`, and nothing is buffered beyond it. An idle
   connection is probed (a non-blocking `peek`) before reuse; a request that
   still fails on a reused connection before any response byte is sent again
   on a new one if the method is idempotent (RFC 9110 9.2.2).
2. **`Connection: close`** in `opts.headers` is honoured and keeps nothing,
   so a program can opt out.
3. **`Transfer-Encoding: chunked`** in `opts.headers` sends the body in
   chunks of 8 KiB; otherwise `Content-Length`, as before. Responses were
   already read chunked, and interim `1xx` responses are now skipped.
4. **`connect_ms` and `total_ms`** in `net.Options`, both 0 by default (0
   means no separate limit). The deadline is stored with the stream, so a
   pooled connection takes the next request's limits, and `total_ms` covers
   every redirect hop. `timeout_ms` still bounds each wait.

## Measurements (Linux x86-64, 200 sequential `GET`s of a 2 byte body)

| | `Connection: close` | keep-alive |
|---|---|---|
| a local HTTP/1.1 server (Python, `TCP_NODELAY`) | 160 ms | 32 ms |

That is 0.8 ms per request against 0.16 ms. Over `https` the saving is the
handshake as well. Against the in-process AIL server both cost about 6 ms
per request, bound by the cooperative scheduler's polling, not the network.
A server without `TCP_NODELAY` that sends the head and body in separate
writes meets the client's delayed acknowledgement (40 ms per request); that
is the server's configuration and applies to a persistent connection only.

## Rejected alternatives

- **A global `net.keep_alive(false)` switch.** Per-request control already
  exists through the header, and a switch is state a library cannot see.
- **Pooling only for `https`.** The saving is smaller on plain sockets, but
  the rule is simpler to explain and test with one code path.
- **Retrying every method on a dead pooled connection.** `POST` may have been
  processed; the caller decides.
- **Streaming response bodies into a `Conn`-like handle.** Bodies are read
  whole as `bytes` (`std.net.16`); a request that needs more uses `Conn`.
  Deferred, not rejected.
