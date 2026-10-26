# 0915. The HTTP client resends any method once when a pooled connection dies before a response byte

- **Status:** Accepted
- **Date:** 2026-10-26
- **Decided by:** the designer ("follow current browser standards", 2026-10-26)
- **Topic:** 17 — Standard library (`std.net`)
- **Spec rules:** std.net.19 (amended)
- **Builds on:** [ADR 0913](0913-http2-client-over-alpn.md)

## Context

The client keeps idle connections in a pool (std.net.19). A server may close one after the client last looked, so the next request can be written to a dead socket and fail with a reset or end of file before any response byte. The client resent only `GET`, `HEAD`, `PUT`, `DELETE` and `OPTIONS` in that case, so a `POST` failed with `closed` or `the connection closed before a response`. The gate's loopback test server showed it in 1 run in 3 (fixed there by `Connection: close`, #929).

## Decision

When a request on a **reused** pooled connection fails with a reset or end of file **before any response byte**, the client sends it again once on a new connection, **whatever its method**. This is what Chromium and Firefox do. It never resends after any response byte arrived, never on a connection it just opened, and never more than once. The body is a string held by the client, so it can be replayed. HTTP/2 keeps its rule (a reused connection that fails or answers GOAWAY for a stream it did not process) for every method too.

## Consequences

- A server that processed a `POST` and died before replying can see it twice. Browsers accept this; applications that need exactly-once send an idempotency key.
- No API change. Test: `tests/net/client.ail` ends with a request on a pooled socket that the test server closes under it (`/linger`).

## Rejected alternatives

- **Keep the method list.** A stale pooled socket fails `POST` on any real server that closes idle connections; callers cannot tell it from a real failure.
- **Retry on any failure, or more than once.** It duplicates side effects after a reply started and hides real outages.
- **Retry only with an idempotency key.** Browsers do not require one, and the designer asked for the browser behaviour.
