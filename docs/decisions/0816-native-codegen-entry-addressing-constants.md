# 0816. Native code generation: parameters stay put, scaled addressing, held constants

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture
- **Builds on:** [ADR 0611](0611-overflow-checks-by-range-guards.md), [ADR 0625](0625-address-folding.md) (address folding)

## Context

After the bounds-check work, the integrator asked for the code generation
items that are not register allocation, in lockstep on both ISAs (ADR 0008):
the argument swap at function entry, scaled addressing, and loop-invariant
constants. Profiles of wordfreq, binarytrees and fannkuch (callgrind, per
instruction) showed 30% of executed instructions were register-to-register
moves.

## Decision

1. **Parameters stay in their arrival registers.** The allocator picked the
   first free register of its pool for a parameter, which put the first two
   arguments in `rsi, rdi` (arriving in `rdi, rsi`) and cost three moves at
   entry. A parameter no hint places now takes the register it arrived in
   when that register is allocatable and free (`Target.int_args`,
   `float_args`; both ISAs). Hints still win, since a move inside a loop
   costs more than one at entry.

Instructions executed, before → after: wordfreq 254.6M → 240.9M, binarytrees
412.2M → 399.5M, fannkuch 284.6M → 274.8M, sieve 128.8M → 125.6M, fasta
81.3M → 79.6M; collatz, spectralnorm and matmul unchanged, and L1 data
misses on matmul (9,813,919 → 9,813,921) and fannkuch (1,247,171 →
1,247,169) unchanged.

## Consequences

- A parameter's register is held from the entry, so a callee-saved register
  is not wasted on it; a parameter that lives across a call still goes to a
  callee-saved register, because the pool decides first.

## Rejected alternatives

- **Preferring the arrival register over a hint.** A hint is a move on a
  loop edge or at a call; the arrival move happens once per call.
- **Fixing the pool order** (putting `rdi` before `rsi`). It would work for
  two parameters and swap again for three.
