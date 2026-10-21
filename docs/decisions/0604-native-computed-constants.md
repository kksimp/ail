# 0604. Computed module constants in native code

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated the native self-compile lanes on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.subset.1](../spec/23-native.md), mod.const.1, own.val.1
- **Builds on:** [ADR 0384](0384-mir-shape-and-native-subset.md) (constants inlined when they are constant expressions), [ADR 0602](0602-native-enum-layout.md)

## Context

The compiler has constants that aren't constant expressions: tables
(`PRIMS := {...}`, `BIT_OPS := {...}`), lists, register lists, values
built by calls. native.subset.1 rejected them; 17 were reported over
`compiler/`.

## Decision

1. A constant whose value is a literal, another constant or arithmetic on
   them stays inlined at each use (ADR 0384). **Any other constant gets a
   program global slot** after the runtime's (`RT_GLOBALS`, 16, which
   lane 171's collector slots fit under).
2. The generated **`init.consts`** computes them into their slots;
   `main` calls `init.enums` (ADR 0602) and then `init.consts` before its
   first statement. A use reads the slot.
3. **Order:** declaration order, except that a constant named in another's
   expression is computed first (depth first over the names it reads).
   The interpreter computes a constant on first use; the two agree unless
   a constant's computation prints, traps, or reaches a later constant
   through a function's body. A trap still reports the constant's line.
4. Values follow own.val.1 as for any place: binding a constant's list to
   a `mut` variable, pushing it or returning it copies it.
5. The executable's global area is sized from the slots the MIR uses
   (`Module.globals_used`).

## Rejected alternatives

- **Lazy evaluation on first use**, as the interpreter does: exact, but a
  flag test on every read of a table the compiler reads in hot loops.
- **Strict declaration order:** `PRIMES := [2, SMALL]` before
  `SMALL := f()` would read an empty slot.
- **Building constant data in the read-only section:** only works for
  values with a fixed layout and no heap objects the program may copy; the
  start-up function covers every type in the subset in one path.

## Consequences

- `compiler/mir/lower_const.ail` holds the lowering.
- A cycle between constants is the checker's error; the native order just
  cuts it.
- Constants of types outside the subset (maps until lane 173) are reported
  once; their uses stay quiet.
