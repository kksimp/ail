# 1233. `std.net.http` client helpers and retry

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); std-gap lane 22
- **Topic:** 17 Standard library
- **Spec rules:** http.client.1 to http.client.5 ([std-net-http](../spec/std-net-http.md))
- **Extends:** [ADR 0418](0418-request-options-and-redirect-rules.md) (net.request), [ADR 1214](1214-std-url.md) (std.url), [ADR 1217](1217-http-router.md) (router)

## Decision
- `get_json`, `post_json` over `net.request`; a non-2xx status fails with `http.HttpError` (link, status, body), bad JSON with `JsonError`.
- `post_form(link, pairs)` uses `url.build_query`; `multipart(parts, boundary: "")` returns `(bytes, content_type)`; `post_multipart` sends it. Names are quoted as browsers do (`"`, CR, LF as %22 %0D %0A).
- `retry(attempts, base_ms, call)` retries on kinds timed_out, refused, closed and on 5xx; the last answer is returned as is. Waits are `backoff_ms(base_ms, n)` (doubling, cap 30000) jittered to half..full.
- All AIL in std/net/http.ail; no checker, lowering or runtime change.

## Rejected alternatives
- Returning `Json` for any status: hides failures an agent forgets to test.
- Retrying 429 and 4xx: needs Retry-After handling; a caller wraps `call` for it.
- A `retry` that takes `get`/`post` arguments: a function value covers every method and header set.
- Retrying only on a fixed count with no jitter: synchronised clients hit a recovering server together.
- Part as an enum of Field/File: two constructors `field` and `file_part` over one struct read simpler.

## Consequences
Written without a build (cloud container); the first Mac gate is its first compile. `retry` sleeps with `time.sleep_ms`.
