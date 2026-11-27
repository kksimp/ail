# 1242. `std.uuid`, `std.base32` and `std.base58`

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for a fuller std on 2026-11-26
- **Topic:** 17 — Standard library; AI understanding
- **Spec rules:** std.uuid.1 to std.uuid.10 in [std-uuid.md](../spec/std-uuid.md), std.base32.1 to std.base32.6 in [std-base32.md](../spec/std-base32.md), std.base58.1 to std.base58.4 in [std-base58.md](../spec/std-base58.md)
- **Extends:** [ADR 1126](1126-std-modules-in-ail-without-use.md)
- **Number:** the integrator assigns it and renames this file (std ADR numbers are the integrator's)

## Context

Agents asked for ids and compact text encodings (database keys, TOTP secrets, short ids, addresses) write them by hand:
a version 4 id from `random_bytes` with the version and variant bits missed, or a Base32 decoder that accepts bad
padding. The std plan (`std-gap-plan.md`, lane 21) lists UUIDs and Base32 and Base58 as one small lane. Standard Base64
and hex already exist on `bytes`.

## Decision

Three modules written in AIL, each in scope without `use` (std.use.4); no syntax, checker or runtime change.

- `std.uuid`: a `Uuid` of 16 bytes with `v4()`, `v7()`, `v7_at(ms)`, `zero()`, `parse`, `is_valid`, `from_bytes` and
  the methods `to_bytes`, `version`, `is_zero`, `simple`, `unix_ms`. It implements `Ord` (by the bytes, so version 7
  ids sort by time) and `ToStr` (the 36-character text). Errors are `UuidError` with an enum kind (type.enum.7).
- `zero()` is the RFC 9562 nil UUID; `nil` is a foreign-habit name the checker flags (foreign.ail).
- `std.base32`: `encode`/`decode` (RFC 4648, padded, decoding accepts either case and optional padding),
  `encode_hex`/`decode_hex` (RFC 4648 section 7) and `encode_crockford`/`decode_crockford` (no padding; `-` ignored; `O`
  as 0, `I` and `L` as 1). Decoding is strict about spare bits, so every byte string has one accepted text.
- `std.base58`: `encode`/`decode` in the Bitcoin alphabet, leading zero bytes as leading `1`.

Module names `uuid`, `base32` and `base58` clash with no local name in the corpus.

## Rejected alternatives

- **`b.base32()` and `b.base58()` methods on `bytes`.** Each needs checker, lowering and card changes in shared files
  and the builtins lane; a module costs one registration.
- **One module `std.encoding` with all three.** `base32.decode` reads better than `encoding.base32_decode`, and the
  names stay searchable by the word an agent types.
- **A monotonic counter in `v7`.** Needs shared state between tasks; the random bits give millisecond order, which is
  what database keys need.
- **UUID as a `str`.** A mistyped id would pass; a `Uuid` is checked once at `parse`.
- **UUID versions 1, 3, 5, 6, 8.** v1 and v6 embed a node id; v3 and v5 need a bytes digest and `std.hash` takes text.
  Follow-up once a bytes `sha1` exists.
- **Base58Check and other alphabets.** Same dependency on a bytes digest; the Bitcoin alphabet is the common one.
