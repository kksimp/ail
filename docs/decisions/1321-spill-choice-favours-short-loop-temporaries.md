# 1321. The register allocator keeps short temporaries in loops out of memory

- **Status:** Accepted
- **Date:** 2026-11-28
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer made ail-llm speed top priority (kksimp/ail#1598, item 1)
- **Topic:** 25 — Optimization strategy
- **Spec rules:** none (instruction selection and register allocation; no behaviour changes)

## Context

When a function runs out of registers, `compiler/codegen/alloc.ail` spilled the value with the lowest use weight.
In ail-llm's two-row kernel that was often a temporary read on the next instruction inside the loop: spilling it
costs a store and a load and frees a register for two positions only, so a chain of accumulator versions went
through memory one by one.

## Decision

The spill choice compares cost, not weight: a value whose interval is at most 64 positions and that is defined or
used inside a loop costs eight times its weight. Ties still spill the value that lives longest. The lane measured
7.7% less time on the two-row kernel. The loop scan runs only for functions that already run out of registers.

## Rejected alternatives

- **Weight divided by interval length (the textbook spill metric).** Changes every spill choice at once; the narrow
  rule fixes the measured case and leaves other code as it was.
- **A second allocation pass with splitting.** Slower to compile and much larger.
