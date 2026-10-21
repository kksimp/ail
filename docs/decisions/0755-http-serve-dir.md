# 0755. `http.serve_dir` serves a directory

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library
- **Spec rules:** http.serve.2 ([std-net-http](../spec/std-net-http.md))

## Context

Agents serve a directory to a browser as often as they write a JSON endpoint
(`python -m http.server`). Without the call it is a handler, a type table and
a path check each time.

## Decision

`http.serve_dir(addr, root)` and `http.run_dir(srv, root)`, written in AIL in
`std/net/http.ail` over `respond_bytes`. They answer `GET` and `HEAD` with the
file, `index.html` for a directory, the type from the extension, `403` for a
`..` segment, `404` for a missing file and `405` for other methods. The path is
not percent-decoded (so `%2e%2e` names a file called that, not the parent) and
symbolic links are followed, as `python -m http.server` does.

Windows: `fs` and `net` are shared with Linux and macOS; nothing here is
platform-specific.

## Token measurement

`http.serve_dir("127.0.0.1:8080", ".")?` is 21 tokens (mean of five
tokenizers, `tools/tokbench`); the smallest handler that serves files, without
a type table, is 146. Language card unchanged.

## Rejected alternatives

- **A `static` option on `serve`.** An options struct for one case.
- **Decoding `%XX`.** Needs a second check for `..` after decoding.
- **Directory listings.** Covers the browsing case only; a program can walk
  and render one in a handler.
