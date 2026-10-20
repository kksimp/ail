# 0418. Request options, redirect rules and streaming deferred for `std.net`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** standard library
- **Spec rules:** [std.net.6–7](../spec/17-std.md#network-stdnet)
- **Criterion that decided:** AI understanding (no ambiguity, good diagnostics), then tokens; speed is a tie

## Context

Before this, `std.net` had `get(url)` and `post(url, body, content_type)`:
no headers, no query, no other method, a fixed 30 s timeout and a fixed
five-redirect limit. Agents calling APIs need an `Authorization` header, a
`PUT` or `DELETE`, a query, and a say in timeouts. A request that has
options should not cost tokens for the common case.

## Options considered

### Option A — `request(method, url, opts Options)` with `options()` defaults (chosen)
- **Pros:** `get(url)` is unchanged; one function covers every method; the
  struct has no defaults problem because `options()` fills them (AIL has no
  field defaults, `type.struct.*` note); each field is named.
- **Cons:** three lines to set one header (`mut o := net.options()`,
  `o.headers[...] = ...`, the call).

### Option B — named optional parameters on `get` and `post`
- **Pros:** shortest for one header.
- **Cons:** named arguments for functions are lane 130's and not decided; an
  optional-parameter mechanism would be a language change for a library
  need.

### Option C — a builder (`net.req("GET", url).header(..).send()`)
- **Pros:** reads as a chain.
- **Cons:** more tokens than Option A for any request with two options; a
  mutable builder type for a one-shot call.

### Option D — headers as a list of pairs
- **Pros:** keeps repeated headers.
- **Cons:** the response is a `Map`; two shapes for the same thing.

## Decision

Option A. `request(method, url, opts)` and `options()` (std.net.6).
Credentials (`Authorization`, `Cookie`) are dropped when a redirect leaves
the origin (std.net.7), so a token set for one host isn't sent to another.
`max_redirects` of 0 returns the redirect (like a client's "don't follow")
instead of failing, because that is the only way to *see* a `Location`.
Header names must be tokens and values must not hold CR, LF or NUL, so a
program can't be made to inject a header by building one from input: this is
kind `bad_request`, raised before anything is sent. **Streaming bodies are
deferred.** A streaming `Response` needs `bytes` (lane 123) and a read
loop in the type; designing it before `bytes` exists would fix a shape that
`bytes` should decide. `Conn` (ADR 0421) covers the need meanwhile.

## Consequences

- `std.net.4` gains kind `bad_request`.
- The client sends `Connection: close` on every request; keep-alive and a
  connection pool are not needed for the measured agent corpus (one request
  per call) and are deferred.
- Tokens: `net.get(url)` is unchanged; a header costs three lines.

## Addendum (2026-10-20): repeated headers

A `Response` joins a repeated header's values with `, `, which loses the
boundary of a `Set-Cookie` whose `Expires` has a comma. Both response types
gain a last field `all_headers Map[str, List[str]]` with each value
separately (`std.net.17`); `net.Response(...)` may leave it out, so the
server-side `Response(status:, headers:, body:)` costs no extra tokens. A
required field would have added `all_headers: {}` to every reply. Rejected: a
method needing hidden state (no place to keep it), and a list of pairs (a
second shape next to `headers`). Asked for by the Garlic browser thread.
