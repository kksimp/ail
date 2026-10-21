# 0754. A handler stops its server with `req.stop()`; network waits poll faster

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library
- **Spec rules:** http.serve.1, http.serve.3 ([std-net-http](../spec/std-net-http.md))
- **Builds on:** [ADR 0419](0419-http-server-as-accept-and-respond.md), [ADR 0640](0640-samesite-and-graceful-shutdown.md)

## Context

Lane 250 asked for an HTTP/1.1 server in `std.net.http`. Most of it was
already there: `net.serve`, keep-alive, chunked request bodies, the 413, 431
and 408 limits, a read timeout and the handler form (`std.net.9`–`11`, `18`,
`22`, `24`; cases `std_net_server_limits`, `std_net_keepalive`). The gaps
were a way for a handler to end the server, static files (ADR 0755) and
speed: 100 sequential requests took 630 ms.

## Decision

1. `Request` gains a last field `server int` and a method `req.stop()`
   (`shutdown` on that server, `std.net.24`). A handler is a declared
   function with no captured state (OQ-121 is not needed: the handler is
   still a plain `fn`), so the request is the one value that reaches the
   server. A struct with a `handle` method would cost a struct, a method and
   a constructor per service; `stop()` costs one call.
2. The pause between polls of a waiting network task now grows from 50
   microseconds to 400 (was 1.6 ms). Each waiting task sleeps in turn, so
   five tasks waiting (four workers and a client) cost about 8 ms per
   round trip at the old cap. 100 requests now take 280 ms. Idle cost is up to
   2,500 wake-ups a second per waiting task.
3. Responses stay framed by `Content-Length`. Chunked responses would help
   only streaming handlers, which `Exchange` cannot express.

## Token measurement

Language card unchanged. The quick reference gains 39 tokens (`r.stop()`,
`serve_dir`, `run_dir`).

## Rejected alternatives

- **A `Stop` return value or a header marker.** Hides control flow in data.
- **A global flag.** Not in the language.
- **Waking on socket readiness.** Right for native; the interpreter's
  cooperative scheduler has no poll set yet.
- **Keeping the 1.6 ms cap.** 630 ms for 100 requests missed the 500 ms goal.
