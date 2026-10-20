# 0443. `std.compress` API: `str` and `List[int]` now, one error type

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 21 — Standard library architecture
- **Spec rules:** [std.compress.1–2, 5](../spec/17-std.md#compression-stdcompress)

## Context

Compression produces arbitrary bytes. Until `bytes` (lane 123) exists, a program can hold them only as `List[int]`.

## Options considered

### Option A — `List[int]` of 0–255 for compressed data, `str` for text; file functions take paths (chosen)
- **Pros:** works today and round-trips; the file functions (`gzip_file`, `zip_dir`) cover real jobs without ever holding bytes in a program; one `CompressError { kind, message }` for everything.
- **Cons:** `List[int]` is 16 bytes per byte in the interpreter; fine for tests and small data, and the file functions avoid it for big data.

### Option B — hex or base64 `str` for compressed data
- **Cons:** an invented text encoding that must be dropped later; costs a conversion.

### Option C — wait for `bytes`
- **Cons:** blocks the lane.

## Decision

Option A. No level parameter yet (fixed 6: smallest output within about 2% of level 9 on the measured corpora, ADR 0445); a `level` argument is an additive change. When `bytes` lands, `deflate`/`gzip` return `bytes`, `inflate`/`gunzip` take `bytes`, and `List[int]` callers get a pointed diagnostic or a `bytes.from_list` fix. The deciding criterion was AI understanding (one error type, kinds that name the cause).

## Update (same day, after lane 123 merged `bytes`)

Compressed data is `bytes`: `deflate`, `gzip` and `zlib` take `bytes` or `str` and return `bytes`; `inflate`, `gunzip` and `unzlib` take and return `bytes`, so binary payloads (PDF streams, PNG) round-trip and `invalid_utf8` is gone (`.to_str()` converts). Programs that used `List[int]` change to `bytes.from_list(xs)`; nothing outside this lane's tests used it yet.

## Consequences

- Effects: byte functions none; file and directory functions `fs`, `io`.
- `inflate`/`gunzip` return `str`, so non-UTF-8 content fails with `invalid_utf8` until `bytes` exists.
