# 0935. Native: live-range splitting at call-free loops

- **Status:** Rejected (2026-11-05, measured no gain)
- **Date:** 2026-11-04
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** none (`native.split.1` was never added)
- **Builds on:** [ADR 0386](0386-linear-scan-register-allocation.md) (linear scan), [ADR 0890](0890-optimizer-roadmap-to-match-c.md) (pass 2), [ADR 0848](0848-idle-values-leave-registers-in-loops.md) (holes round loops)

## Context

The linear-scan allocator gives a value one location for its whole life. A
value live across a call can only use a callee-saved register (five integer
ones on x86-64), and when they run out it lives in a stack slot for its whole
life, so every use reloads it, also in the hot loop that calls nothing.
fannkuch (1.42x gcc -O2) and binarytrees (1.62x) show loop scalars reloaded
from `[rbp - k]` this way. ADR 0848 already frees the registers of values a
loop does not touch; this is the opposite case: values a loop does touch.

## Decision

1. **Split before allocation,** `compiler/codegen/split.ail`, after
   `split_for_calls`, the same for every target. In a function that has a
   call, each outermost loop with a single preheader and no call (pushes
   excepted, as in the allocator) gets, at the end of its preheader, a `copy`
   of each non-constant value it reads that is defined before it (at most 16),
   and its blocks read the copies. A copy lives only in its loop, never across
   a call, so caller-saved registers can hold it.
2. **Copies share registers.** The allocator lets a value and its copies (a
   chain of `copy` back to the same value) take the same register even where
   both are live: they hold the same bits and SSA never writes either again.
   When the original has a register, its copy takes it through the existing
   copy hint, and the copy emits no move.

The weights of ADR 0622 then see a value's loop uses on the copy, so under
pressure the original (used only outside the loop) is the one that waits in a
slot. `AIL_OPT_OFF=split` turns step 1 off.

## Measured

Mac, min of 7, stage-2 compiler, promote and split on against off: runtime
neutral on the classic programs (lzr 1.01, vm 1.00, matmul 0.98) and the
stage-2 self-build 2.6% slower. ail-linux agreed (all within ±5%). See
Rejection below.

## Rejected alternatives

- **Splitting inside the allocator** (a value with a location per block). Every
  operand read in both code generators would need a per-block lookup; the MIR
  copy gives the same split with no code generator change.
- **Saving caller-saved registers around each call** (a value in a
  caller-saved register, stored before a call and loaded after). Needs new
  moves at every call site in both backends, and costs at each call instead of
  once per loop entry.
- **More callee-saved registers** (`rbp` as a general register). Changes the
  frame layout every stub and the unwinder rely on; helps only up to one more
  value.
- **Splitting values defined in the loop and live after it.** Needs a copy on
  each exit and new parameters where exits meet; left for a measured need.

## Consequences

Loops that call nothing keep the values they read in registers even in
functions with many values live across calls. Each split costs at most one
move per entry to the loop, none when the value already has a register.

## Rejection (2026-11-05)

Not merged. Measured on ail-linux at the lane's final head (09db8282,
promote and split together, interleaved, passes on against off): every
benchmark within ±5%, lzr 0.95 (slower), and the stage-2 self-build 794 ms
(2.8%) slower. Self-host fixed point and the pass goldens were green, so the
change was correct but bought nothing. Likely cause: register holes round
loops (ADR 0848), load forwarding (ADR 0816) and struct scalar replacement
(ADR 0923) already remove most of the loads these passes target, and the
remaining ones hit in L1. Under ADR 0076 (speed is the fourth criterion,
compile time after runtime) a pass with no runtime gain and a compile-time
cost is not kept. Revisit only with a profile showing slot reloads dominating
a hot loop.
