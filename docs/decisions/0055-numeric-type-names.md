# 0055. Numeric types are named by bit width, with `int` and `float` as default aliases

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 4 — Type system
- **Spec rules:** [type.num.1–3](../spec/04-types.md)
- **Evidence:** `i64` and `int64` measured identical in all five tokenizers

## Options considered

- **`i8`…`i64`, `u8`…`u64`, `f32`, `f64`, plus `int` = `i64` and
  `float` = `f64` (chosen):** precise and compact, with the width always
  available. Con: two names for the default types.
- **Bit-width names only:** fully canonical, but everyday code always states a width.
- **`int32`, `uint8`, `float64`:** self-explanatory, but visually longer for
  the same tokens.

## Decision

1. Signed integers `i8`, `i16`, `i32`, `i64`. Unsigned integers `u8`, `u16`,
   `u32`, `u64`. Floating point `f32`, `f64`.
2. `int` is an alias for `i64`, and `float` is an alias for `f64` (64-bit
   defaults, consistent with ADR 0007).
3. Unannotated integer literals default to `int`, and unannotated
   floating-point literals default to `float`, when inference has no other
   constraint.

## Consequences

- **OQ-031:** which spelling is canonical when the formatter writes a default
  type (`int` or `i64`). Two equal spellings conflict with canonical syntax
  unless one is chosen.
- The name of the 64-bit size/index type (`plat.ptr.2`) is still open
  (e.g. `usize` or using `int`).
- 128-bit integers, `f16`/`bf16`, and SIMD vector types are still to be designed.

## Rejected alternatives

- **Width-only names:** less approachable for everyday code.
- **Long names:** visually noisier for no token gain.
