# 0194. Sized numeric types: contextual literals, same-type arithmetic, trapping overflow

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 4 — Type system
- **Spec rules:** [type.num.13–14](../spec/04-types.md)
- **Evidence:** deciding criterion: AI understanding (no ambiguity, local reasoning). Tokens don't differ: no new syntax.

## Context

[ADR 0055](0055-numeric-type-names.md) names `i8`…`u64` and `f32`, but the
bootstrap knew only `int` and `float`, so the spec's own `gram.fn.2` example
(`fn add(a i32, b i32) i32`) could not run. The spec says what the types are
and that overflow traps ([ADR 0035](0035-integer-overflow-traps.md)); it
doesn't say how literals get a sized type, or how sized types mix.

## Options considered

### Option A — contextual literals, same-type arithmetic only (chosen)
- **Pros:** reads locally: an operation's type is its operands' type. A literal
  takes its type from where it is used, as `int` literals already do for
  `float` (`type.num.10`). No hidden widening, so no hidden overflow.
- **Cons:** `a.to_u32() + b` is needed to mix widths.

### Option B — implicit widening to the larger type
- **Pros:** fewer conversions.
- **Cons:** two integer types give a third result type, signed with unsigned has no
  safe answer, and the spec (`trait.op.3`) already forbids implicit conversion.

### Option C — wrap on overflow for sized types
- **Pros:** matches hashing code.
- **Cons:** contradicts ADR 0035, and the wrapping methods already cover the case.

## Decision

1. Add `i8 i16 i32 u8 u16 u32 u64 f32`; `i64` and `f64` stay spellings of `int`
   and `float` (`type.num.4`).
2. An integer literal takes the sized type its context expects, on either side
   of an operator, and must fit (`type.num.13`). A `-` on a literal belongs to it.
3. Arithmetic and comparison are between equal types only; overflow and
   division by zero trap exactly as for `int`; unsigned types have no unary `-`;
   `f32` has no `%` (`type.num.14`).
4. Sized integers have the `checked_`, `wrapping_`, `saturating_`, `abs` and
   `to_hex` methods of ADR 0137, computed in the type.
5. Sized numbers are `Eq` and `Ord`; all but `f32` are `Hash`.

## Consequences

- Locals have no type annotation, so a sized local comes from a parameter, field,
  return value or conversion ([ADR 0195](0195-sized-numeric-conversions.md)).
- Not in this change: `u64` literals above `i64::MAX` (the lexer's limit,
  `lex.num.3`), `for` ranges and indexes over sized integers (indexes are `int`,
  `type.num.5`), user `impl` blocks on sized types, wrapping or saturating
  division, and 128-bit and `f16` types (still ADR 0055 follow-ups).
- The interpreter holds a sized value as a tag plus 64 bits; arithmetic is done in
  128 bits and range-checked. A compiled backend would use native widths.

## Rejected alternatives

- **Implicit widening:** see Option B.
- **Wrapping sized arithmetic:** see Option C.
