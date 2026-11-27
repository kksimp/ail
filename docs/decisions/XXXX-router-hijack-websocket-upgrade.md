# XXXX. A router route can take over its connection: `Exchange.hijack`, `Router.upgrade`, `websocket.upgrade`

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 13 — Standard library
- **Spec rules:** [std.net.50–51](../spec/17-std.md), [http.route.6–8](../spec/std-net-http.md)
- **Extends:** [ADR 1217](1217-http-router.md) (router), [ADR 1222](1222-std-net-websocket.md) (WebSocket), [ADR 0421](0421-raw-tcp-and-tls-conn.md) (`Conn`)

## Context

An `Exchange` can answer once, with a whole `net.Response` (`std.net.11`), so a route cannot upgrade to WebSocket or
stream Server-Sent Events. `std.net.websocket` and the SSE lane therefore listen on a port of their own, and a program
that serves pages and a socket needs two ports and a reverse proxy. The std-gap plan (section 3) asks for a hook that lets
a route take the connection.

## Decision

- **Runtime (one function).** `ex.hijack() Conn ! NetError` (`std.net.50`, `rt.serve_hijack` in `net_serve.ail`, one
  line in `check/net.ail` and one in `lower_net.ail`). The exchange's socket object, already a connection object, turns
  from an exchange into a plain `Conn` that leaves the server's keep-alive table, with the default timeout. The bytes the
  client sent after the request stay buffered: `serve_try` now advances the read position past the request, which
  nothing else read. `respond` on the exchange then fails, as for an answered one. Nothing in the pool, the
  readiness set or the shutdown code changes.
- **Router.** `r.upgrade(pattern, handler fn(Req, net.Conn))` adds a `GET` route whose handler owns the connection
  (`http.route.6`). A worker finds the route (the first matching route must be an upgrade route, `http.route.7`),
  hijacks, and sends the request, connection and handler through a channel to `Router.run`, which spawns the handler
  into its group, so a long exchange holds none of the four workers. The router closes the connection when the
  handler returns. `handle` and `HEAD` give such a route `426`; `on_request` is not called for it (`http.route.8`).
- **WebSocket.** `websocket.upgrade(conn, method, path, headers)` answers the handshake of the request the router
  parsed, sharing its checks with `Server.accept` (`std.net.50`). SSE needs no more than `conn.write`.
- **permessage-deflate, receive only, opt in.** `websocket.upgrade_deflate` agrees the extension when offered
  (answering `server_no_context_takeover; client_no_context_takeover`); `recv` inflates compressed messages with
  `compress.inflate`, restoring the four cut bytes and an empty final block (RFC 7692 7.2.2), and applies `max_size` to
  the inflated size. Messages this side sends are not compressed (`std.net.51`).

## Rejected alternatives

- **A streaming `Response` (a body that is a function or channel).** It would fit `Exchange.respond`, but WebSocket
  needs the raw connection and reads after the response; one hook serves both, and SSE is a loop of `conn.write`.
- **`Router.websocket(pattern, handler fn(Req, WebSocket))`.** It makes `std.net.http` import `std.net.websocket`
  (and its hash, crypto and compress modules) in every HTTP program. A one-line `websocket.upgrade` call in the
  handler keeps the modules apart and the handler in control of the failure case.
- **Running the handler in the worker.** Four workers would be four sockets. A task per connection is what the group
  is for.
- **Also compressing what is sent.** `compress.deflate` ends its stream with a final block; RFC 7692 needs a sync-flushed
  stream without one. It needs a `deflate` that can end non-final, a `std.compress` change of its own; receive-only is
  correct (RSV1 is per message) and the offer is opt in because inflating untrusted input is a memory cost.
- **Default on for permessage-deflate.** The inflate step is bounded only after the fact (`compress.inflate` caps at
  2^30 bytes), so a client could make a small message cost a gigabyte. Opt in leaves the choice to the program.

## Consequences

- The code was written without a build (no cloud builds); the first gate run may need fix-forward edits.
- `Router.run` returns only after hijacked handlers have returned, so a handler must end when the peer does (reads time out
  after 30 s by default).
- Follow-ups: send-side compression; a bounded `inflate`; SSE (`std.net.sse`) on the router (it only needs `conn.write`).
