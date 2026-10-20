# 0195. Sized numbers convert with `to_<type>()` methods

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 4 — Type system
- **Spec rules:** [type.num.15](../spec/04-types.md)
- **Evidence:** `tools/tokbench/run.sh` on a 5-line checksum loop written three ways (mean of five tokenizers): `x.to_u32()` 64.0, `u32(x)` 64.4, `x as u32` 62.2.

## Context

`type.num.9` already converts with methods (`n.to_float()`, `x.to_int()`), and
`type.num.12` has `to_hex`. Sized types need conversions in both directions.

## Options considered

### Option A — `x.to_u8()` methods (chosen)
- **Pros:** one rule for every conversion (`to_float`, `to_int`, `to_u8`); the
  method list in diagnostics and `ail api` shows every conversion a type has;
  chains left to right.
- **Cons:** `as` is about 3% cheaper on the measured loop.

### Option B — `u8(x)` constructor-style calls
- **Cons:** tokens tie with A (+0.6%), but a call named after a type collides with
  struct construction (`type.struct.3`).

### Option C — `x as u8`
- **Pros:** cheapest on the loop.
- **Cons:** `as` is reserved for FFI casts (`ffi.decl.1`) and parses as an
  operator needing precedence rules. Contextual literals (ADR 0194) make
  conversions rare, so the 3% is under ADR 0076's "rare constructs" threshold.

## Decision

Integer types (including `int`) have `to_i8 to_i16 to_i32 to_u8 to_u16 to_u32
to_u64 to_f32`; `float` has `to_f32`; sized numbers have `to_int` and
`to_float`. An integer conversion traps with `type.num.15` when the value
doesn't fit. Float conversions round to nearest, ties to even, and don't trap.
Nothing converts implicitly.

## Consequences

- A conversion to the type a value already has is rejected (`to_u8` on a `u8`):
  there is no such method, so the diagnostic lists the real ones.
- `i64` and `f64` are `int` and `float`, so `to_i64` doesn't exist; use `to_int`.

## Rejected alternatives

- **`u8(x)`** and **`x as u8`:** see above.
