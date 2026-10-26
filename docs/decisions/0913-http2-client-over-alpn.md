# 0913. An HTTP/2 client over ALPN in the native `std.net`

- **Status:** Accepted
- **Date:** 2026-10-26
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** standard library; networking
- **Spec rules:** `std.net.31` (new)
- **Builds on:** [ADR 0369](0369-https-in-std-net.md), [ADR 0836](0836-native-tls.md)

## Context

`net.get`, `net.post` and `net.request` spoke HTTP/1.1 only, and the TLS
client offered no ALPN. Some servers (Google Search among them) answer an
HTTP/1.1 client with a challenge page where an HTTP/2 client gets the page.
Garlic, the browser written in AIL, needs HTTP/2 with a stable, browser-like
header order. Scope is the standards: no copying of another client's TLS
fingerprint, no automation masking.

## Decision

- The TLS client can offer ALPN `h2` and `http/1.1` (RFC 7301) and reports the
  server's choice. A choice that was not offered fails the handshake with kind
  `tls`; no ALPN extension in the reply means `http/1.1`.
- `net.get`, `net.post`, `net.request` (and their `_bytes` forms) offer ALPN on
  `https://` URLs on a new connection. When the server picks `h2`, the request
  goes over HTTP/2 (RFC 9113) with HPACK (RFC 7541); otherwise the HTTP/1.1
  code runs on the same connection as before. `net.open` (streaming bodies) and
  `AIL_HTTP2=0` keep HTTP/1.1.
- The response is the same `net.Response`: status, headers (names are already
  lowercase), body, redirects, `Content-Encoding` handling.
- Pseudo-headers go first, in the order `:method`, `:authority`, `:scheme`,
  `:path`. Regular headers follow in a fixed rank order (content-length,
  content-type, `sec-ch-ua*`, user-agent, accept, `sec-fetch-*`, referer,
  accept-encoding, accept-language, other headers, cookie), in the options'
  insertion order within a rank. Connection-specific headers (`connection`,
  `keep-alive`, `proxy-connection`, `transfer-encoding`, `upgrade`, `te`) and
  `content-length` set by the caller are dropped; a `host` header names the
  `:authority`.
- Flow control is honoured in both directions (connection and stream windows,
  SETTINGS changes, WINDOW_UPDATE). SETTINGS, PING, GOAWAY, RST_STREAM,
  CONTINUATION, padding and priority fields are handled; push is refused with
  `ENABLE_PUSH` 0. Failures are kind `protocol` with the HTTP/2 error name.
- One request at a time per connection, but the connection is kept: an idle HTTP/2 connection waits in the pool of std.net.19 (its own key, with its HPACK table, windows and next stream id) and the next request to the origin opens stream 3, 5, ... on it. A GOAWAY, an unread frame or an unfinished record means it is closed instead. A pooled connection the server closed, or a request it did not process (GOAWAY), is sent again on a new connection when the method may be repeated, as for HTTP/1.1.
- `AIL_NET_DEBUG` set prints a line on standard error per request naming the protocol.

Code: `compiler/rt/native/net_h2.ail`, the ALPN parts of `tls_client.ail`, and
the dispatch in `http_exchange_on`. Check: `tools/native-net/h2-check.sh`.

## Rejected alternatives

- **Copying Chrome's TLS fingerprint (GREASE, cipher order, extension order):**
  against the project's stance; the ask is HTTP/2 by the standards only.
- **Multiplexing (several streams at once):** a larger change (stream
  table, a reader task); sequential reuse of the pooled connection gets the
  handshake savings that matter (measured: a fresh handshake per request made
  repeat fetches 3 to 6 times slower than pooled HTTP/1.1).
- **Huffman-coding what we send:** saves bytes on the wire only; the decoder
  needs the table anyway, so the encoder can follow.
- **A user switch on `net.Options`:** a public API change; `AIL_HTTP2=0` is
  enough to opt out.

## Consequences

An `https://` request on a new connection now costs the server's choice of
`h2`, so servers that prefer it see HTTP/2 from every AIL program. A server
that mishandles ALPN can be reached with `AIL_HTTP2=0`.
