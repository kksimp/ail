# 0574. Native optionals: a pointer or 0, scalars boxed

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.subset.1](../spec/23-native.md), err.prop.2, err.else.1, err.else.6, expr.if.5
- **Builds on:** [ADR 0384](0384-mir-shape-and-native-subset.md) (MIR), [ADR 0407](0407-bump-allocator-no-header.md) (allocator), the heap header in [native-abi.md](../design/native-abi.md)

## Context

The compiler uses `T?` throughout. A MIR value is one register, and the
backend has no pair values; a two-register optional would touch parameters,
returns, block parameters and the allocator, which are lane 120's files.

## Decision

`T?` is **one register**: 0 is `none`. A `str`, a list, a struct or an enum with fields is its own pointer
(never 0). An `int`, `float`, `bool` or fieldless enum is boxed: a pointer to 8 bytes
(`rt.box`, `rt.boxf`, `rt.box_bool`; read with `rt.unbox*`). No `T??`.
MIR gets the type `Opt(T)`; the lowering is `compiler/mir/lower_err.ail`.

- A plain `T` where a `T?` is expected becomes `some` in `convert`.
- `none` is the constant 0 typed by its context; `x == none` and
  `x != none` compare with 0.
- `if v := o`, `x := o else d` (a value, a leaving statement or a block),
  `o else d` and `o?` (in a function returning an optional: return 0)
  branch on `o != 0` and unwrap in the present block.
- All boxing goes through the six `rt.box*`/`rt.unbox*` functions, so the
  heap lane (171) changes one place to give boxes the kind-10 header.

**Measured** (x86-64, 30M calls of `pick(i) else 0` against a function
returning -1): 0.39 s boxed against 0.09 s unboxed, about 10 ns per `Some`
of a scalar, most of it page faults on the never-freed bump heap (320 MB).
Pointer optionals cost nothing extra.

## Consequences

- Scalar optionals allocate until the collector (lane 171) reclaims boxes;
  a hot loop over `int?` is several times slower than the same loop over a
  sentinel. A two-register `Opt` (value and flag in MIR, rdx/x1 on return)
  is the follow-up once lane 120's backend can return pairs; callers of
  `some`/`unwrap` change, nothing else.

## Rejected alternatives

- **Two registers now.** Fastest, but needs a pair in MIR, the allocator and
  both generators: lane 120's critical files, in parallel with its work.
- **A sentinel (`i64::MIN`) for `int?`.** Wrong for programs that use that
  value, and no sentinel exists for `float` or `bool`.
- **Boxing pointers as well.** One more load for nothing.
