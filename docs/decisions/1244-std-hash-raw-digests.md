# 1244. Raw-digest `_bytes` forms in `std.hash`

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))

## Context

`std.hash` returns lower-case hex only. UUID v5, Base58Check, JWT HMAC signing and
digest chaining need the raw bytes, and a hex round trip costs a conversion and a
fallible `bytes.from_hex`. Inputs already accept `bytes` (std.hash.1, ADR 0442).

## Decision

Each digest and HMAC function gets a `_bytes` sibling returning `bytes` (std.hash.7):
`sha256`, `sha384`, `sha512`, `sha3_256`, `sha3_512`, `sha1_legacy`, `md5_legacy`,
`hmac_sha256`, `hmac_sha384`, `hmac_sha512`. The runtime digests are shared with the hex forms
(`hash_NAME_digest`), so there is one implementation per algorithm. `crc32` already
returns an `int`. BLAKE2b is not added here (a separate algorithm, not a reuse).

## Rejected alternatives

- Changing the hex functions to return `bytes`: breaks every caller and the spec.
- A `raw: bool` argument: the result type would depend on a value, which hurts local reasoning.
- Always `.hex()`-then-`from_hex`: slow and fallible for no reason.
- A `bytes` method `.sha256()`: a new built-in method touches builtins.ail and the card coverage gate for no gain.
