# XXXX. `std.net.fileserve`: validators, ranges and precompressed files for static serving

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for the std library to be filled out on 2026-11-26
- **Topic:** 17 — Standard library
- **Spec rules:** [http.file.1](../spec/std-net-http.md) to [http.file.7](../spec/std-net-http.md); amends [http.serve.2](../spec/std-net-http.md)
- **Extends:** [ADR 0755](0755-http-serve-dir.md), [ADR 1217](1217-std-net-http-router.md)

## Context

`serve_dir` and `Router.static` answered every request with the whole file: no `etag`, no `304`, no range
requests (so no seeking in audio or video), 21 content types, and no way to serve a file compressed ahead of time
or to set `cache-control`.

## Decision

- A new module `std.net.fileserve` holds the file answer, as `fileserve.answer`. `std.net.http` calls it, so the
  two other lanes that edit `http.ail` meet only a small change there.
- Validators are weak (`W/"size-mtime"`, both hex): no file is hashed, so an answer to a conditional request costs
  one `stat`. `if-none-match` wins over `if-modified-since` (RFC 9110 13.1.3).
- One range is served; a list of ranges is answered whole, which RFC 9110 14.2 allows. `if-range` accepts only a
  date, as an entity tag in it must be strong.
- `NAME.br` and `NAME.gz` neighbours are used when present and accepted, so a build step can compress once.
- `Options` (cache-control, index file, sidecars on or off) with `_with` forms of `serve_dir`, `run_dir` and
  `Router.static`; the old forms use the defaults.
- A path segment with a backslash or colon is `403` besides `..`, which closes Windows-style escapes.

## Alternatives rejected

- **Strong tags from a hash of the content:** reads every file on each request. Rejected for speed.
- **`multipart/byteranges`:** rarely sent by clients. Left out; the whole body is the allowed answer.
- **Compressing on the fly:** CPU for every request. Rejected; sidecars give the same bytes for free.
- **Editing `http.ail` in place:** three lanes would conflict on one file.
- **Following symbolic links never:** breaks the documented behaviour (`http.serve.2`) of deployments that link
  asset directories.
