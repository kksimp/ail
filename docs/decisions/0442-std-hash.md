# 0442. `std.hash`: digests over `str` now, `bytes` when it lands

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 21 — Standard library architecture
- **Spec rules:** [std.hash.1–2](../spec/17-std.md#hashing-stdhash)

## Context

Lane 128 exposes the in-house SHA-2 and HMAC of ADR 0366 to programs. The `bytes` type (lane 123) is not merged, so binary input cannot be named yet.

## Options considered

### Option A — `str` in, hex `str` out; add `bytes` overloads later (chosen)
- **Pros:** usable today, one obvious call (`hash.sha256(s)`), hex is text an agent can print and compare; adding a `bytes` parameter later accepts every old call.
- **Cons:** cannot hash arbitrary binary until `bytes` lands.

### Option B — wait for `bytes`
- **Cons:** blocks the lane and the drag-race programs that hash text.

### Option C — digests as `List[int]`
- **Cons:** 32 to 64 list elements to print or compare; hex costs fewer tokens and is the form every tool prints.

## Decision

Option A. `sha256`, `sha384`, `sha512` and `hmac_sha256` take `str` and return lower-case hex, with no effects. They call the constant-time-where-it-matters primitives of `net::crypto` unchanged. Deciding criteria: token usage (one call, printable result), then AI understanding. No `md5` or `sha1`: they are broken for security and a `crc32` or `hash64` covers checksums.

## Update (same day, after lane 123 merged `bytes`)

Each data argument now accepts `bytes` or `str` (checker rule `std.hash.1`); digests stay hex `str`.

## Consequences

- When `bytes` merges, the parameters widen to `str | bytes` and a `digest` variant returning `bytes` can be added; no call changes meaning.
- SHA-256 runs at about 160 MB/s in software (SHA-NI and ARMv8 SHA2 instructions are a follow-up).

## Rejected alternatives

- Waiting for `bytes` before hashing (option B): rejected because it would block the lane and the drag-race programs that hash text. The `bytes` overloads were added later.
- Digests as `List[int]` (option C): rejected because 32 to 64 list elements cost more to print or compare than hex, which is the form every tool prints.
