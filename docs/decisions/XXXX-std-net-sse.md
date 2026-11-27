# XXXX. `std.net.sse`: Server-Sent Events server and client

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 13 — Standard library
- **Spec rules:** [std.sse.1–10](../spec/std-sse.md)
- **Extends:** [ADR 0421](0421-raw-tcp-and-tls-conn.md) (raw `Conn`), [ADR 1222](1222-std-net-websocket.md) (the same shape)

> The number XXXX is a placeholder: the integrator assigns it, renames this file, and replaces `ADR TBD` in
> `std/net/sse.ail`, `docs/spec/std-sse.md`, `docs/status.md` and `docs/glossary.md`.

## Context

Live updates (progress of a job, a log tail, model tokens streamed by an agent) are most simply sent as
Server-Sent Events. `std.net` answers each request with one `Response`, so an agent that needs a stream has to
write the HTTP framing and the event format by hand.

## Decision

- A module in AIL, `std/net/sse.ail`, used with `use std.net.sse` (like `std.net.websocket`). It needs nothing new
  in the compiler or runtime.
- Server: its own listener, as `websocket.listen`. `accept` returns an unanswered `Request` (method, path, headers,
  `Last-Event-ID`) so the program can route and refuse before it commits to a stream; `open` answers `200` with
  `text/event-stream`, `refuse` answers an error. `Stream.send`, `note` and `pump` write events and comments;
  `pump` reads a `Channel[Event]` and sends keep-alive comments while it is idle, which also finds a client that left.
- Client: `connect(url)` and `next()`. `next` returns whole events and, when the stream ends or the connection
  fails, reconnects after the server's `retry` time with `Last-Event-ID`, up to a limit in a row (default 5).
  It reads plain and chunked bodies, and handles a byte-order mark, CRLF and comments.
- No syntax change; one error type `SseError { kind, message }` with a `SseKind` enum.

## Rejected alternatives

- **Streaming through `http.Router`.** A handler returns one `net.Response`; a response that stays open needs
  `net.Server` to hand the connection over, which is a change to `std.net` and its runtime. Left for a later ADR; a
  program runs the SSE listener on its own address (or behind a proxy that routes by path).
- **A callback client** (`on_event`). A `next` loop reads more locally with colorless tasks and needs no feature.
- **Infinite reconnecting by default.** A client that never gives up hides a dead server; a limit with
  `set_retries` is explicit. The standard's rule that only the `retry` time and `Last-Event-ID` matter is kept.
- **Chunked encoding on the server.** `Connection: close` with no length is valid HTTP/1.1 and needs no framing
  code; the cost is that a stream cannot share a keep-alive connection, which SSE clients do not need.
- **Lone CR as a line end.** The standard allows it; real servers do not send it. A CR is dropped.

## Consequences

Tests are native programs over loopback (`tests/native/run/net_sse.ail`). Windows and Linux use the same code over
`Conn`. Written without a compiler in reach: the first gate run is its first compile.
