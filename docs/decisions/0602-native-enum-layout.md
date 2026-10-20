# 0602. Enum values in native code

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated the native self-compile lanes on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.subset.1](../spec/23-native.md), type.enum.1-6, own.val.1
- **Builds on:** [ADR 0384](0384-mir-shape-and-native-subset.md) (MIR), lane 120's structs (kind-4 objects, `rt.obj_new`), the heap header of [native-abi.md](../design/native-abi.md) §3

## Context

`ail build compiler` needs enums: the compiler's own code declares 20 of
them and matches on them everywhere. Measured over `compiler/`: 4 have no
variant with fields (`BinOp` with 13 variants, `UnOp`, `CompKind`,
`Header`); 11 have fields in every variant (`ast.Expr` with 27, `Stmt`,
`TypeShape`, `Pattern`...), recursive and payload-heavy; 5 mix both, and
they are the hot ones: `mir.Ty` (`Int`, `Float`, `Str`... beside
`List(elem)`, `Struct(name)`), the checker's `Ty` (22 variants, 13 with
fields) and the lexer's `Tok` (14, 9 with fields). The fieldless variants
of the mixed enums (`Ty.Int`, `Tok.Newline`) are built constantly.

## Decision

1. **An enum with no fields in any variant is an `int`:** the variant's
   index in declaration order. No allocation; `==` is an integer compare.
2. **Any other enum value is a pointer to a kind-5 heap object:** slot 0
   holds the variant's index, slots 1.. the variant's fields in order, 8
   bytes each, after the usual header word. A variant without fields of
   such an enum is **one shared object**, built by the generated
   `init.enums` at start-up into a program global (ADR 0604); `Ty.Int` is
   then a load from that global, not an allocation.
3. **Enum values are immutable and shared, never copied.** No assignment
   reaches into a variant (fields are read only through `match`
   bindings, expr.match.6, which are immutable), so sharing is invisible.
   A list or struct put into a variant is copied when the variant is built
   from a place, and a binding read back is copied when it is bound
   mutably, pushed or returned, as for structs (own.val.1).
4. **`==` and `!=`** on an enum with fields compare the variant index when
   one side is a variant without fields (`tok == Newline`, the common case
   in the compiler); otherwise they call a generated `eq.E`: the same
   pointer, or the same index and equal fields (numbers, strings through
   `rt.str_eq`, enums, and lists of those through a generated
   `eq.List[T]`). An enum holding a struct isn't comparable natively yet
   (native.subset.1).
5. **Names.** `Ty.Enum(name)` in MIR. Variants are built qualified
   (`E.V`, `E.V(...)`), bare where the expected type names the enum
   (type.enum.3), or bare when only one enum has that variant name.
   Methods on enums reuse the struct method lowering (`fn_key`,
   `struct_call`) with `self` of the enum type.

## Rejected alternatives

- **Small integers for fieldless variants of mixed enums**, pointers for
  the rest. Construction is free, but every `match` then tests "small or
  pointer" before it can read the index, and lane 171's collector, which
  is precise inside objects, would have to tell a small integer from a
  pointer in every enum slot. Shared objects cost one load and keep every
  value of such an enum a valid pointer (also what lane 175's
  pointer-or-0 optionals need).
- **Every enum boxed, including the fieldless ones.** `BinOp` and `UnOp`
  would allocate per use and compare through a call for no gain.
- **A fresh object per fieldless variant.** `Ty.Int` is built in hot loops
  of the compiler; one allocation each is pure waste when values are
  immutable.
- **Tagged two-word values in registers** (index plus one payload word).
  MIR values are one machine word (ADR 0384), and most variants carry more
  than one field.

## Consequences

- Enum keys carry the module's import path like structs (`ast.Expr`,
  ADR 0393); `m.E.V` resolves through the file's imports.
- `compiler/mir/lower_enum.ail` holds the lowering; `mir.Ty` gains
  `Enum(name)`; native-abi.md §2 documents the layout.
- A collector needs per-variant pointer maps for kind 5 (slot 0 is never a
  pointer); until then lane 171 scans kind-5 slots conservatively.
- Generic enums, error enums and printing an enum remain outside the
  subset.
