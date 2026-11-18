# 1144. Native: element-wise loops four elements a pass (a `Vec4`: one 256-bit register on x86-64, two 128-bit on ARM64)

- **Status:** Accepted
- **Date:** 2026-11-18
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for lanes on the
  remaining speed gaps (2026-11-17), runtime speed only, no syntax change
- **Topic:** 25 — Optimization strategy
- **Spec rules:** [native.vec.5](../spec/23-native.md)
- **Extends:** [ADR 1142](1142-element-wise-vector-loops-and-list-alias-classes.md) (its rejected "four lanes on
  x86-64-v3 now"), [ADR 1066](1066-pair-adjacent-float-operations.md); rests on [ADR 0005](0005-x86-64-baseline-v3.md)

## Context

ADR 1142 gave element-wise `float` loops two lanes. On ail-linux axpy then ran in 215 ms against 87 ms for
gcc -O3 with AVX2, a 2.4x gap that is only the register width: the x86-64 baseline is x86-64-v3 (ADR 0005), so
256-bit operations need no run-time test, and the assembler already had them for the `rt.vec_*` kernels. On the
Mac a hand-written loop with two 128-bit operations a pass ran 8% faster than one (`neon2-mac.txt`, 75 against 82
ms). ADR 1142 kept four lanes for later because ADR 0008 wants both instruction sets in step.

## Decision

1. **A four-lane value, `Vec4`, in MIR.** Four `float`s, under the rules of a `Vec2`: made only by `vecmap.ail`,
   inside one block, never across a call. `vldx vstx vfadd vfsub vfmul vfdiv vsplat` take it. MIR stays the same
   for every target.
2. **x86-64: one ymm register** (`vmovupd`, `vaddpd` and so on with the 256-bit form, `vbroadcastsd` for a
   splat). A spilled `Vec4` takes four stack slots.
3. **ARM64: two `Vec2`s.** `compiler/mir/vec4.ail` rewrites each `Vec4` into its two halves before the backend
   allocates registers: one operation per half, the second half's accesses at the byte index plus 16 (one add per
   index and pass), a splat shared by both. The backend and the allocator see only `Vec2`.
4. **Three loops in front of the original.** Single iterations until the first store's element is on a 32-byte
   boundary (0 to 3; ADR 1142 peeled to 16), then four a pass while four remain, then two a pass (at most once),
   then the original loop for what is left. All vector loops count in bytes as before.
5. **`vend`.** After the vector loops x86-64 runs `vzeroupper`: code outside AIL (the C library, system
   libraries) may use the registers without VEX and is slowed by a dirty upper half. No code on ARM64.
6. **Exact results, as ADR 1142.** One scalar operation per lane, operands in order, never fused; nothing
   reassociated; nothing in the body can trap.
7. **Knob.** `AIL_OPT_OFF=vmap` as before (it turns all the vector loops off).

## Rejected alternatives

- **A run-time CPU test or two versions of the loop.** Not needed for 256 bits: AVX and AVX2 are in the baseline
  (ADR 0005). It would be needed for AVX-512, which is not taken here: its gain over 256 bits on these loops is
  bounded by memory speed, and ADR 0005 puts it behind dispatch.
- **A target-specific vector pass.** The MIR pipeline does not know the target, and MIR text is the same on every
  host (the `.mir` goldens). Splitting in the ARM64 backend keeps that.
- **A `Vec4` in two registers through the ARM64 allocator.** Every location, move and spill would need a
  two-register form for one user; the rewrite to `Vec2` needs none.
- **Unrolling the two-lane loop on x86-64 instead of 256-bit registers.** Measured slower than not unrolling on
  ail-linux (`neon2-linux.txt` variant 5).
- **Pointer-bumped addressing on ARM64 (`ldp q` on a moving pointer).** One more register and add per list a
  pass; the register-offset form with one shared `index + 16` is the shape ADR 1142 measured as fast.
- **Fused multiply-add.** Changes the bits of results; ruled out by native.vec.4.
