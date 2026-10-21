# 0630. `is_empty` on strings and collections, and `path.stem`

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 17 — Standard library
- **Spec rules:** type.str.19, type.coll.20 ([04-types](../spec/04-types.md)), std.path.5 ([17-std](../spec/17-std.md))

## Context

Lane 192's survey: agents test emptiness constantly (`if not xs`, `if not s`
in Python) and write `xs.len() == 0` in AIL; file tools take the name
without its extension (`Path.stem`), which took three lines
(`find(".")`, `else`, `slice`).

## Decision

`is_empty()` on `str`, `List`, `Map` and `Set`; `path.stem(p)`.

## Token measurement

`xs.len() == 0` against `xs.is_empty()`: 7 vs 5 tokens (cl100k) per test.
`path.stem(p)`: 6 tokens against about 30 for the by-hand version.

## Rejected alternatives

- **Truthiness (`if xs`).** Implicit conversions are excluded by the
  language design (conditions are `bool`).
- **`path.name_without_ext`.** Longer; `stem` is the Python (`Path.stem`)
  and Rust (`file_stem`) name an agent already knows.
- **`int.pow` and `clamp`.** Math belongs to the `std.math` lane (184).
