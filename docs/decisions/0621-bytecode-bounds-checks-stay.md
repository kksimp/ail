# 0621. Bytecode tier: bounds checks stay; no safe way to remove them

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 20 — Interpreter
- **Builds on:** [ADR 0619](0619-bytecode-calls-loops-and-fusion.md), [ADR 0620](0620-bytecode-element-method-calls.md)

## Context

After ADR 0620 the tier is about 0.95-0.97x of CPython (geometric mean over the
compute programs). Programs still above 1.0x are binarytrees (1.5x), sieve
(1.1-1.3x) and matmul (1.1-1.3x). Callgrind on binarytrees (depth 10, 368 M
instructions) puts 60% of the instructions in the `exec` loop itself: slice
indexing 11%, `mem::replace` 8%, `Vec` handling 5%, pointer arithmetic 5%,
`run_code` 5%, `run_call` 3%. There are no hand-offs left in these programs.
Sieve (55 M bytecode instructions, about 9 ns each) has the same shape.

## Decision

Keep the per-operand bounds checks. The loader validates every register and
constant index, but the compiler cannot see that proof across the loader:
`regs[i]` and `consts[i]` stay checked unless the access is `unsafe`
(`get_unchecked`), which the designer has not approved (decision card open).
Safe forms do not help: fixed-length slices and one up-front assert don't let
the optimiser drop checks on indices that come from instruction fields at run
time, and replacing `mem::replace` with an enum swap moves the same stores.

The measured ceiling for unchecked access is about 12% fewer instructions
(lane 190, step 1). Beyond that the remaining cost is the design: operands are
decoded per instruction, locals live in a `Vec<(&str, Value)>`, and each call
recurses through `run_code` and `exec`.

## Rejected alternatives

- **Audited `unsafe` unchecked access.** About 12% fewer instructions, but
  needs the designer's approval; the card stays open.
- **A frame with locals in the register file and non-recursive calls.** The
  real fix for calls (binarytrees), but a redesign of the tier's environment
  that the walker shares; a lane of its own, not an end-of-night change.
- **Per-program fusion (a sieve-inner-loop instruction).** Wins one benchmark
  and nothing else.
