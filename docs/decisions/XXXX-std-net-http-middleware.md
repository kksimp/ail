# XXXX. `std.net.http` router middleware, groups and built-ins

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for the std library to be filled out on 2026-11-26
- **Topic:** 17 — Standard library
- **Spec rules:** `http.mw.1` to `http.mw.6` in [std-net-http](../spec/std-net-http.md)
- **Extends:** [ADR 1217](1217-http-router.md)

## Context

The router has one `on_request` hook and no way to wrap handlers, so auth, logging, request ids and panic
recovery are repeated in every handler. The std.net.web lane found this gap.

## Decision

- `r.use(mw)`; `mw` is `fn(http.Req, http.Next) net.Response`. `Next` holds the remaining middleware and the
  handler, and `next.run(req)` runs them, so one function is both the before and the after hook, and can
  short-circuit. Router middleware wrap every matching route and the `404` and `405` answers, not static files.
- `r.group(prefix, sub)` copies the routes and mounts of a router built on its own, with the prefix, and bakes
  in the middleware of `sub` (inside the parent's).
- Built-ins as plain functions: `recover` (the chain runs as a task; a trap is a `500`), `request_id`,
  `log_requests`. `r.gzip()` is a router flag applied last, in the send path.
- All in `std/net/http.ail`; no checker, lowering or runtime change.

## Rejected alternatives

- **`next` as a closure argument `fn(Req) net.Response`.** Needs closures built per request in a loop; a
  `Next` struct with a `run` method needs none and reads the same.
- **A `Group` handle returned by `r.group(prefix)`.** A `Router` is a value, so a handle would not write back;
  a sub-router passed in matches value semantics.
- **Separate `before` and `after` hooks.** Two functions for one concern, and no way to skip the handler.
- **Gzip as a middleware.** `net.Response.body` is `str`, which cannot hold compressed bytes; changing
  `net.Response` touches the checker and runtime for every `net` program.
- **Middleware that sees static files.** Files are sent as raw bytes outside `net.Response`; wrapping them
  needs the same body change.
