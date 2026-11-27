# 0717. `std.net` asks for gzip or deflate and undoes it

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 21 — Standard library architecture
- **Spec rules:** [std.net.26](../spec/17-std.md)
- **Builds on:** [ADR 0442](0442-std-hash.md) (in-house compression), [ADR 0637](0637-http-persistent-connections-and-timeouts.md)

## Context

Most servers compress when asked, and every agent HTTP library (`requests`,
`curl --compressed`, `fetch`) does the asking and the decoding. `std.net`
sent no `Accept-Encoding`, so responses were larger than needed, and Garlic
(PR 211) wrote its own request header and decoder.

## Decision

The whole-body calls add `Accept-Encoding: gzip, deflate` and decode the
final response using the in-house codecs (`bootstrap/src/codec`), limited
to 256 MiB, removing `Content-Encoding` and `Content-Length`. A program that
sets `Accept-Encoding` itself opts out and keeps the raw bytes and headers,
so Garlic's decoder keeps working unchanged. `net.open` neither asks nor
decodes.

## Rejected alternatives

- **Decoding only when `std.compress` is imported.** The module is the
  codec, not a switch; a program that imports it for other reasons would
  change behaviour.
- **Always sending the header, even for `open`.** A streamed body would
  arrive compressed with nothing to decode it.
- **Keeping `Content-Encoding` on a decoded response.** It would tell the
  reader to decode twice.
- **Brotli and zstd.** Not offered, so no server sends them.

## Amendment (2026-11-27)

With `compress.unbrotli` (ADR 1241) the header is `gzip, deflate, br` and a `br` body is decoded the same way (std.net.26). zstd is not sent until its decoder lands.
