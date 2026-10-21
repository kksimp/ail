# 0639. HTTP server: keep-alive connections

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 21 — Standard library architecture
- **Spec rules:** [std.net.22](../spec/17-std.md), std.net.11
- **Builds on:** [ADR 0419](0419-http-server-as-accept-and-respond.md), [ADR 0637](0637-http-persistent-connections-and-timeouts.md)

## Context

The server answered every request with `Connection: close`, so clients that
reuse connections (ours now does, and every browser and HTTP library does)
paid a connect per request, and ADR 0637's saving applied to external
servers only.

## Decision

`respond` keeps the connection when the request was HTTP/1.1 without
`Connection: close`, the buffer held exactly that request (so there is no
pipelined request to lose), the status is final, and the program's headers
don't ask to close. The connection goes back to the server's pending set,
marked idle, with a 15 second limit that is silent (no `408`); the first
byte of the next request switches it to the usual 10 second request limit.
`Connection: keep-alive` is sent. `HEAD`, `100-continue` and the request
limits (16 MiB body, 64 KiB head) are unchanged. `srv.close()` drops idle
connections with the server.

## Rejected alternatives

- **Pipelining.** A request already buffered behind the one being answered
  closes the connection instead; clients that pipeline retry safely and
  almost none do.
- **A streaming handler writing chunked responses.** It needs a response
  handle type the checker and card would grow by; bodies are values today
  (`std.net.16`). Deferred.
- **A graceful-shutdown method.** `srv.close()` already lets an accepted
  `Exchange` finish (`std.net.9`); only idle connections are dropped.
