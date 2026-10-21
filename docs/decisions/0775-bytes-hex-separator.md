# 0775. `b.hex(sep)` spaces the byte pairs

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 04 — Types (bytes)
- **Spec rules:** type.bytes.14 ([04-types](../spec/04-types.md))
- **Amends:** [ADR 0414](0414-bytes-methods.md) (`b.hex()` had no separator)

## Context

The hexdump corpus program prints 16 bytes a row as `7b 22 73 ...`. With only
`b.hex()` it builds `["{b:0>2x}" for b in row].join(" ")`. Python's `bytes.hex(" ")`
(3.8+) is one call.

## Decision

`b.hex(sep)` returns the hex digits with `sep` between the byte pairs; `hex()` is
unchanged. An empty `sep` gives the same as `hex()`; empty bytes give `""`. One
optional argument on an existing method; no new name.

## Token measurement (tools/tokbench, five-tokenizer mean)

`hexdump` (corpus program): 124.2 to **104.2** (-20.0, -16%), against its
Python twin at 117.6: from +6% to -11%. Output byte-identical to the twin.
Language card +10.2 tokens (6,964.6 to 6,974.8), a clause on the existing `hex` mention.

## Rejected alternatives

- **`hex_spaced()` or `hexdump(width)`:** a new name for one separator, and a
  fixed row width only fits one program.
- **A `sep` for `base64`:** no use seen.
