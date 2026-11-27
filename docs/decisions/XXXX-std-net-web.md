# TBD. `std.net.web`: cookies, sessions, CSRF, CORS and uploads for the HTTP server

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked on 2026-11-26 for the std library to be filled out
- **Topic:** 17 — Standard library; AI understanding
- **Spec rules:** std.net.web.1 to std.net.web.12 in [std-net-web.md](../spec/std-net-web.md)
- **Extends:** [ADR 1217](1217-http-router.md)

## Context

`http.router()` routes requests and answers them, but a service that has users also needs cookies, a session, a way to
refuse forged form posts, CORS for a browser front end, and file uploads. Agents write these by hand each time, and the
security details (cookie flags, session fixation, constant-time comparison, a boundary parser that trusts nothing) are
where hand-written versions fail. Other lanes were editing `std/net/http.ail` the same night.

## Decision

A new module in its own file, `std/net/web.ail`, written in AIL, imported with `use std.net.web` like `std.net.jar`
(no change to `http.ail`, no syntax, runtime or checker work). Functions take `http.Req` and `net.Response`.
Middleware is a function from a handler to a handler. The defaults are the safe ones: `HttpOnly`, `SameSite=Lax`, a
256-bit random session id from the operating system's generator, no session created until the handler stores
something, a 10 MiB body limit for uploads.

- Cookies: `cookies`, `set_cookie` with a `CookieOpts` struct and a `CookieError` whose kind is an enum, `clear_cookie`,
  `with_cookie`, HMAC-SHA-256 signed values bound to the cookie name.
- Sessions: a `Store` trait (`load`, `save`, `remove`), `memory_store` (16 `Mutex` shards, a size cap), a `Session`
  handle, `with_session`, `regenerate` and `destroy`.
- `csrf_token`, `csrf_valid`, `csrf_guard`, `same_origin`; `cors` and `preflight`; `multipart` with `UploadLimits`,
  `Form` and `Part`; `safe_filename`.

## Rejected alternatives

- Router middleware (`r.use(...)`): needs changes to `http.ail` and a rule for order; a wrapped handler is explicit.
- Session data inside a signed cookie: bigger requests, readable data, no way to end a session.
- Encrypting cookies: needs an AEAD and a key schedule; signing covers what a session id and a flag need.
- A `Session` that is a plain `Map` returned by the handler: the handler would have to return a pair; the handle form
  keeps `fn(Req, Session) Response`.
- Streaming uploads to disk: needs the server to hand over the connection; the server reads a whole body (16 MiB limit,
  `std.net.10`). A follow-up for a runtime that can hijack a connection.
- A random id with a MAC: the id is already unguessable.

## Consequences

The file was written without a build (the lane has no Mac or Linux session), so the first gate run may need
fix-forward edits. The module never touches the 16 MiB server limit. A persistent store (SQLite, a file) is a
user-written `Store`.
