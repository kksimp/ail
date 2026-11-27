# ADR TBD. `std.cloud.s3`: an S3-compatible object storage client with SigV4

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for the std library to be filled out
- **Topic:** 17 — Standard library; AI understanding
- **Spec rules:** std.cloud.s3.1 to std.cloud.s3.14 in [std-cloud-s3.md](../spec/std-cloud-s3.md)
- **Extends:** [ADR 0442](0442-std-hash.md) (HMAC-SHA256), [ADR 0418](0418-request-options-and-redirect-rules.md) (the HTTP client)

## Context

Programs that store files in the cloud (backups, build caches, static sites, the seed mirror on R2) all speak the S3
protocol, which AWS, Cloudflare R2, MinIO and Ceph share. The hard part is AWS Signature Version 4: a canonical request
built from sorted, encoded parts, then four chained HMACs. Agents get the encoding and header rules wrong, and the
failure is an opaque `SignatureDoesNotMatch`. The std plan (`std-gap-plan.md`, section 3) lists it for the cloud wave.

## Decision

A new module written in AIL, `std/cloud/s3.ail`, used with `use std.cloud.s3` as `s3` (like `std.net.websocket`: a
submodule that keeps `use`). It needs only `std.net`, `std.hash`, `std.url`, `std.xml` and `std.time.date`; no runtime,
checker or syntax work.

- `s3.client(endpoint, region, key, secret)` (path style), `s3.aws(region, key, secret)` (virtual host) and
  `s3.r2(account, key, secret)` build a `Client`, a plain struct whose fields may be changed (`session_token`,
  `virtual_host`).
- `put`, `get`, `get_range`, `head`, `delete`, `list_page` (ListObjectsV2, one page) and `list` (every page).
- Multipart: `start_upload`, `upload_part`, `finish_upload`, `abort_upload`, and `put_large`, which picks one request or
  parts of at least 5 MiB and aborts on failure.
- `presign(method, bucket, key, seconds)` and `presign_at` (an explicit time, for tests): query-string SigV4 with
  `UNSIGNED-PAYLOAD`.
- The signer is public: `sign_request`, `signing_key`, `canonical_query`, `canonical_headers`, so a program can sign for
  another AWS service, and the tests check it against AWS's published vectors (the get-vanilla case of the SigV4 test
  suite, the GET Object and presigned URL examples of the S3 documentation).
- Every request signs its payload hash (`x-amz-content-sha256`), which works over `http` and `https`.
- Failures are one `S3Error` with an enum `kind` (`BadRequest`, `Auth`, `NotFound`, `Status`, `Parse`, `Net`), the HTTP
  `status` and the service's `code` (type.enum.7).
- An object is read or written whole, as `bytes`.

## Rejected alternatives

- **A built-in module in the runtime.** Per-function checker, effects and lowering work for what is string handling and
  `std.net` calls.
- **An unsigned or `UNSIGNED-PAYLOAD` default.** Not accepted over plain `http` by every service, and it weakens the
  signature for little saving.
- **Streaming bodies.** `std.net` bodies are whole values (std.net.7); a large upload uses multipart parts instead.
- **Bucket management, ACLs, encryption headers, user metadata.** Rare in programs and different per vendor; each is an
  addition later without changing this API.
- **A trait or generic backend interface over S3, GCS and Azure.** Only S3 is specified here; a shared interface would be
  guessed from one implementation.
