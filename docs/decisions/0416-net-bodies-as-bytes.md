# 0416. `std.net` bodies as `bytes`: new functions and last fields, `body` stays `str`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 17 — Binary data
- **Spec rules:** [std.net.16](../spec/17-std.md#network-stdnet)
- **Builds on:** [ADR 0412](0412-bytes-type.md)

## Context

`net.Response.body` is a `str` with invalid UTF-8 replaced, so a download
or an upload of binary data is damaged. `net.Response`, `net.Request` and
`net.Options` are also written by programs: servers build `net.Response(...)`,
and every example, test and document does. The brief says to keep `r.body`
as `str` working and to decide the compatible shape.

## Options considered

### Option A — new `_bytes` functions and a `RawResponse`, plus last fields where programs don't construct the struct (chosen)
- **Tokens:** the common text case costs nothing more; the byte case adds
  the suffix `_bytes`, like `read_bytes` (ADR 0415).
- **AI understanding:** the type of the result says whether the body is text;
  no existing program changes meaning.

### Option B — change `Response.body` to `bytes`
- **Cons:** breaks every `r.body` use and every `net.Response(body: "...")`
  literal. Rejected by the brief.

### Option C — add `data bytes` to `Response`
- **Cons:** the server writes `net.Response(status:, headers:, body:)`; a
  required field breaks all such code, and a default field is not a language
  feature. `respond_bytes` gives the server the same power without it.

### Option D — a `text()` method on a bytes-bodied `Response`
- **Cons:** the `Response` type would have to hold bytes (Option B). Instead
  `bytes.text()` (ADR 0414) does the same on `RawResponse.data`.

## Decision

Option A. Client: `get_bytes`, `post_bytes(url, data, content_type)` and
`request_bytes(method, url, opts)` return `RawResponse { status, headers,
data }`. `Options.data` and `Request.data` are last fields, because
`options()` and the server make those values; `Options.data`, when not empty,
is the body sent. Server: `Exchange.respond_bytes(status, headers, data)`.
Raw TCP: `Conn.read_bytes(max)` and `Conn.write_bytes(data)`, which
`std.net.12` had reserved.

## Consequences

- `Response`, `Output` and every existing literal keep their shape.
- The client keeps the body as bytes inside the runtime and decodes it only
  for `Response`; a server's `Request.body` is the same bytes decoded.
- `Child` streams (`std.proc.4`) are still lines of text.

## Rejected alternatives

- Changing `Response.body` to `bytes` (option B): rejected by the brief, since it breaks every `r.body` use and every `net.Response(body: "...")` literal.
- Adding a required `data bytes` field to `Response` (option C): rejected because the server writes `net.Response(status:, headers:, body:)`, so a required field breaks all such code, and a default field is not a language feature. `respond_bytes` gives the server the same power.
- A `text()` method on a bytes-bodied `Response` (option D): rejected because it needs `Response` to hold bytes. `bytes.text()` does the same on `RawResponse.data`.
