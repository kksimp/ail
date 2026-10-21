# 0834. Native sized integers are `Ty.Sized(kind)`, held sign- or zero-extended in a register

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture
- **Builds on:** [ADR 0194](0194-sized-numeric-types.md), [ADR 0820](0820-driver-level-unsafe.md), [ADR 0823](0823-lower-ail-feature-files.md)

## Context

`Ty` in `compiler/mir/mir.ail` had only the 64-bit `int`, so no `i8 i16 i32 u8
u16 u32` value built natively. Native `*T` and `std.hw` (ADR 0820 step 2) need
them: MMIO registers are 32 bits wide and `*u8` is the C-string type. The
interpreter behaviour is the spec (`type.num.13-15`).

## Decision

1. **A new variant `Ty.Sized(kind)`** for `i8 i16 i32 u8 u16 u32`, kept apart
   from `Ty.Int` so no existing `Int` path changes. `u64` and `f32` follow
   later (`f32` reuses `Ty.F32` and `f32_round` from the native tensor work).
2. **Representation.** A sized value is one 64-bit register, sign-extended
   (signed kinds) or zero-extended (unsigned kinds) from its width. So
   comparison, interpolation, `print` and indexing use the `int` paths.
3. **Conversions** (`to_u8()` and so on, `to_int()`) check the range and
   trap with `type.num.15` and the interpreter's text, "`300` doesn't fit in
   `u8`". The message is built by the runtime (`rt.sized_msg`, trap kind 15).
   `to_int()` on a sized value is a move.
4. **Where.** `compiler/mir/lower_sized.ail` holds the lowering and
   `compiler/rt/native/sized.ail` the runtime. `lower.ail` changes only in
   the `Ty`-driven places: `type_of` (kind names), the output and dispatch
   arms (`| Sized(_)`).
5. **Order.** PR 1 here: conversions, comparison, printing, and sized
   parameters, returns and fields. PR 2: checked arithmetic and
   `wrapping_*`, `checked_*`. PR 3: scalar `f32`. PR 4: sized list elements,
   `Opt` and map keys. The 16- and 32-bit loads and stores for `*T` come with
   native unsafe (ADR 0835).

## Rejected alternatives

- **Only `*u32`/`*u8` pointers, no sized values.** Writing a register still
  needs a `u32` value, so values come first.
- **`Ty.Int` plus a kind tag.** MIR would not know the width at an
  operation, and each site would re-derive it.
- **A 32-bit-only subset (`u32`, `u8`).** The range checks are the same code
  per kind, so it saves little.
- **Wrapping conversions.** The interpreter traps (`type.num.15`); native
  must print what `ail run` prints.

## Consequences

Programs that use sized numbers start to build natively. Part 2 adds checked
`+ - * / %`, negation, `abs`, `wrapping_*` and `saturating_*` (not
`saturating_mul` on `u32`, which needs a 128-bit product), with the
interpreter's trap texts; `checked_*` waits for `Opt` of a sized integer
(part 4). Tests `sized_basics` and `sized_trap`
in `bootstrap/tests/native/run/` check the executable against `ail run`.

Part 4 lets the sized integers (not `f32` yet) be list elements, `Opt`
payloads, tuple elements, map keys and values, set elements and `match`
subjects. They are stored as 8-byte integer slots like an `int`, so sorting,
`contains`, `min`, `max`, equality and printing use the `int` paths of the
runtime, and the runtime needs no change. `sum` on a list of a sized integer
and `op=` on a map entry of one stay `native.subset.1`, because both need the
type's overflow check. `f32` in these places follows once scalar `f32` has
merged.

Part 3 adds scalar `f32` as `Ty.F32` in a float register (`is_float` is true
for it), in `lower_f32.ail`. `+ - * /` are the `float` operation followed by
the `f32_round` primitive, `to_f32` rounds once (an `int` through
`rt.int_to_f32`, which folds the bits below a `float`'s 53 into a sticky bit so
no value rounds twice), and the text is the shortest digits that read back as an
`f32` (`digits_of_f32` in `rt/native/fmt.ail`), in the interpreter's shape.
`f32` in lists, maps, `Opt` and tuples, `min`, `max`, `cmp` and the
transcendental methods are `native.subset.1` until part 4.
