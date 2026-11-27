# 1217. `std.net.http` router, request parameters and response helpers

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for the std library to be filled out on 2026-11-26
- **Topic:** 17 — Standard library
- **Spec rules:** [http.route.1](../spec/std-net-http.md) to [http.route.5](../spec/std-net-http.md); amends [http.serve.2](../spec/std-net-http.md)
- **Extends:** [ADR 0419](0419-http-server-as-accept-and-respond.md), [ADR 0755](0755-http-serve-dir.md)

## Context

A service on `http.serve` dispatches by hand: an `if r.path == ...` chain, its own path splitting, its own 404
and 405, and a hand-built `net.Response` with the content type for each reply. Every web task in the agent trials
wrote that code first, and the std-gap plan (lane 4) ranks a router as the most-used missing server piece.

## Decision

- `http.router()` makes a `Router`; `r.get`, `post`, `put`, `patch`, `delete`, `any` and `route(method, ...)` add
  routes, tried in the order added. Patterns are `/`-separated segments: literal, `:name` (one segment) or
  `*name` (the rest). Empty segments are dropped from patterns and paths, and captured values are `%XX`-decoded.
  A `GET` route answers `HEAD`.
- A handler is `fn(http.Req) net.Response`. `http.Req` is the fields of `net.Request` plus `params`, with
  `param(name) str?`, `json() Json ! JsonError`, `form() Map[str, str]` and `stop()`.
- No match is `404`; a matching pattern with the wrong method is `405` with `allow`. `r.static(prefix, dir)`
  serves files as `serve_dir` does when no route matches. `r.on_request(hook)` sees each answer.
- `r.handle(req)` answers without a server; `r.run(srv)` and `r.serve(addr)` use the four workers of `http.run`.
- Helpers `text`, `html`, `json`, `redirect` (302), `not_found` and `mime_type` (the `serve_dir` table).
- All of it is AIL in `std/net/http.ail`; no checker, lowering or runtime change.
- `serve_dir`'s `405` now carries `Allow: GET, HEAD`, and its extension lookup ignores case.

## Rejected alternatives

- **`req.param` on `net.Request` itself.** `net.Request` is a built-in type with checker-listed methods
  (`check/net.ail`); adding a field or method there would touch the checker and the native runtime for every
  `net` program, and a hand-made `Request` would need the new field. A separate `http.Req` keeps the change in one
  AIL file and keeps `http.serve` handlers unchanged.
- **Handlers `fn(net.Request, Map[str, str]) net.Response`.** Two arguments every handler must accept, even with
  no parameters; `req.param("id")` reads the way agents write it in other frameworks.
- **Regex or typed patterns (`{id:int}`).** More to learn and to parse for little gain; a handler converts with
  `to_int() else` and answers `400` itself.
- **Trailing-slash redirects and strict empty segments.** Agents mix `/a` and `/a/`; treating them as one path
  removes a class of 404s at no cost.
- **Several hooks or middleware chains.** One hook covers logging, the only use the trials showed; wrapping a
  handler in a closure covers the rest.
- **`redirect(status, to)`.** `302` is what a browser and `net.get` follow; a handler that needs `301` or `307`
  writes `net.Response` directly.

## Consequences

The router is a value: routes added after `run` starts are not seen by its workers. Files from `r.static` are sent
as bytes by `run`; `r.handle` gives their text. Tests are loopback only (`tests/cases/std_http_router*.ail`,
`examples/http_router.ail`).
