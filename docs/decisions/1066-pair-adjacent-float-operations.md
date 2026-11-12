# 1066. Native: pair adjacent float element operations into two-lane vector operations

- **Status:** Accepted
- **Date:** 2026-11-12
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for the nbody gap to be closed in 0.1.0
- **Topic:** 25 — Optimization strategy
- **Spec rules:** [native.vec.3](../spec/23-native.md)
- **Extends:** [ADR 0246](0246-simd-vectorisation.md), [ADR 0845](0845-vectorization-order.md)

## Context

nbody ran at 1.22x the C twin (Mac, main 312099e4, 20M steps: AIL 929 ms, C 760 ms).
The inner loop was already free of checks. Building the C twin with its
compiler's pairing of straight-line code turned off gave 885 ms, so 124 ms of
the 168 ms gap is two-lane vector code for the x and y components: one load,
subtract, multiply and store where the scalar code has two. Sharing element
addresses (60 to 48 instructions, ADR 1064 "not covered") changed nothing,
because the integer work was never the limit.

Until now MIR had no vector value: ADRs 0942 and 0945 replace whole loops with
hand-written kernels.

## Decision

1. **A two-lane value in MIR.** `Ty.Vec2` is two `float`s in one 128-bit
   register, lane 0 low. Its operations are `vldx`, `vstx`, `vfadd`, `vfsub`,
   `vfmul`, `vfdiv`, `vsplat`, `vpack`, `vlane` and `vhadd` (`mir.ail`).
2. **A `Vec2` never leaves its block and is never alive over a call.** The
   allocator then needs two things only: a `Vec2` takes a caller-saved float
   register (AAPCS64 keeps the low half of v8-v15 only; Windows x86-64 saves
   xmm6-15 as one `float`), and a spilled one takes two stack slots.
3. **The pass (`compiler/mir/slp.ail`)** runs after address folding, per
   block. Two stores of `float`s to elements `k` and `k + 1` of one base and
   index seed a pair; operands pair bottom-up through `fadd fsub fmul fdiv`
   to adjacent loads; a shared operand is splat; `u + w` over the lanes of one
   operation becomes `vhadd`. Lanes keep operand order, nothing is fused or
   reassociated, so results are bit-identical (ADR 0246 item 4).
4. **Memory order.** A paired load sits at its earlier load, a paired store at
   its later store. The block is left alone unless every pair of accesses
   that may touch the same memory (one a store) keeps its order and no access
   crosses an instruction that can trap or call. Two accesses are known to be
   apart only when base, index value and element size are equal and the
   constant offsets differ.
5. **Cost.** A block changes only when it loses at least two instructions,
   counting splats, gathers (two) and lane extracts.
6. **Both ISAs in one change** (ADR 0008): NEON `.2d` forms on ARM64 and VEX
   `xmm` forms on x86-64. `AIL_OPT_OFF=slp` turns the pass off.

## Rationale

Criterion 4 (speed) at no cost to tokens, understanding or readability: the
source is unchanged. Equal bits make the pass invisible, so it needs no
language rule beyond the permission in native.vec.3.

## Consequences

- The backends have vector values now; wider pairs (reductions, four lanes
  with AVX) can build on them.
- Only check-free accesses (`ldx`, `stx`) pair, so the pass follows bounds
  check elimination: code whose checks stay is unchanged.

## Rejected alternatives

- **Rewriting the benchmark** to keep body `i`'s velocity in locals (832 ms).
  It is not like-for-like with the C twin, which keeps it in memory.
- **A hand-written kernel for the nbody loop**, as ADRs 0942 and 0945 do for
  their loops. The shape is too particular; a kernel would help one program.
- **Swapping the operands of `+` and `*` to make more pairs.** x86-64 returns
  the first operand's payload when both are NaN, so the bits could change.
- **Pairs across blocks or loop iterations.** They need vector block
  parameters, 128-bit moves and loop holes in the allocator; nbody does not
  need them.
- **`vhaddpd` on x86-64.** It is three micro-operations; `vunpckhpd` and
  `vaddsd` are two and keep the lane order explicit.
