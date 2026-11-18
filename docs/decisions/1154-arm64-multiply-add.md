# 1154. arm64: an unchecked multiply and the add after it are one `madd`

- **Status:** Accepted
- **Date:** 2026-11-18
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked on 2026-11-17 for lanes on the open slowdowns against C, runtime speed only, no syntax change
- **Topic:** 25 — Optimization strategy
- **Spec rules:** none (instruction selection; no behaviour changes)
- **Builds on:** [ADR 0623](0623-branch-and-codegen-peepholes.md), [ADR 0008](0008-isa-priority-policy.md)

## Context

The key sequence of the map micro-benchmarks, `seed = (seed * 3877 + 29573)
% 139968`, is a chain each pass waits on. The arm64 code was `mul` then
`add`; clang's is one `madd`. Without a map the loop took 0.694 s for 200
million passes against 0.594 s for its C twin (Mac,
`hotpaths-job-mac-madd.txt`).

## Options considered

### Option A — fuse in the arm64 code generator (chosen)

In `Gen.gen_block` of `compiler/a64/gen.ail`: an unchecked `mul` (`site`
-1) read only by an unchecked `add`, or subtracted by an unchecked `sub`,
later in the same block with nothing but constants between them, is emitted
at the `add` as `madd` (`msub`). Constants are rebuilt at their uses and
emit nothing in between, so no register changes from the `mul`'s place to
the `add`'s.

- **Pros:** twenty lines, no MIR change, nothing for other passes to learn.
- **Cons:** only within a block.

### Option B — a `madd` MIR operation

- **Cons:** every pass that reads `mul` and `add` (ranges, folding,
  vectorizer) would need the new operation; x86-64 has no such instruction.

## Decision

Option A. Kept apart, because they are already better: a factor that is a
power of two or 3, 5, 9 (one shift or shifted `add`), and a constant addend
an `add` takes as an immediate (under 4096). A checked `mul` or `add` keeps
its own instruction, since the overflow test reads its flags or high half.
`AIL_OPT_OFF=madd` turns it off.

**x86-64 (ADR 0008):** there is no multiply-add for integers; `imul` and
`add` (or `lea` for the small factors) is what clang emits there too, and
the x86-64 generator is unchanged.

Keeping constants a loop uses in registers, the second item of the same
review, is not in this change; it needs the allocator to hold a constant
across a loop and will amend this ADR.

**Amendment, 2026-11-18 — a stored 0 is `xzr`.** `store`, `set` and `stx`
of the constant 0 (`false`, `'\0'`) name the zero register instead of
loading 0 into a scratch register first: one instruction fewer in a loop
that clears elements (the sieve's inner loop was five instructions for
clang's four). x86-64 already stores an immediate.

## Consequences

Mac arm64, best of 5, main 32371c1b against the change (measured together
with ADR 1153's loop bound, which does not touch these two programs' hot
loops): mapseed 0.694 s to 0.593 s (C twin 0.594 s); fasta 0.404 s to
0.379 s (C twin 0.638 s). Other benchmark programs within noise.

## Rejected alternatives

- **Option B:** cost in every pass for one target's instruction.
- **Fusing a checked multiply:** `madd` sets no flags and has no high half.
