# 1230. `std.net.http`: gzip responses on request

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 17 Standard library
- **Spec rules:** [http.gzip.1-3](../spec/std-net-http.md)
- **Extends:** [ADR 1217](1217-http-router.md) (the router)

## Decision

- `r.gzip()` makes a router's `run` and `serve` answer with gzip (`compress.gzip`) when the request's
  `accept-encoding` allows it; `http.accepts_gzip(header)` is that negotiation (`gzip`/`x-gzip`, `q=0` refuses, `*`
  counts when gzip is not named).
- Only text bodies (`text/*`, JSON, XML, JavaScript, WASM) of 256 bytes or more are compressed, and they carry
  `vary: accept-encoding` either way. 1xx, 204, 206, 304 and already-encoded responses are left alone.
- The `on_request` hook sees the uncompressed response; `handle`, `http.run`, `serve_dir` and `run_dir` do not compress.

## Rejected alternatives

- Compressing by default: it changes every existing response.
- Compressing in the handler form: `net.Response.body` is a `str`.
- Brotli and deflate: gzip is what every client accepts; others can follow the same switch.
