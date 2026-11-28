# 1285. `std.net.http2`: an HTTP/2 server

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 13 — Standard library
- **Spec rules:** [std.net.http2.1–14](../spec/std-net-http2.md)
- **Extends:** [ADR 0913](0913-http2-client-over-alpn.md) (the HTTP/2 client), [ADR 0419](0419-http-server-as-accept-and-respond.md) (the
  handler form), [ADR 1222](1222-std-net-websocket.md) (a protocol module in AIL over `Conn`)

## Context

The native client speaks HTTP/2 (ADR 0913) but every server in `std.net` is HTTP/1.1. Browsers and gRPC-style
clients prefer `h2`, and the TLS 1.3 server being added alongside (ADR 1237, `net.listen_tls_alpn`, std.net.46-49)
negotiates it by ALPN, so the std needs something to speak on the connection it hands over.

## Decision

- A module written in AIL, `std/net/http2.ail`, used with `use std.net.http2` like `std.net.websocket`. It needs
  nothing new in the compiler or runtime: `Conn.read_bytes` and `write_bytes`, `bytes` reads, tasks.
- The handler is the one `std.net.http` takes, `fn(net.Request) net.Response`, so a program moves between HTTP/1.1
  and HTTP/2 by changing one call. `serve_conn(conn, handler)` serves one open connection (h2c with prior knowledge,
  or TLS whose ALPN chose `h2`); `run(l, handler)` accepts on a `net.Listener` and serves each connection in a task of its own; `serve(addr,
  handler)` listens for h2c.
- Multiplexing in rounds: when every whole frame received has been handled, the complete requests are answered in
  the connection's task, and their bodies go out interleaved within the flow-control windows, buffered into one
  write before each read (no Nagle stalls). Handler tasks per request were tried and starve the pool: a task blocked
  on a connection holds a pool thread, and with many connections no thread is left to run the handlers. One
  task owns the connection, as `std.net.12` requires of a `Conn`; while every window is shut it reads frames.
- HPACK is a decoder with the full RFC 7541 (dynamic table, Huffman) and an encoder that never indexes and never
  Huffman-codes, which keeps the encoder stateless and every block readable by any decoder; the cost is a few bytes
  a response. Both are public (`decoder`, `Hpack.decode`, `encode`) for tests and for clients over `Conn`.
- The static table and Huffman code are the client's (`compiler/rt/native/net_h2.ail`), copied as data: the client
  is runtime code over raw memory that AIL code cannot call, and the client must stay in the runtime for
  `net.get`. Both copies are checked by RFC 7541 appendix C vectors (`net_http2_hpack`).
- A stream API for gRPC and other streamed bodies: `serve_streams(conn, fn(Stream))` gives each stream to the
  handler when its headers arrive, with `read`, `respond`, `write`, `finish(trailers)`, `reset` and `window`.
  Streams are served one at a time in the connection's task, the `Stream` holding a `ref` to the connection
  state (`own.ref.1`, never shared across tasks), so the one-task rule of `Conn` holds and the handler blocks
  on the network like any other code.
- Errors are values: `H2Error { kind H2Kind, message }`; a protocol violation sends GOAWAY or RST_STREAM with the
  RFC's code. A trapping handler traps the connection task, as in `http.run`.

## Rejected alternatives

- **Concurrent stream handlers in `serve_streams` over channels.** The connection task would have to wait on the
  socket and on the handlers' channels at once, which needs a reader task on the `Conn`. Sequential streams serve
  gRPC calls correctly; concurrency comes with a full-duplex `Conn`.
- **A reader task and a writer task per connection**, which would let a slow handler's stream not delay frames for
  the others. `std.net.12` allows one task at a time on a `Conn`, and over TLS reads and writes share the record
  state; rounds keep to the rule. Revisit when `Conn` documents a full-duplex split.
- **Extending the runtime client into a server.** It is raw-memory runtime code built for one stream per connection;
  a server in AIL is shorter, safe, and readable by agents.
- **Huffman-coding and indexing responses.** Saves bytes per response at the cost of encoder state per connection
  and a second code path to get wrong; possible later without changing the API.
- **The `Upgrade: h2c` handshake from HTTP/1.1.** Deprecated by RFC 9113 and unused by browsers; prior knowledge and
  ALPN cover the real clients.
- **Server push.** Browsers removed it; the server never sends PUSH_PROMISE.
- **Naming the module `std.net.h2`.** `h2` is a common local name (headings, hosts) that would hide the module
  (`std.use.4`); `http2` reads as what it is.

## Consequences

Tests are native programs over loopback (`tests/native/run/net_http2.ail` drives the server with a client written
over `net.Conn`; `net_http2_hpack.ail` checks the coder against RFC 7541 C.3 and C.4). The same code runs on every
target over `Conn`. Streaming bodies and trailers in responses wait for the streaming `Response` that HTTP/1.1 also
lacks. Wiring `http.serve_tls` to pick HTTP/2 by ALPN is a follow-up once ADR 1237 lands.
