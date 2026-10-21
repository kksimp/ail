# 0624. Register allocation: spill the value that is cheapest to keep in memory

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** native.build.1 (output unchanged)
- **Builds on:** [ADR 0386](0386-linear-scan-register-allocation.md)

## Context

Linear scan (ADR 0386) spilled, when registers ran out, whichever value
ended last. A bound like `n`, used in every loop test, ended last in `main`
and went to the stack, so matmul's loop compared against `[rbp-0x30]`.

## Decision

`codegen/alloc.ail` weighs each value by its definitions and uses, each
counted 8 times over per enclosing loop (at most 4 deep, loops from
`find_loops`). When a register is needed, the value with the lowest weight
among the candidates is spilled (the one ending last among equals).

## Measurements

The effect is small on the perfbench set (matmul 49.78M to 49.64M
instructions; `n` now stays in a register in `main`) and larger where
many values are live in one function. It is the base for splitting live
ranges around calls, which the call-heavy programs (fannkuch, binarytrees)
still need.

## Rejected alternatives

- **Weight divided by interval length (graph-colouring style):** no case in
  the perfbench set differed; the simpler comparison wins.
