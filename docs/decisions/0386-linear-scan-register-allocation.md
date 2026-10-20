# 0386. Register allocation by linear scan over reverse postorder, one location per value

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** none (no observable behaviour)
- **Builds on:** [ADR 0384](0384-mir-shape-and-native-subset.md), [ADR 0385](0385-x86-64-code-generation.md)

## Context

The brief allows "linear scan is enough" for the first native slice. The
allocator must serve x86-64 and ARM64 alike (ADR 0008), and stay small enough
to port into a later optimising tier.

## Decision

`compiler/codegen/alloc.ail`, shared by the targets: Poletto and Sarkar's
linear scan.

- Blocks are laid out in **reverse postorder**; positions number every
  instruction. A value's interval runs from its definition to its last use,
  widened over every block it is live through (iterative liveness) and to
  the end of each block that passes it to a block parameter.
- **One location per value** for its whole life: a register, a stack slot, or
  `REMAT` for constants (rebuilt at each use, never holding a register).
- A value whose interval contains a call gets a **callee-saved** register, so
  calls need no save and restore around them; the prologue saves the
  callee-saved registers used.
- **Hints:** a block parameter prefers its arguments' registers and the
  reverse, so most edge moves vanish.
- When no register is free, the interval that **ends last** is spilled.

## Options considered

- **Graph colouring (Chaitin–Briggs).** Better allocation in dense code, but
  an interference graph and coalescing are three times the code; the
  drag-race loops fit in registers already. Later tier.
- **Second-chance binpacking / interval splitting.** Splits long intervals
  around their uses; worth it once spills show in measurements. Rejected for
  now for size.
- **SSA-based colouring (Hack).** Needs the parallel-copy machinery at every
  block parameter that linear scan's hints mostly avoid; no gain at this size.

## Consequences

- A spilled value lives in its slot everywhere, so spills cost a load per use;
  measured spill counts belong in `docs/design/native-performance.md`.
- The allocator takes a `Target` (four register lists), so ARM64 reuses it.
