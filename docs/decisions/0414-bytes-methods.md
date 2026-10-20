# 0414. `bytes` methods: strict `to_str`, lossy `text`, encodings on the type

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 17 — Binary data
- **Spec rules:** [type.bytes.6–13](../spec/04-types.md#bytes)


## Context

The brief asks for `to_str()` (UTF-8, fallible), `str.to_bytes()`, `from_list`,
`to_list`, `concat`, `find`, hex and base64, and a choice between a `std.encode`
module and methods on `bytes`.

## Options considered

### Option A — methods on `bytes` and associated functions (chosen)
- **Tokens:** `b.hex()` and `bytes.from_hex(s)` need no `use` line (a line
  of about 5 tokens) and no `encode.` prefix.
- **AI understanding:** one place to look: the `bytes` type lists its own
  conversions, and `ail api` shows them.

### Option B — a `std.encode` module
- **Pros:** room for more codecs (percent, base32).
- **Cons:** a `use` and a prefix on each call; the codecs here (hex, base64)
  are conversions of `bytes` and nothing else. More codecs can still land
  in a module later without moving these.

## Decision

Option A.
- `b.to_str()` is `str?` (`none` for invalid UTF-8) and `b.text()` is a `str`
  with U+FFFD for invalid sequences: the fallible and the common case are
  two names, so neither needs a `?` or an `else` to read a body that is
  known to be text. Because built-in methods win over the standard traits'
  (`candidates`), `to_str` on `bytes` is this conversion; the text form is
  `"{b}"` or `ToStr.to_str(b)`.
- `hex()` is lowercase and `from_hex` takes either case (RFC 4648 for base64:
  standard alphabet, `=` padding required, nothing else accepted). Unlike
  `from_list`, `from_hex` and `from_base64` are `bytes?`, because the text
  can be invalid.
- `find` returns an `int?` like `str.find`; an empty needle is found at 0.
- `zeros(n)` builds a buffer of known size, the one way to make `bytes` of a
  length without a list.

## Consequences

- Hashing (lane 128) takes and returns `bytes` and can use `hex()`.
- `str.to_bytes()` is the only `str` method added; `s.bytes()` (an iteration
  view) is not needed because `for x in s.to_bytes()` works.
