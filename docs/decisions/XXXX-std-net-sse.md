# XXXX. `std.net.sse`: Server-Sent Events client and server

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for the std library to be filled out on 2026-11-26
- **Topic:** 17 — Standard library
- **Spec rules:** [std.net.sse.1–11](../spec/std-net-sse.md)
- **Extends:** [ADR 0421](0421-raw-tcp-and-tls-conn.md) (raw `Conn`), [ADR 1222](1222-std-net-websocket.md) (the same shape)

## Context

Live feeds (logs, progress, notifications, model token streams) are usually Server-Sent Events: a long HTTP response
of `text/event-stream`. std has no parser, no client that reconnects and no way to stream a response, because
`Exchange.respond` writes one whole response. Agents write the format by hand and get the line endings, the
`id`/`Last-Event-ID` protocol or the chunked framing wrong.

## Decision

- A module in AIL, `std/net/sse.ail`, used with `use std.net.sse`. It needs nothing new in the compiler or runtime
  (`Conn.read_bytes`, `write`, `net.listen`, `net.connect_tls`, `Channel.recv_for`).
- `Parser` (pure, byte-wise, any chunking) and `encode` are public so the format is testable and a program can serve
  or read it over any transport.
- `Client.next()` returns one event and reconnects on its own with `Last-Event-ID` and the server's `retry:` time,
  bounded by `set_max_retries`. A non-200 answer is fatal, as the standard says.
- The server is its own listener, like `std.net.websocket`: `accept` gives a `Request`, `req.stream()` answers and
  returns a `Stream` whose methods take no `mut`, so a handler `fn(Request)` and `run` (64 tasks) work. `Stream.pipe`
  feeds a stream from a `Channel[Event]` with heartbeat comments.
- Streams use `Connection: close` and no length, so no chunked encoder is needed.
- `sse.response(events)` makes a finite stream as an ordinary `net.Response` for a `Router` handler.

## Rejected alternatives

- **Streaming behind the `http.Router`.** A handler returns one whole `net.Response`; `Exchange` has no way to write
  a body in pieces. Doing it needs a runtime change (a hijack or streaming respond), which this lane cannot build or
  test. Recorded as a follow-up; the separate listener works today.
- **A callback client (`on_event`).** A `next()` loop reads locally and needs no feature (as ADR 1222).
- **Reconnecting forever by default.** A dead server would hang a program silently; five tries, then the error.
- **`mut` stream methods.** They would force `fn(mut Stream)` handlers; a `Conn` is a handle, and the state needed
  (headers sent) is fixed when `req.stream()` is called.
- **Following redirects and compression.** Rare for event streams; both can be added without changing signatures.

## Consequences

- Written without a build (cloud lane); the first compile and the golden `tests/native/run/net_sse.out` run on the
  Mac gate. A heartbeat in `pipe` detects a closed channel by `recv_for` returning in under half the interval.
- The integrator assigns the ADR number, renames this file, replaces `ADR TBD` in `std/net/sse.ail`,
  `docs/spec/std-net-sse.md`, `docs/status.md` and `docs/glossary.md`, adds the index row and regenerates the cards.
