# 0533. Native calls pass integer arguments past the registers on the stack

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.subset.1](../spec/23-native.md)
- **Builds on:** [ADR 0385](0385-x86-64-code-generation.md), [ADR 0388](0388-arm64-code-generation.md), [ADR 0404](0404-native-calling-convention-and-value-layouts.md)

## Context

The native subset rejected any function with more than six integer-like
parameters. The compiler's own code has functions with seven to nine (the x86-64
assembler's VEX encoders, the Mach-O writer's `section`, `Lower.assign_slot`), so
`ail build compiler` can't happen without them. ARM64 also used only six of
its eight argument registers, against the C convention native-abi.md §1
promises for `extern` calls.

## Decision

1. Integer-like arguments go in the platform's registers, six on x86-64
   (rdi rsi rdx rcx r8 r9) and **eight on ARM64** (x0-x7). The rest go on
   the stack, 8 bytes each, the first one lowest, as System V and AAPCS64
   do for 8-byte values.
2. The caller reserves the area, rounded up to 16 bytes, just before the
   call, stores the extra arguments, and frees it after. The callee loads
   them from above its frame record (`[rbp+16]`, `[x29+16]`) in its entry
   moves. On ARM64, spill slots are addressed from `sp`, so the generator
   adds the reserved size to slot offsets while the area exists.
3. Float parameters stay limited to eight (the float registers); more is
   reported under native.subset.1 at the function.

## Consequences

- Any number of integer, `bool`, `str`, list and struct parameters builds.
- A call with stack arguments costs a stack adjustment and one store per
  extra argument; calls with register arguments only are unchanged.
- `bootstrap/tests/native/run/stack_args.ail` checks mixed, recursive and
  nested calls on both targets; `subset_ext` checks the float limit.

## Rejected alternatives

- **Pack extra arguments into a heap block.** An allocation per call, and
  not the C convention, so `extern` calls would need a thunk.
- **Reserve a fixed outgoing-argument area in every frame.** Saves the
  adjustment, but every frame grows for the few calls that need it.
- **Keep six registers on ARM64 for symmetry.** Breaks the C convention for
  `extern` calls with seven or eight integer arguments.
- **Stack floats now too.** No program in the compiler or the drag race has
  more than eight float parameters; it can follow when one does.
